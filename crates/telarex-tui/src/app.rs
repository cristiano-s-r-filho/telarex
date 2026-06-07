//! Application — screen manager, event loop, and top-level state.
use crate::screens::{WelcomeView, EditorView, ConfigView, DiscoveredLodge};
use crate::components::{NodeKind, ErrorModal};
use crate::network::{NetworkManager, NetworkEvent, NetworkCommand};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{Frame, layout::Rect, widgets::{Clear, Block}, prelude::Stylize};
use crate::tui_compat::{AppContext, Component, DrawContext, Event, EventResult};
use telarex_core::config;
use telarex_core::errors::TrexError;
use telarex_core::network::auth::QuantumAuth;
use telarex_core::actor::{BufferActor, BufferActorCommand};
use tokio::sync::mpsc;
use uuid::Uuid;
use std::collections::HashMap;

use telarex_core::workspace::PendingJoin;
use crate::theme::Theme;

/// Top-level screen the application is currently displaying.
#[derive(PartialEq, Clone, Copy)]
pub enum AppScreen { Welcome, Editor, Config }

use std::time::Instant;

/// Root application controller — owns all screens, network state, and the main event loop.
pub struct App {
    current_screen: AppScreen,
    previous_screen: AppScreen,
    welcome: WelcomeView,
    editor: EditorView,
    config: ConfigView,
    error_modal: ErrorModal,
    network_rx: mpsc::Receiver<NetworkEvent>,
    network_tx: mpsc::Sender<NetworkCommand>,
    session_id: Option<String>,
    db: telarex_core::database::Database,
    buffer_tx: mpsc::Sender<BufferActorCommand>,
    peer_sync_states: HashMap<(Uuid, std::path::PathBuf, String), automerge::sync::State>,
    lodge_members: HashMap<Uuid, Vec<String>>,
    pending_join_requests: Vec<PendingJoin>,
    identity_keys: telarex_core::network::auth::Keypair,
    last_shared_lodge: Option<Uuid>,
    last_tick: Instant,
    config_data: config::TelaRexConfig,
    active_lodge: Option<Uuid>,
    theme: Theme,
    theme_engine: telarex_core::config::ThemeEngine,
    auto_save_counter: u32,
    auto_save_enabled: bool,
}

impl App {
    /// Creates a new `App`, initialising config, database, network, and screens.
    pub fn new(initial_file: Option<String>, session_id: Option<String>) -> Self {
        let config_data = config::load(session_id.as_deref()).unwrap_or_default();
        let db = telarex_core::database::Database::open().expect("Failed to open database");
        let buffer_tx = BufferActor::start();
        
        let mut theme_engine = telarex_core::config::ThemeEngine::new();
        let _ = theme_engine.load_themes("themes");
        if !config_data.editor.theme.is_empty() {
            let _ = theme_engine.set_theme(&config_data.editor.theme);
        }
        let theme = Theme::from_stylesheet(theme_engine.get_current());

        let mut welcome = WelcomeView::new(session_id.clone());
        welcome.theme = theme.clone();
        welcome.username = config_data.profile.username.clone();

        // RECENT PROJECTS: Pull from DB
        if let Ok(recent) = db.get_recent_projects() {
            welcome.recent_projects = recent.clone();
            welcome.last_opened = recent.first().cloned();
        }

        if let Ok(lodges) = db.get_my_lodges() {
            for (id, _path, name) in lodges {
                welcome.discovered_lodges.push(DiscoveredLodge {
                    id,
                    name: format!("{} (Local)", name),
                    peer_id: "me".to_string(),
                });
            }
        }

        let mut editor = EditorView::new();
        editor.apply_theme(theme_engine.get_current());
        editor.status_bar.username = config_data.profile.username.clone();
        
        let error_modal = ErrorModal::new();
        let (event_tx, network_rx) = mpsc::channel(100);
        let (network_tx, cmd_rx) = mpsc::channel(100);
        editor.set_network_tx(network_tx.clone());
        let network_manager = NetworkManager::new(event_tx, cmd_rx);
        
        let identity_seed_for_net = config_data.profile.identity_seed.clone();
        let listen_addr_from_config = if config_data.network.listen_addr.is_empty() { None } else { Some(config_data.network.listen_addr.clone()) };
        tokio::spawn(async move {
            if let Err(e) = network_manager.start(identity_seed_for_net, listen_addr_from_config).await {
                log::error!("Failed to start network: {}", e);
            }
        });

        let identity_keys = QuantumAuth::generate_identity();
        
        let auto_save_enabled = config_data.editor.auto_save;

        let mut app = Self {
            current_screen: AppScreen::Welcome,
            previous_screen: AppScreen::Welcome,
            welcome,
            editor,
            config: ConfigView::new(session_id.clone()),
            error_modal,
            network_rx,
            network_tx: network_tx.clone(),
            session_id: session_id.clone(),
            db,
            buffer_tx,
            peer_sync_states: HashMap::new(),
            lodge_members: HashMap::new(),
            pending_join_requests: Vec::new(),
            identity_keys,
            last_shared_lodge: None,
            last_tick: Instant::now(),
            config_data,
            active_lodge: None,
            theme,
            theme_engine,
            auto_save_counter: 0,
            auto_save_enabled,
        };

        if let Some(path) = initial_file {
            let _ = app.db.add_recent_project(&path);
            if let Ok(()) = app.load_file_to_editor(&path) {
                app.current_screen = AppScreen::Editor;
            }
        }
        app
    }

    fn load_file_to_editor(&mut self, path_str: &str) -> std::io::Result<()> {
        let raw_path = std::path::PathBuf::from(path_str);
        let path = if raw_path.exists() {
            std::fs::canonicalize(&raw_path)?
        } else {
            raw_path.to_path_buf()
        };

        // Update DB
        let _ = self.db.add_recent_project(&path.to_string_lossy());

        let (reply_tx, reply_rx) = std::sync::mpsc::channel();
        let _ = self.buffer_tx.try_send(BufferActorCommand::GetOrCreate { 
            path: path.clone(), 
            reply: reply_tx 
        });

        if let Ok(doc) = reply_rx.recv() {
            if matches!(self.current_screen, AppScreen::Editor) {
                self.editor.load_doc_to_active_pane(path, doc);
            } else {
                if let Some(idx) = self.editor.tabs.find_tab_by_path(&path) {
                    self.editor.tabs.active_tab = idx;
                } else {
                    let replace_initial = {
                        let active_tab = self.editor.tabs.active_tab_ref();
                        active_tab.name == "Main" && active_tab.path.is_none() && self.editor.tabs.tabs.len() == 1
                    };

                    if replace_initial {
                        self.editor.tabs.add_tab(path, doc);
                        self.editor.tabs.tabs.remove(0);
                        self.editor.tabs.active_tab = 0;
                    } else {
                        self.editor.tabs.add_tab(path, doc);
                    }
                }
            }

            // Ensure the new tab gets the current theme
            self.editor.apply_theme(self.theme_engine.get_current());
            return Ok(());
        }
        Err(std::io::Error::other("Actor communication failed"))
    }

    fn switch_screen(&mut self, screen: AppScreen) {
        if self.current_screen != screen {
            self.previous_screen = self.current_screen;
            self.current_screen = screen;
            
            // Refresh data on switch
            if screen == AppScreen::Welcome {
                if let Ok(recent) = self.db.get_recent_projects() {
                    self.welcome.recent_projects = recent;
                }
            }
        }
    }

    fn show_info(&mut self, msg: &str) {
        self.error_modal.show(TrexError::new("TRX-000", telarex_core::errors::ErrorLevel::Info, msg, "Information only."));
    }

    fn show_error(&mut self, error: TrexError) { self.error_modal.show(error); }

    fn poll_network(&mut self) {
        if self.auto_save_enabled {
            self.auto_save_counter += 1;
            if self.auto_save_counter >= 600 {
                self.auto_save_counter = 0;
                for tab in self.editor.tabs.tabs.iter_mut() {
                    for node in tab.layout.nodes.iter_mut() {
                        if let crate::components::NodeKind::Pane(ref mut editor) = node.kind {
                            if editor.is_modified() && editor.file_path().is_some() {
                                let _ = editor.save();
                            }
                        }
                    }
                }
            }
        }

        if let Some(id) = self.welcome.take_delete_request() {
            let _ = self.db.delete_lodge(id);
            self.peer_sync_states.retain(|(lodge_id, _, _), _| *lodge_id != id);
            self.lodge_members.remove(&id);
            if self.active_lodge == Some(id) {
                self.active_lodge = None;
                self.editor.status_bar.lodge_status = "Offline".to_string();
                self.editor.status_bar.peer_count = 0;
            }
            self.show_info("Lodge data and memory purged.");
        }

        while let Ok(event) = self.network_rx.try_recv() {
            match event {
                NetworkEvent::LodgeDiscovery { id, name, peer_id } => {
                    if !self.welcome.discovered_lodges.iter().any(|l| l.id == id) {
                        self.welcome.discovered_lodges.push(DiscoveredLodge { id, name, peer_id });
                    }
                }
                NetworkEvent::JoinRequest { lodge_id: _, peer_id, username, public_key } => {
                    self.editor.workspace.add_pending_join(peer_id, username.clone(), public_key);
                    self.show_info(&format!("Pending join request from {}", username));
                }
                NetworkEvent::LodgeMembersUpdated { lodge_id, members } => {
                    self.lodge_members.insert(lodge_id, members.clone());
                    if self.active_lodge == Some(lodge_id) {
                        self.editor.status_bar.peer_count = members.len() + 1;
                        self.editor.status_bar.lodge_status = "Online".to_string();
                    }
                }
                NetworkEvent::LodgeLeft { lodge_id } => {
                    if self.active_lodge == Some(lodge_id) {
                        self.editor.workspace.is_shared = false;
                        self.editor.status_bar.lodge_status = "Offline".to_string();
                        self.editor.status_bar.peer_count = 0;
                        self.active_lodge = None;
                        self.show_info("Successfully left the Lodge.");
                    }
                }
                NetworkEvent::NetworkShutdown => {
                    self.editor.status_bar.lodge_status = "Offline".to_string();
                    self.editor.status_bar.peer_count = 0;
                    self.active_lodge = None;
                    self.show_info("Disconnected from LodgeNet.");
                }
                NetworkEvent::AuthChallenge { lodge_id, challenge } => {
                    let proof = QuantumAuth::sign_challenge(&self.identity_keys, &challenge);
                    let _ = self.network_tx.try_send(NetworkCommand::SendAuthResponse { lodge_id, proof });
                }
                NetworkEvent::AuthVerify { lodge_id, challenge, proof, public_key } => {
                    if QuantumAuth::verify(&public_key, &challenge, &proof) {
                        log::info!("PQ-Auth successful for Lodge {}", lodge_id);
                        self.show_info("New member authenticated via ML-DSA.");
                        let _ = self.network_tx.try_send(NetworkCommand::AnnouncePresence { lodge_id, username: self.config_data.profile.username.clone() });
                    } else {
                        log::warn!("PQ-Auth failed for Lodge {}", lodge_id);
                        self.show_error(TrexError::auth_failed());
                    }
                }
                NetworkEvent::SyncMessage { lodge_id, path, data } => {
                    let active_tab = self.editor.tabs.active_tab_mut();
                    for node in active_tab.layout.nodes.iter_mut() {
                        if let NodeKind::Pane(ref mut editor) = node.kind {
                            if let Some(editor_path) = editor.file_path() {
                                if editor_path == path {
                                    let state = self.peer_sync_states.entry((lodge_id, path.clone(), "network".to_string())).or_insert_with(automerge::sync::State::new);
                                    if let Ok(msg) = automerge::sync::Message::decode(&data) {
                                        self.editor.sync_engine.receive_sync_message(&path, state, msg);
                                        if let Some(content) = self.editor.sync_engine.get_content(&path) {
                                            editor.set_text(&content);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn share_workspace_with_name(&mut self, name: String) {
        let id = self.editor.workspace.id;
        self.active_lodge = Some(id);
        self.editor.workspace.share(name.clone());
        let root_path = self.editor.workspace.root.display().to_string();
        let _ = self.db.register_lodge(id, &root_path, &name, true);
        let _ = self.network_tx.try_send(NetworkCommand::ShareLodge { id, name: name.clone() });
        self.last_shared_lodge = Some(id);
        
        self.editor.status_bar.lodge_status = "Online".to_string();
        self.editor.status_bar.peer_count = 1; 
        self.editor.status_bar.lodge_id = Some(id);
        
        let _ = self.network_tx.try_send(NetworkCommand::AnnouncePresence { lodge_id: id, username: self.config_data.profile.username.clone() });

        self.welcome.discovered_lodges.push(DiscoveredLodge {
            id,
            name: format!("{} (Local)", name),
            peer_id: "me".to_string(),
        });
        
        self.show_info(&format!("Workspace shared as '{}'", name));
    }

    fn reset_data(&mut self) {
        let _ = self.db.reset();
        let config_data = config::TelaRexConfig::default();
        let _ = config::save(&config_data, self.session_id.as_deref());
        self.editor.apply_theme(self.theme_engine.get_current());
        self.show_info("Application data reset successfully.");
    }
}

impl Component for App {
    fn draw(&self, frame: &mut Frame, area: Rect, ctx: &DrawContext) {
        // NUCLEAR DRAW: Force clear and background fill on every pass
        frame.render_widget(Clear, area);
        frame.render_widget(Block::default().bg(self.theme.bg), area);

        match self.current_screen {
            AppScreen::Welcome => self.welcome.draw(frame, area, ctx),
            AppScreen::Editor => self.editor.draw(frame, area, ctx),
            AppScreen::Config => self.config.draw(frame, area, ctx),
        }
        if self.error_modal.active { self.error_modal.draw(frame, area, ctx); }
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut AppContext) -> EventResult {
        self.poll_network();
        self.last_tick = Instant::now();

        if self.error_modal.active { return self.error_modal.handle_event(event, ctx); }

        // DUPLICATION FIX: Only process Press events
        if let Event::Key(key) = event {
            if key.kind != KeyEventKind::Press && key.kind != KeyEventKind::Repeat {
                return EventResult::Handled; // Consume Release events
            }
            
            match (key.code, key.modifiers) {
                (KeyCode::Char('q'), KeyModifiers::CONTROL) => { ctx.quit(); return EventResult::Handled; }
                (KeyCode::Char('c'), KeyModifiers::CONTROL) if self.current_screen == AppScreen::Welcome => { ctx.quit(); return EventResult::Handled; }
                _ => {}
            }
        }

        let handled = match self.current_screen {
            AppScreen::Welcome => {
                let result = self.welcome.handle_event(event, ctx);
                if let Some(path) = self.welcome.take_open_request() {
                    if let Ok(()) = self.load_file_to_editor(&path) { self.switch_screen(AppScreen::Editor); }
                    return EventResult::Handled;
                }
                if let Some(lodge_id) = self.welcome.take_join_request() {
                    self.active_lodge = Some(lodge_id);
                    self.editor.workspace.id = lodge_id;
                    self.editor.workspace.is_shared = true;
                    self.editor.status_bar.lodge_status = "Online".to_string(); // OPTIMISTIC
                    self.editor.status_bar.peer_count = 1; 
                    self.editor.status_bar.lodge_id = Some(lodge_id);

                    let _ = self.network_tx.try_send(NetworkCommand::JoinLodge {
                        lodge_id,
                        public_key: self.identity_keys.public.to_vec(),
                        username: self.config_data.profile.display_name.clone(),
                    });
                    self.switch_screen(AppScreen::Editor);
                    return EventResult::Handled;
                }
                if self.welcome.take_config_request() {
                    self.config.show();
                    self.switch_screen(AppScreen::Config);
                    return EventResult::Handled;
                }
                result.is_handled()
            }
            AppScreen::Editor => {
                let result = self.editor.handle_event(event, ctx);
                
                if let Some(path) = self.editor.take_file_to_open() {
                    let _ = self.load_file_to_editor(&path.to_string_lossy());
                }

                if let Some(name) = self.editor.take_share_request() {
                     self.share_workspace_with_name(name);
                }
                
                if result.is_handled() { return result; }

                if self.editor.reset_requested {
                    self.editor.reset_requested = false;
                    self.reset_data();
                    self.switch_screen(AppScreen::Welcome);
                    return EventResult::Handled;
                }
                if self.editor.take_config_request() {
                    self.config.show();
                    self.config.apply_theme(&self.theme);
                    self.switch_screen(AppScreen::Config);
                    return EventResult::Handled;
                }
                
                if let Event::Key(KeyEvent { code: KeyCode::Esc, .. }) = event {
                    if !self.editor.is_palette_active() {
                        self.switch_screen(AppScreen::Welcome);
                        return EventResult::Handled;
                    }
                }
                result.is_handled()
            }
            AppScreen::Config => {
                let result = self.config.handle_event(event, ctx);
                if self.config.should_exit {
                    self.config.should_exit = false;
                    let new_config = self.config.get_config();
                    
                    // Sync theme back to App
                    if let Ok(_) = self.theme_engine.set_theme(&new_config.editor.theme) {
                        self.theme = Theme::from_stylesheet(self.theme_engine.get_current());
                        self.welcome.theme = self.theme.clone();
                        self.editor.apply_theme(self.theme_engine.get_current());
                    }

                    // Sync editor config back to App
                    self.editor.tab_size = new_config.editor.tab_size;
                    self.editor.show_line_numbers = new_config.editor.line_numbers;
                    self.editor.apply_editor_config();
                    self.auto_save_enabled = new_config.editor.auto_save;

                    self.current_screen = self.previous_screen;
                    return EventResult::Handled;
                }
                result.is_handled()
            }
        };
        if handled { EventResult::Handled } else { EventResult::Unhandled }
    }
}


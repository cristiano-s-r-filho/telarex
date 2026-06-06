use telarex_core::command::Command;
use telarex_core::workspace::Workspace;
use crate::components::{Editor, FileTree, StatusBar, TabBar, CommandPalette, SearchPalette, SearchResult, NodeKind, TabController};
use crate::components::modals::{MacroPalette, InputModal};
use crate::events::{UIAction, KeyMapper};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    Frame,
};
use crate::tui_compat::{AppContext, Component, DrawContext, Event, EventResult};

use std::cell::RefCell;
use tokio::sync::mpsc;

use telarex_core::lsp::LspClient;
use crate::network::NetworkCommand;

use telarex_core::crdt::sync_engine::SyncEngine;

pub struct EditorView {
    pub workspace: Workspace,
    pub file_tree: FileTree,
    pub tabs: TabController,
    pub tab_bar: TabBar,
    pub status_bar: StatusBar,
    command_palette: RefCell<CommandPalette>,
    search_palette: RefCell<SearchPalette>,
    macro_palette: RefCell<MacroPalette>,
    focused_child: FocusTarget,
    explorer_visible: bool,
    config_requested: bool,
    pub reset_requested: bool,
    lsp_client: Option<LspClient>,
    doc_version: i32,
    macro_state: MacroState,
    recorded_events: Vec<KeyEvent>,
    pub sync_engine: SyncEngine,
    pub network_tx: Option<mpsc::Sender<NetworkCommand>>,
    key_mapper: KeyMapper,
    last_area: RefCell<Rect>,
    search_rx: Option<mpsc::Receiver<SearchResult>>,
    pub share_lodge_modal: InputModal,
    file_to_open: Option<std::path::PathBuf>,
    pub mode: String,
    git_sidecar: Option<telarex_core::git_sidecar::GitSidecar>,
    pending_git_commit: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum MacroState { Idle, Recording(String), Replaying }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusTarget { Explorer, Editor }

impl EditorView {
    pub fn new() -> Self {
        let editor = Editor::new();
        let file_tree = FileTree::new();
        let workspace = Workspace::new(file_tree.root.clone());
        let config = telarex_core::config::load(None).unwrap_or_default();
        let git_sidecar = telarex_core::git_sidecar::GitSidecar::open(&file_tree.root).ok();
        let mut view = Self {
            workspace, file_tree,
            tabs: TabController::new(editor),
            tab_bar: TabBar { theme: crate::theme::Theme::default() },
            status_bar: StatusBar::default(),
            command_palette: RefCell::new(CommandPalette::new()),
            search_palette: RefCell::new(SearchPalette::new()),
            macro_palette: RefCell::new(MacroPalette::new()),
            focused_child: FocusTarget::Explorer,
            explorer_visible: true,
            config_requested: false,
            lsp_client: None,
            doc_version: 1,
            macro_state: MacroState::Idle,
            recorded_events: Vec::new(),
            sync_engine: SyncEngine::new(),
            network_tx: None,
            reset_requested: false,
            key_mapper: KeyMapper::from_config(&config.keymaps),
            last_area: RefCell::new(Rect::default()),
            search_rx: None,
            share_lodge_modal: InputModal::new("Enter Lodge Name"),
            file_to_open: None,
            mode: "normal".to_string(),
            git_sidecar,
            pending_git_commit: None,
        };
        view.sync_git_status();
        view.update_focus_state();
        view
    }

    pub fn apply_theme(&mut self, ss: &telarex_core::syntax::StyleSheet) {
        let theme = crate::theme::Theme::from_stylesheet(ss);
        self.tab_bar.theme = theme.clone();
        self.file_tree.theme = theme.clone();
        // Update all editors in the layout
        for tab in self.tabs.tabs.iter_mut() {
            for node in tab.layout.nodes.iter_mut() {
                if let NodeKind::Pane(ref mut editor) = node.kind {
                    editor.apply_theme(ss);
                }
            }
        }
        self.sync_status_bar();
    }

    pub fn load_file(&mut self, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
        let path_ref = path.as_ref();
        let path_buf = if path_ref.exists() {
            std::fs::canonicalize(path_ref)?
        } else {
            path_ref.to_path_buf()
        };

        if path_buf.is_dir() { let _ = self.file_tree.change_dir(&path_buf); return Ok(()); }
        self.file_to_open = Some(path_buf);
        Ok(())
    }

    pub fn take_file_to_open(&mut self) -> Option<std::path::PathBuf> {
        if let Some(p) = self.file_to_open.take() { return Some(p); }
        if let Some(path) = self.file_tree.take_file_to_open() {
             if let Ok(canon) = std::fs::canonicalize(&path) {
                 return Some(canon);
             }
             return Some(path);
        }
        if let Some(editor) = self.get_active_editor() {
            return editor.take_file_to_open();
        }
        None
    }

    pub fn get_active_editor(&mut self) -> Option<&mut Editor> {
        let active_id = self.tabs.active_tab_mut().layout.active_pane;
        for node in self.tabs.active_tab_mut().layout.nodes.iter_mut() {
            if node.id == active_id {
                if let NodeKind::Pane(ref mut editor) = node.kind { return Some(editor); }
            }
        }
        None
    }

    pub fn get_active_editor_ref(&self) -> Option<&Editor> {
        let active_id = self.tabs.active_tab_ref().layout.active_pane;
        for node in self.tabs.active_tab_ref().layout.nodes.iter() {
            if node.id == active_id {
                if let NodeKind::Pane(ref editor) = node.kind { return Some(editor); }
            }
        }
        None
    }

    pub fn is_palette_active(&self) -> bool {
        self.command_palette.borrow().active || self.search_palette.borrow().active || self.macro_palette.borrow().active || self.share_lodge_modal.active
    }

    pub fn start_lsp(&mut self, root: &std::path::Path) {
        let (tx, _rx) = mpsc::channel(100);
        let client = LspClient::start("rust-analyzer", root, tx);
        match client {
            Ok(c) => {
                self.lsp_client = Some(c);
                log::info!("LSP started for root: {:?}", root);
            }
            Err(e) => {
                log::error!("Failed to start LSP: {}", e);
            }
        }
        self.sync_status_bar();
    }

    pub fn notify_lsp_change(&mut self, _path: std::path::PathBuf, text: String) {
        self.doc_version += 1;
        let uri = format!("file:///{}", _path.display());
        
        if let Some(client) = &mut self.lsp_client {
            let version = self.doc_version;
            client.notify_change(&uri, version, text);
        }
    }

    pub fn sync_status_bar(&mut self) {
        let (path_str, modified, pos, lang, selections) = {
            let active_id = self.tabs.active_tab_ref().layout.active_pane;
            let mut editor_ref: Option<&Editor> = None;
            for node in self.tabs.active_tab_ref().layout.nodes.iter() {
                if node.id == active_id {
                    if let NodeKind::Pane(ref editor) = node.kind { editor_ref = Some(editor); break; }
                }
            }
            if let Some(editor) = editor_ref {
                let path = editor.file_path().map(|p| p.display().to_string()).unwrap_or_else(|| "Untitled".to_string());
                let (line, col) = editor.cursor_position();
                (path, editor.is_modified(), (line, col), editor.language().unwrap_or("Plain").to_string(), if editor.selection.is_some() { 1 } else { 0 })
            } else {
                ("None".to_string(), false, (0, 0), "None".to_string(), 0)
            }
        };

        self.status_bar.file_path = Some(path_str);
        self.status_bar.modified = modified;
        self.status_bar.cursor_position = pos;
        self.status_bar.language = Some(lang);
        self.status_bar.selection_count = selections;
        self.status_bar.lodge_id = Some(self.workspace.id);
        
        self.status_bar.editor_mode = "EDIT".to_string();
    }

    fn update_focus_state(&mut self) {
        self.file_tree.focused = self.focused_child == FocusTarget::Explorer;
        self.tabs.sync_focus(self.focused_child == FocusTarget::Editor);
    }

    fn sync_git_status(&mut self) {
        if let Some(ref git) = self.git_sidecar {
            if let Ok(status) = git.status() {
                self.status_bar.git_branch = Some(status.branch);
                self.status_bar.git_dirty = status.modified + status.untracked + status.staged;
            }
        }
    }

    pub fn set_network_tx(&mut self, tx: mpsc::Sender<NetworkCommand>) {
        self.network_tx = Some(tx);
    }

    fn perform_project_search(&mut self) {
        let query = self.search_palette.borrow().get_query();
        if query.is_empty() { return; }
        let root = self.workspace.root.clone();
        let (tx, rx) = mpsc::channel(100);
        self.search_rx = Some(rx);
        
        std::thread::spawn(move || {
            use ignore::WalkBuilder;
            for result in WalkBuilder::new(root).build() {
                if let Ok(entry) = result {
                    if entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                        let path = entry.path();
                        if let Ok(content) = std::fs::read_to_string(path) {
                            for (i, line) in content.lines().enumerate() {
                                if line.contains(&query) {
                                    let sr = SearchResult {
                                        file: path.to_path_buf(),
                                        line_number: i + 1,
                                        content: line.trim().to_string(),
                                    };
                                    if let Err(_) = tx.blocking_send(sr) { break; }
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    fn execute_command(&mut self, cmd: Command, ctx: &mut AppContext) {
        match cmd {
            Command::OpenFile => { self.command_palette.borrow_mut().active = false; self.focused_child = FocusTarget::Explorer; self.update_focus_state(); }
            Command::Save => { if let Some(e) = self.get_active_editor() { let _ = e.save(); } }
            Command::Quit => { ctx.quit(); }
            Command::ToggleExplorer => { self.explorer_visible = !self.explorer_visible; }
            Command::OpenConfig => { self.config_requested = true; }
            Command::ShareWorkspace => { self.handle_action(UIAction::PromptShareWorkspace, ctx); }
            Command::LeaveWorkspace => { self.handle_action(UIAction::LeaveWorkspace, ctx); }
            Command::DisconnectNetwork => { self.handle_action(UIAction::DisconnectNetwork, ctx); }
            Command::ResetData => { self.reset_requested = true; }
            Command::GitStatus => { self.sync_git_status(); }
            Command::GitStageAll => {
                if let Some(ref git) = self.git_sidecar {
                    let _ = git.stage_all();
                }
                self.sync_git_status();
            }
            Command::GitCommit => {
                if self.git_sidecar.is_some() {
                    self.share_lodge_modal.title = "Commit message".to_string();
                    self.share_lodge_modal.show();
                }
            }
            Command::GitPush => {
                if let Some(ref git) = self.git_sidecar {
                    let branch = git.status().ok().map(|s| s.branch).unwrap_or_else(|| "main".to_string());
                    let _ = git.push("origin", &branch);
                }
            }
            Command::GitPull => {
                if let Some(ref git) = self.git_sidecar {
                    let _ = git.fetch("origin");
                }
            }
            Command::GitLog => {
                if let Some(ref git) = self.git_sidecar {
                    if let Ok(commits) = git.log(10) {
                        for c in &commits {
                            log::info!("GIT {}: {} by {}", &c.oid[..8], c.message.trim(), c.author);
                        }
                    }
                }
            }
        }
        self.sync_status_bar();
        self.sync_command_palette_on_close();
    }
        self.sync_status_bar();
    }

    fn handle_action(&mut self, action: UIAction, ctx: &mut AppContext) -> EventResult {
        match action {
            UIAction::Quit => { ctx.quit(); EventResult::Handled }
            UIAction::ToggleExplorer => { self.explorer_visible = !self.explorer_visible; EventResult::Handled }
            UIAction::SwitchFocus => {
                self.focused_child = match self.focused_child {
                    FocusTarget::Explorer => FocusTarget::Editor,
                    FocusTarget::Editor => FocusTarget::Explorer,
                };
                self.update_focus_state();
                EventResult::Handled
            }
            UIAction::EnterCommandMode => { self.command_palette.borrow_mut().show(); EventResult::Handled }
            UIAction::EnterSearchMode => { self.search_palette.borrow_mut().show(); EventResult::Handled }
            UIAction::ToggleMacroPalette => { self.macro_palette.borrow_mut().active = !self.macro_palette.borrow().active; EventResult::Handled }
            UIAction::StartRecordingMacro(name) => { self.macro_state = MacroState::Recording(name); self.recorded_events.clear(); EventResult::Handled }
            UIAction::StopRecordingMacro => {
                if let MacroState::Recording(name) = &self.macro_state {
                    log::info!("Macro '{}' recorded with {} events", name, self.recorded_events.len());
                }
                self.macro_state = MacroState::Idle;
                EventResult::Handled
            }
            UIAction::PlayMacro(_) => {
                let events = self.recorded_events.clone();
                self.macro_state = MacroState::Replaying;
                for ev in events {
                    let event = Event::Key(ev);
                    self.handle_event(&event, ctx);
                }
                self.macro_state = MacroState::Idle;
                EventResult::Handled
            }
            UIAction::NextTab => { self.tabs.next_tab(); self.update_focus_state(); EventResult::Handled }
            UIAction::PrevTab => { self.tabs.prev_tab(); self.update_focus_state(); EventResult::Handled }
            UIAction::NewTab => { self.tabs.new_tab(); self.update_focus_state(); EventResult::Handled }
            UIAction::EnterWindowMode => {
                self.mode = "window".to_string();
                self.sync_status_bar();
                EventResult::Handled
            }
            UIAction::ExitMode => {
                self.mode = "normal".to_string();
                self.sync_status_bar();
                EventResult::Handled
            }
            UIAction::PromptShareWorkspace => {
                self.share_lodge_modal.show();
                EventResult::Handled
            }
            UIAction::LeaveWorkspace => {
                if let Some(tx) = &self.network_tx {
                    let _ = tx.try_send(NetworkCommand::LeaveLodge { lodge_id: self.workspace.id });
                }
                self.workspace.is_shared = false;
                self.sync_status_bar();
                EventResult::Handled
            }
            UIAction::DisconnectNetwork => {
                if let Some(tx) = &self.network_tx {
                    let _ = tx.try_send(NetworkCommand::Disconnect);
                }
                self.status_bar.lodge_status = "Offline".to_string();
                self.status_bar.peer_count = 0;
                EventResult::Handled
            }
            UIAction::Copy => {
                if let Some(editor) = self.get_active_editor() {
                    let _ = editor.copy();
                }
                EventResult::Handled
            }
            UIAction::Paste => {
                let paste_result = if let Some(editor) = self.get_active_editor() {
                    editor.paste().ok().map(|c| (editor.file_path(), c))
                } else { None };

                if let Some((Some(p), c)) = paste_result {
                    self.sync_engine.apply_local_splice(&p, c.pos, c.del.try_into().unwrap(), &c.text);
                    if let Some(editor) = self.get_active_editor_ref() {
                        if let Some(doc_arc) = editor._document() {
                            let doc_content = doc_arc.lock().unwrap().rope.to_string();
                            self.notify_lsp_change(p, doc_content);
                        }
                    }
                }
                self.sync_status_bar();
                EventResult::Handled
            }
            UIAction::SplitVertical => {
                let active_id = self.tabs.active_tab_ref().layout.active_pane;
                self.tabs.active_tab_mut().layout.split_pane(active_id, ratatui::layout::Direction::Horizontal);
                self.sync_status_bar();
                EventResult::Handled
            }
            UIAction::SplitHorizontal => {
                let active_id = self.tabs.active_tab_ref().layout.active_pane;
                self.tabs.active_tab_mut().layout.split_pane(active_id, ratatui::layout::Direction::Vertical);
                self.sync_status_bar();
                EventResult::Handled
            }
            UIAction::ClosePane => {
                let active_id = self.tabs.active_tab_ref().layout.active_pane;
                self.tabs.active_tab_mut().layout.close_pane(active_id);
                self.sync_status_bar();
                EventResult::Handled
            }
            UIAction::FocusLeft => { self.tabs.active_tab_mut().layout.navigate(crate::components::layout::NavDir::Left); self.sync_status_bar(); EventResult::Handled }
            UIAction::FocusRight => { self.tabs.active_tab_mut().layout.navigate(crate::components::layout::NavDir::Right); self.sync_status_bar(); EventResult::Handled }
            UIAction::FocusUp => { self.tabs.active_tab_mut().layout.navigate(crate::components::layout::NavDir::Up); self.sync_status_bar(); EventResult::Handled }
            UIAction::FocusDown => { self.tabs.active_tab_mut().layout.navigate(crate::components::layout::NavDir::Down); self.sync_status_bar(); EventResult::Handled }
            _ => EventResult::Unhandled,
        }
    }

    pub fn take_share_request(&mut self) -> Option<String> {
        self.share_lodge_modal.take_value()
    }

    pub fn take_config_request(&mut self) -> bool {
        let req = self.config_requested;
        self.config_requested = false;
        req
    }
}

impl Component for EditorView {
    fn draw(&self, frame: &mut Frame, area: Rect, ctx: &DrawContext) {
        *self.last_area.borrow_mut() = area;
        let chunks = Layout::vertical([
            Constraint::Length(1), // Tab bar
            Constraint::Min(0),    // Workspace
            Constraint::Length(2), // Status bar + Hints
        ]).split(area);

        self.tab_bar.render(frame, chunks[0], &self.tabs);
        self.status_bar.draw(frame, chunks[2], ctx);

        let workspace_chunks = if self.explorer_visible {
            Layout::horizontal([
                Constraint::Percentage(20),
                Constraint::Min(0),
            ]).split(chunks[1])
        } else {
            Layout::horizontal([
                Constraint::Percentage(0),
                Constraint::Min(0),
            ]).split(chunks[1])
        };

        if self.explorer_visible {
            self.file_tree.draw(frame, workspace_chunks[0], ctx);
        }

        let editor_area = workspace_chunks[1];
        self.tabs.draw(frame, editor_area, ctx);

        self.command_palette.borrow_mut().render(frame, area);
        self.search_palette.borrow_mut().render(frame, area);
        self.macro_palette.borrow_mut().render(frame, area);
        if self.share_lodge_modal.active { self.share_lodge_modal.draw(frame, area, ctx); }
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut AppContext) -> EventResult {
        if let Event::Key(key) = event {
            log::info!("[VIEW] Trace: code={:?}, mods={:?}, kind={:?}", key.code, key.modifiers, key.kind);
        }

        if let Some(ref mut rx) = self.search_rx {
            while let Ok(sr) = rx.try_recv() {
                self.search_palette.borrow_mut().add_result(sr);
            }
        }

        if self.share_lodge_modal.active {
            let result = self.share_lodge_modal.handle_event(event, ctx);
            if !self.share_lodge_modal.active && self.share_lodge_modal.title == "Commit message" && !self.share_lodge_modal.value.is_empty() {
                self.pending_git_commit = Some(self.share_lodge_modal.value.clone());
                self.share_lodge_modal.value.clear();
            }
            return result;
        }

        if let Some(msg) = self.pending_git_commit.take() {
            if let Some(ref git) = self.git_sidecar {
                let _ = git.stage_all();
                let _ = git.commit(&msg);
                log::info!("Git commit: {}", msg);
            }
        }

        let cmd_action = if self.command_palette.borrow().active {
            let res = self.command_palette.borrow_mut().handle_event(event, ctx);
            let cmd = self.command_palette.borrow_mut().take_selected();
            if res.is_handled() || cmd.is_some() { Some((res, cmd)) } else { None }
        } else { None };

        if let Some((res, cmd)) = cmd_action {
            if let Some(c) = cmd { self.execute_command(c, ctx); }
            if res.is_handled() { return res; }
        }

        let search_action = if self.search_palette.borrow().active {
            let res = self.search_palette.borrow_mut().handle_event(event, ctx);
            let req = self.search_palette.borrow_mut().take_search_request();
            let sel = self.search_palette.borrow_mut().take_selected();
            if res.is_handled() || req || sel.is_some() { Some((res, req, sel)) } else { None }
        } else { None };

        if let Some((res, req, sel)) = search_action {
            if req { self.perform_project_search(); }
            if let Some(sr) = sel {
                let _ = self.load_file(&sr.file);
                self.focused_child = FocusTarget::Editor;
                self.update_focus_state();
            }
            if res.is_handled() { self.sync_status_bar(); return res; }
        }

        let macro_palette_action = if self.macro_palette.borrow().active {
            let res = self.macro_palette.borrow_mut().handle_event(event, ctx);
            let action = self.macro_palette.borrow_mut().take_action();
            if res.is_handled() || action.is_some() { Some((res, action)) } else { None }
        } else { None };

        if let Some((res, action)) = macro_palette_action {
            if let Some(a) = action {
                match a {
                    crate::components::modals::macro_palette::MacroAction::RecordNew => {
                        let _ = self.handle_action(UIAction::StartRecordingMacro("new_macro".to_string()), ctx);
                    }
                    crate::components::modals::macro_palette::MacroAction::Play(name) => {
                        let _ = self.handle_action(UIAction::PlayMacro(name), ctx);
                    }
                }
            }
            if res.is_handled() { return res; }
        }

        if let Event::Mouse(mouse) = event {
            if mouse.kind == crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left) {
                let area = *self.last_area.borrow();
                let chunks = Layout::vertical([
                    Constraint::Length(1),
                    Constraint::Min(0),
                    Constraint::Length(2),
                ]).split(area);
                let workspace_chunks = if self.explorer_visible {
                    Layout::horizontal([Constraint::Percentage(20), Constraint::Min(0)]).split(chunks[1])
                } else {
                    Layout::horizontal([Constraint::Percentage(0), Constraint::Min(0)]).split(chunks[1])
                };
                
                if self.explorer_visible && workspace_chunks[0].contains(ratatui::layout::Position::new(mouse.column, mouse.row)) {
                    self.focused_child = FocusTarget::Explorer;
                } else if workspace_chunks[1].contains(ratatui::layout::Position::new(mouse.column, mouse.row)) {
                    self.focused_child = FocusTarget::Editor;
                }
                self.update_focus_state();
            }
        }

        if let MacroState::Recording(_) = &self.macro_state {
            if let Event::Key(key) = event {
                if key.kind == KeyEventKind::Press {
                    if key.code == KeyCode::Char('m') && key.modifiers.contains(KeyModifiers::CONTROL) { return self.handle_action(UIAction::StopRecordingMacro, ctx); }
                    self.recorded_events.push(*key);
                }
            }
        }

        match self.focused_child {
            FocusTarget::Explorer => {
                let res = self.file_tree.handle_event(event, ctx);
                if let Some(path) = self.file_tree.take_file_to_open() {
                    let _ = self.load_file(&path);
                    self.focused_child = FocusTarget::Editor;
                    self.update_focus_state();
                }
                return res;
            }
            FocusTarget::Editor => {
                if let Event::Key(key_event) = event {
                    let action = self.key_mapper.resolve(*key_event, &self.mode, Some("editor"));
                    if let Some(a) = action {
                        return self.handle_action(a, ctx);
                    }

                    let mut editor_data = None;
                    if let Some(editor) = self.get_active_editor() {
                        let (res, change) = editor.handle_key_event(key_event);
                        if res.is_handled() {
                            let path_info = if let Some(c) = change {
                                editor.file_path().map(|p| (p.to_path_buf(), c))
                            } else { None };
                            editor_data = Some((res, path_info, editor.cursor_offset));
                        }
                    }

                    if let Some((res, path_info, offset)) = editor_data {
                        if let Some((p, c)) = path_info {
                    self.sync_engine.apply_local_splice(&p, c.pos, c.del as isize, &c.text);
                            if let Some(editor) = self.get_active_editor_ref() {
                                if let Some(doc_arc) = editor._document() {
                                    let doc_content = doc_arc.lock().unwrap().rope.to_string();
                                    self.notify_lsp_change(p.clone(), doc_content);
                                }
                            }
                            self.sync_engine.update_cursor(&p, &self.status_bar.username, offset);
                        } else if let Some(editor) = self.get_active_editor_ref() {
                            if let Some(p) = editor.file_path() {
                                self.sync_engine.update_cursor(&p.to_path_buf(), &self.status_bar.username, editor.cursor_offset);
                            }
                        }
                        self.sync_status_bar();
                        return res;
                    }
                }
                
                let res = self.tabs.handle_event(event, ctx);
                self.sync_status_bar();
                res
            }
        }
    }
}


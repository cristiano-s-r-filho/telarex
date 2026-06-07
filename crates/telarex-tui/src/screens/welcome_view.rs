use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout, Rect, Alignment},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Padding},
    Frame,
};
use crate::tui_compat::{AppContext, Component, DrawContext, Event, EventResult};
use std::cell::RefCell;
use uuid::Uuid;
use crate::theme::Theme;
use crate::components::modals::InputModal;
use crate::utils::sanitize;
use rand::seq::SliceRandom;

pub struct DiscoveredLodge {
    pub id: Uuid,
    pub name: String,
    pub peer_id: String,
}

pub struct WelcomeView {
    pub theme: Theme,
    open_editor_with_path: Option<String>,
    open_config: bool,
    join_lodge_requested: Option<Uuid>,
    pub delete_lodge_requested: Option<Uuid>,
    input_modal: InputModal,
    join_by_id_modal: InputModal,
    pub recent_projects_modal_active: bool,
    pub discovered_lodges_modal_active: bool,
    recent_projects_state: RefCell<ListState>,
    discovered_lodges_state: RefCell<ListState>,
    pub discovered_lodges: Vec<DiscoveredLodge>,
    pub recent_projects: Vec<String>,
    pub username: String,
    pub last_opened: Option<String>,
    selected_menu_item: usize,
    subtitle: String,
    lodge_search_query: String,
}

impl WelcomeView {
    pub fn new(_session: Option<String>) -> Self {
        let subtitles = vec![
            "Where code meets the void.",
            "Collaborate or die (just kidding, please collaborate).",
            "Post-Quantum, Pre-Coffee.",
            "The only terminal you'll ever need. Maybe.",
            "Built for speed, hardened for war.",
            "Your nodes are talking. Listen closely.",
            "Hyper-speed editing for technical minds.",
            "Decentralized. Hardened. Sovereign.",
        ];
        let mut rng = rand::thread_rng();
        let subtitle = subtitles.choose(&mut rng).unwrap_or(&"Collaborative technical workspace.").to_string();

        Self {
            theme: Theme::default(),
            open_editor_with_path: None,
            open_config: false,
            join_lodge_requested: None,
            delete_lodge_requested: None,
            input_modal: InputModal::new("Enter Path"),
            join_by_id_modal: InputModal::new("Enter Lodge UUID"),
            recent_projects_modal_active: false,
            discovered_lodges_modal_active: false,
            recent_projects_state: RefCell::new(ListState::default()),
            discovered_lodges_state: RefCell::new(ListState::default()),
            discovered_lodges: Vec::new(),
            recent_projects: Vec::new(),
            username: "User".to_string(),
            last_opened: None,
            selected_menu_item: 0,
            subtitle,
            lodge_search_query: String::new(),
        }
    }

    pub fn take_join_request(&mut self) -> Option<Uuid> { self.join_lodge_requested.take() }
    pub fn take_delete_request(&mut self) -> Option<Uuid> { self.delete_lodge_requested.take() }
    pub fn take_open_request(&mut self) -> Option<String> { self.open_editor_with_path.take() }
    pub fn take_config_request(&mut self) -> bool {
        let req = self.open_config;
        self.open_config = false;
        req
    }

    fn menu_items(&self) -> Vec<(&'static str, &'static str, Color)> {
        vec![
            ("N", "New Project", self.theme.success),
            ("O", "Open Folder", self.theme.info),
            ("R", "Recent Files", self.theme.warning),
            ("L", "Join Lodge", self.theme.accent),
            ("J", "Join by ID", self.theme.info),
            ("C", "Settings", Color::Gray),
            ("Q", "Quit", self.theme.error),
        ]
    }

    fn handle_menu_key(&mut self, key: &str) {
        match key.to_uppercase().as_str() {
            "N" | "O" => self.input_modal.show(),
            "R" => {
                if !self.recent_projects.is_empty() {
                    self.recent_projects_state.borrow_mut().select(Some(0));
                    self.recent_projects_modal_active = true;
                }
            }
            "L" => {
                self.discovered_lodges_state.borrow_mut().select(Some(0));
                self.discovered_lodges_modal_active = true;
            }
            "J" => self.join_by_id_modal.show(),
            "C" => self.open_config = true,
            _ => {}
        }
    }

    fn filtered_lodges(&self) -> Vec<&DiscoveredLodge> {
        if self.lodge_search_query.is_empty() {
            self.discovered_lodges.iter().collect()
        } else {
            let q = self.lodge_search_query.to_lowercase();
            self.discovered_lodges.iter()
                .filter(|l| l.name.to_lowercase().contains(&q) || l.id.to_string().contains(&q))
                .collect()
        }
    }
}

impl Component for WelcomeView {
    fn draw(&self, frame: &mut Frame, area: Rect, _ctx: &DrawContext) {
        frame.render_widget(Clear, area);
        
        let chunks = Layout::vertical([
            Constraint::Length(8), // ASCII Banner
            Constraint::Length(1), // Subtitle
            Constraint::Min(0),    // Main Content
            Constraint::Length(1), // Footer
        ]).split(area);

        // 1. ASCII Banner (High Blocky)
        let banner = vec![
            r"████████╗███████╗██╗      █████╗ ██████╗ ███████╗██╗  ██╗",
            r"╚══██╔══╝██╔════╝██║     ██╔══██╗██╔══██╗██╔════╝╚██╗██╔╝",
            r"   ██║   █████╗  ██║     ███████║██████╔╝█████╗   ╚███╔╝ ",
            r"   ██║   ██╔══╝  ██║     ██╔══██║██╔══██╗██╔══╝   ██╔██╗ ",
            r"   ██║   ███████╗███████╗██║  ██║██║  ██║███████╗██╔╝ ██╗",
            r"   ╚═╝   ╚══════╝╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝",
        ];
        let banner_paragraph = Paragraph::new(banner.iter().map(|s| Line::from(Span::styled(*s, self.theme.header))).collect::<Vec<_>>())
            .alignment(Alignment::Center);
        frame.render_widget(banner_paragraph, chunks[0]);

        // 2. Subtitle
        let subtitle = Paragraph::new(format!("-- {} --", self.subtitle))
            .alignment(Alignment::Center)
            .style(self.theme.subtitle);
        frame.render_widget(subtitle, chunks[1]);

        // 3. Main Content (Options List | Status Box)
        let main_content = Layout::horizontal([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ]).split(chunks[2]);

        // Options List
        let mut menu_lines = Vec::new();
        for (i, (key, label, color)) in self.menu_items().iter().enumerate() {
            let is_selected = i == self.selected_menu_item && !self.recent_projects_modal_active && !self.discovered_lodges_modal_active;
            let style = if is_selected {
                self.theme.list_selected
            } else {
                Style::default()
            };
            let symbol = match *key {
                "N" => "◆",
                "O" => "▸",
                "R" => "○",
                "L" => "●",
                "J" => "◉",
                "C" => "◇",
                "Q" => "■",
                _ => " ",
            };
            menu_lines.push(ListItem::new(Line::from(vec![
                Span::styled(format!(" {}  ", symbol), Style::default().fg(*color).add_modifier(Modifier::BOLD)),
                Span::raw(*label),
            ])).style(style));
        }
        let options_list = List::new(menu_lines)
            .block(Block::default().borders(Borders::ALL).padding(Padding::uniform(1)))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));
        frame.render_widget(options_list, main_content[0]);

        // Status Box
        let status_lines = vec![
            Line::from(vec![
                Span::styled(" ◆ SESSION DATA ", self.theme.header),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(" User: ", self.theme.status_label),
                Span::styled(&self.username, self.theme.status_value),
            ]),
            Line::from(vec![
                Span::styled(" Last opened: ", self.theme.status_label),
                Span::styled(self.last_opened.as_deref().unwrap_or("None"), self.theme.status_value),
            ]),
            Line::from(vec![
                Span::styled(" Active Lodges: ", self.theme.status_label),
                Span::styled(format!("{} active", self.discovered_lodges.len()), self.theme.status_value),
            ]),
            Line::from(vec![
                Span::styled(" Recent Projects: ", self.theme.status_label),
                Span::styled(self.recent_projects.len().to_string(), self.theme.status_value),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(" Status: ", self.theme.status_label),
                Span::styled(" READY ", Style::default().bg(self.theme.success).fg(Color::Black).add_modifier(Modifier::BOLD)),
            ]),
        ];
        let status_box = Paragraph::new(status_lines)
            .block(Block::default().borders(Borders::ALL).padding(Padding::uniform(1)));
        frame.render_widget(status_box, main_content[1]);

        // 4. Footer
        let footer = Paragraph::new(" ◆ [↑/↓] Navigate  [Enter] Select  [key] Shortcut  [Ctrl+Q] Exit ◆ ")
            .alignment(Alignment::Center)
            .style(self.theme.footer_hint);
        frame.render_widget(footer, chunks[3]);

        // Modals (Overlays)
        if self.recent_projects_modal_active {
            let modal_area = crate::utils::centered_rect_fixed(80, 20, area);
            frame.render_widget(Clear, modal_area);
            let items: Vec<ListItem> = self.recent_projects.iter()
                .map(|p| ListItem::new(format!(" {} ", sanitize(p))))
                .collect();
            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title(" Recent Projects ").border_style(Style::default().fg(self.theme.warning)))
                .highlight_style(self.theme.list_selected);
            frame.render_stateful_widget(list, modal_area, &mut self.recent_projects_state.borrow_mut());
        }

        if self.discovered_lodges_modal_active {
            let modal_area = crate::utils::centered_rect_fixed(70, 18, area);
            frame.render_widget(Clear, modal_area);

            let inner = Layout::vertical([
                Constraint::Length(3),
                Constraint::Min(0),
            ]).split(modal_area);

            // Search bar
            let search_text = if self.lodge_search_query.is_empty() {
                " Type to search lodges...".to_string()
            } else {
                format!(" Search: {}", self.lodge_search_query)
            };
            let search_style = if self.lodge_search_query.is_empty() {
                self.theme.list_inactive
            } else {
                Style::default().fg(self.theme.fg)
            };
            frame.render_widget(
                Paragraph::new(search_text).style(search_style)
                    .block(Block::default().borders(Borders::ALL).title(" Filter ").border_style(Style::default().fg(self.theme.border_active))),
                inner[0],
            );

            let filtered = self.filtered_lodges();
            let items: Vec<ListItem> = filtered.iter()
                .map(|l| {
                    let peer_short = if l.peer_id.len() > 12 { format!("{}…", &l.peer_id[..12]) } else { l.peer_id.clone() };
                    ListItem::new(format!(" {} [{}] ", sanitize(&l.name), peer_short))
                })
                .collect();
            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title(" Discovered Lodges ").border_style(Style::default().fg(self.theme.accent)))
                .highlight_symbol("▶ ")
                .highlight_style(self.theme.list_selected);
            frame.render_stateful_widget(list, inner[1], &mut self.discovered_lodges_state.borrow_mut());
        }

        self.input_modal.draw(frame, area, _ctx);
        self.join_by_id_modal.draw(frame, area, _ctx);
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut AppContext) -> EventResult {
        if self.input_modal.modal.active {
            let res = self.input_modal.handle_event(event, ctx);
            if let Some(path) = self.input_modal.take_value() { self.open_editor_with_path = Some(path); }
            return res;
        }

        if self.join_by_id_modal.modal.active {
            let res = self.join_by_id_modal.handle_event(event, ctx);
            if let Some(uuid_str) = self.join_by_id_modal.take_value() {
                if let Ok(id) = Uuid::parse_str(&uuid_str) { self.join_lodge_requested = Some(id); }
            }
            return res;
        }

        if let Event::Key(key) = event {
            if key.kind != KeyEventKind::Press { return EventResult::Handled; }
            match key.code {
                KeyCode::Esc => {
                    self.recent_projects_modal_active = false;
                    self.discovered_lodges_modal_active = false;
                    self.lodge_search_query.clear();
                    EventResult::Handled
                }
                KeyCode::Down => {
                    if self.recent_projects_modal_active {
                        let mut state = self.recent_projects_state.borrow_mut();
                        let i = match state.selected() { Some(i) => (i + 1) % self.recent_projects.len(), None => 0 };
                        state.select(Some(i));
                    } else if self.discovered_lodges_modal_active {
                        let mut state = self.discovered_lodges_state.borrow_mut();
                        let filtered = self.filtered_lodges();
                        let i = match state.selected() {
                            Some(i) => if filtered.is_empty() { None } else { Some((i + 1) % filtered.len()) },
                            None => if filtered.is_empty() { None } else { Some(0) },
                        };
                        state.select(i);
                    } else {
                        self.selected_menu_item = (self.selected_menu_item + 1) % self.menu_items().len();
                    }
                    EventResult::Handled
                }
                KeyCode::Up => {
                    if self.recent_projects_modal_active {
                        let mut state = self.recent_projects_state.borrow_mut();
                        let i = match state.selected() { Some(i) => if i == 0 { self.recent_projects.len() - 1 } else { i - 1 }, None => 0 };
                        state.select(Some(i));
                    } else if self.discovered_lodges_modal_active {
                        let mut state = self.discovered_lodges_state.borrow_mut();
                        let filtered = self.filtered_lodges();
                        let i = match state.selected() {
                            Some(i) => if i == 0 { filtered.len().saturating_sub(1) } else { i - 1 },
                            None => filtered.len().saturating_sub(1),
                        };
                        state.select(if filtered.is_empty() { None } else { Some(i) });
                    } else {
                        self.selected_menu_item = if self.selected_menu_item == 0 { self.menu_items().len() - 1 } else { self.selected_menu_item - 1 };
                    }
                    EventResult::Handled
                }
                KeyCode::Enter => {
                    if self.recent_projects_modal_active {
                        let state = self.recent_projects_state.borrow();
                        if let Some(i) = state.selected() {
                            self.open_editor_with_path = Some(self.recent_projects[i].clone());
                            self.recent_projects_modal_active = false;
                        }
                    } else if self.discovered_lodges_modal_active {
                        let state = self.discovered_lodges_state.borrow();
                        if let Some(i) = state.selected() {
                            let filtered = self.filtered_lodges();
                            if let Some(lodge) = filtered.get(i) {
                                self.join_lodge_requested = Some(lodge.id);
                                self.discovered_lodges_modal_active = false;
                                self.lodge_search_query.clear();
                            }
                        }
                    } else {
                        let key = self.menu_items()[self.selected_menu_item].0;
                        self.handle_menu_key(key);
                    }
                    EventResult::Handled
                }
                KeyCode::Char(c) => {
                    if self.discovered_lodges_modal_active {
                        self.lodge_search_query.push(c);
                        let filtered = self.filtered_lodges();
                        self.discovered_lodges_state.borrow_mut().select(if filtered.is_empty() { None } else { Some(0) });
                    } else {
                        self.handle_menu_key(&c.to_string());
                    }
                    EventResult::Handled
                }
                KeyCode::Backspace => {
                    if self.discovered_lodges_modal_active {
                        self.lodge_search_query.pop();
                        let filtered = self.filtered_lodges();
                        self.discovered_lodges_state.borrow_mut().select(if filtered.is_empty() { None } else { Some(0) });
                        return EventResult::Handled;
                    }
                    EventResult::Unhandled
                }
                _ => EventResult::Unhandled,
            }
        } else { EventResult::Unhandled }
    }
}

//! Configuration screen — editor, appearance, network, and keymap settings.
use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Rect, Layout, Constraint},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, List, ListItem, Padding},
    Frame,
};
use ratatui::prelude::Stylize;
use crate::tui_compat::{AppContext, Component, DrawContext, Event, EventResult};
use telarex_core::config::{self, TelaRexConfig, ThemeEngine};

use crate::theme::Theme;
use crate::utils::sanitize;

use crate::components::modals::InputModal;

/// Categories of configuration settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigCategory {
    Editor,
    Appearance,
    Network,
    Keymaps,
}

impl ConfigCategory {
    fn all() -> &'static [ConfigCategory] {
        &[Self::Editor, Self::Appearance, Self::Network, Self::Keymaps]
    }
    fn name(&self) -> &'static str {
        match self {
            Self::Editor => "Editor",
            Self::Appearance => "Appearance",
            Self::Network => "Network",
            Self::Keymaps => "Keymaps",
        }
    }
    fn icon(&self) -> &'static str {
        match self {
            Self::Editor => "◆",
            Self::Appearance => "◇",
            Self::Network => "●",
            Self::Keymaps => "◈",
        }
    }
}

/// Full-screen configuration editor — browse and modify all settings.
pub struct ConfigModal {
    pub active: bool,
    pub should_exit: bool,
    config: TelaRexConfig,
    modified: bool,
    selected_category: usize,
    selected_field: usize,
    pub theme: Theme,
    input_modal: InputModal,
    session_id: Option<String>,
    theme_engine: ThemeEngine,
    available_themes: Vec<String>,
    focus_on_fields: bool,
}

impl ConfigModal {
    pub fn new(session_id: Option<String>) -> Self {
        let config = config::load(session_id.as_deref()).unwrap_or_default();
        let mut theme_engine = ThemeEngine::new();
        let _ = theme_engine.load_themes("themes");
        let available_themes = theme_engine.list_themes();
        Self {
            active: false,
            should_exit: false,
            config,
            modified: false,
            selected_category: 0,
            selected_field: 0,
            theme: Theme::default(),
            input_modal: InputModal::new("Change Value"),
            session_id,
            theme_engine,
            available_themes,
            focus_on_fields: false,
        }
    }

    pub fn apply_theme(&mut self, theme: &Theme) {
        self.theme = theme.clone();
    }

    pub fn show(&mut self) {
        self.active = true;
        self.config = config::load(self.session_id.as_deref()).unwrap_or_default();
        self.modified = false;
        self.selected_category = 0;
        self.selected_field = 0;
        self.focus_on_fields = false;
        self.available_themes = self.theme_engine.list_themes();
    }

    pub fn hide(&mut self) {
        self.active = false;
        self.should_exit = true;
    }

    pub fn get_config(&self) -> &TelaRexConfig {
        &self.config
    }

    fn save(&mut self) {
        if let Err(_) = config::save(&self.config, self.session_id.as_deref()) {}
        else { self.modified = false; }
    }

    fn current_category(&self) -> ConfigCategory {
        ConfigCategory::all()[self.selected_category]
    }

    fn fields_count(&self) -> usize {
        match self.current_category() {
            ConfigCategory::Editor => 6,
            ConfigCategory::Appearance => 1,
            ConfigCategory::Network => 2,
            ConfigCategory::Keymaps => 0,
        }
    }

    fn next_item(&mut self) {
        if self.focus_on_fields {
            let count = self.fields_count();
            if count > 0 { self.selected_field = (self.selected_field + 1) % count; }
        } else {
            self.selected_category = (self.selected_category + 1) % ConfigCategory::all().len();
            self.selected_field = 0;
        }
    }

    fn prev_item(&mut self) {
        if self.focus_on_fields {
            let count = self.fields_count();
            if count > 0 {
                if self.selected_field == 0 { self.selected_field = count - 1; }
                else { self.selected_field -= 1; }
            }
        } else {
            if self.selected_category == 0 { self.selected_category = ConfigCategory::all().len() - 1; }
            else { self.selected_category -= 1; }
            self.selected_field = 0;
        }
    }

    fn cycle_theme(&mut self, forward: bool) {
        if self.available_themes.is_empty() { return; }
        let current = self.available_themes.iter().position(|t| t == &self.config.editor.theme).unwrap_or(0);
        let next = if forward { (current + 1) % self.available_themes.len() } else { if current == 0 { self.available_themes.len() - 1 } else { current - 1 } };
        self.config.editor.theme = self.available_themes[next].clone();
        let _ = self.theme_engine.set_theme(&self.config.editor.theme);
        self.theme = Theme::from_stylesheet(self.theme_engine.get_current());
        self.modified = true;
    }

    fn toggle_vim_mode(&mut self) { self.config.editor.vim_mode = !self.config.editor.vim_mode; self.modified = true; }
    fn toggle_line_numbers(&mut self) { self.config.editor.line_numbers = !self.config.editor.line_numbers; self.modified = true; }
    fn toggle_auto_save(&mut self) { self.config.editor.auto_save = !self.config.editor.auto_save; self.modified = true; }
    fn toggle_wrap(&mut self) { self.config.editor.wrap_text = !self.config.editor.wrap_text; self.modified = true; }

    fn adjust_tab_size(&mut self, inc: bool) {
        if inc { self.config.editor.tab_size = self.config.editor.tab_size.saturating_add(2).min(8); }
        else { self.config.editor.tab_size = self.config.editor.tab_size.saturating_sub(2).max(2); }
        self.modified = true;
    }
}

impl Component for ConfigModal {
    fn handle_event(&mut self, event: &Event, ctx: &mut AppContext) -> EventResult {
        if !self.active { return EventResult::Unhandled; }

        if self.input_modal.modal.active {
            let res = self.input_modal.handle_event(event, ctx);
            if let Some(new_val) = self.input_modal.take_value() {
                if !new_val.is_empty() {
                    match (self.current_category(), self.selected_field) {
                        (ConfigCategory::Network, 0) => { self.config.profile.username = sanitize(&new_val); }
                        (ConfigCategory::Network, 1) => { self.config.network.bootstrap_node = sanitize(&new_val); }
                        _ => {}
                    }
                    self.modified = true;
                }
            }
            return res;
        }

        if let Event::Key(key_event) = event {
            if key_event.kind != KeyEventKind::Press { return EventResult::Handled; }

            match key_event.code {
                KeyCode::Esc => { self.hide(); return EventResult::Handled; }
                KeyCode::Char('s') if key_event.modifiers == KeyModifiers::CONTROL => { self.save(); if self.active { self.hide(); } return EventResult::Handled; }
                KeyCode::Tab | KeyCode::Right if !self.focus_on_fields => { if self.fields_count() > 0 { self.focus_on_fields = true; } return EventResult::Handled; }
                KeyCode::Left if self.focus_on_fields => { self.focus_on_fields = false; return EventResult::Handled; }
                KeyCode::Down | KeyCode::Char('j') => { self.next_item(); return EventResult::Handled; }
                KeyCode::Up | KeyCode::Char('k') => { self.prev_item(); return EventResult::Handled; }
                KeyCode::Enter => {
                    if !self.focus_on_fields { if self.fields_count() > 0 { self.focus_on_fields = true; } }
                    else {
                        match (self.current_category(), self.selected_field) {
                            (ConfigCategory::Editor, 1) => self.toggle_vim_mode(),
                            (ConfigCategory::Editor, 2) => self.toggle_line_numbers(),
                            (ConfigCategory::Editor, 3) => self.toggle_auto_save(),
                            (ConfigCategory::Editor, 4) => self.toggle_wrap(),
                            (ConfigCategory::Network, 0) => { self.input_modal.modal.title = " Username ".to_string(); self.input_modal.show(); }
                            (ConfigCategory::Network, 1) => { self.input_modal.modal.title = " Bootstrap Node ".to_string(); self.input_modal.show(); }
                            _ => {}
                        }
                    }
                    return EventResult::Handled;
                }
                KeyCode::Right | KeyCode::Char('l') if self.focus_on_fields => {
                    match (self.current_category(), self.selected_field) {
                        (ConfigCategory::Editor, 0) => self.adjust_tab_size(true),
                        (ConfigCategory::Appearance, 0) => self.cycle_theme(true),
                        _ => {}
                    }
                    return EventResult::Handled;
                }
                KeyCode::Left | KeyCode::Char('h') if self.focus_on_fields => {
                    match (self.current_category(), self.selected_field) {
                        (ConfigCategory::Editor, 0) => self.adjust_tab_size(false),
                        (ConfigCategory::Appearance, 0) => self.cycle_theme(false),
                        _ => {}
                    }
                    return EventResult::Handled;
                }
                _ => {}
            }
        }
        EventResult::Handled
    }

    fn draw(&self, frame: &mut Frame, area: Rect, ctx: &DrawContext) {
        if !self.active { return; }

        // Full-screen layout
        let chunks = Layout::vertical([
            Constraint::Length(3), // Header bar
            Constraint::Min(0),    // Body
            Constraint::Length(1), // Footer bar
        ]).split(area);

        // ── Header ──
        let save_indicator = if self.modified { "  ● unsaved" } else { "" };
        let header_line = Line::from(vec![
            Span::styled("  ◆  Settings", self.theme.header.add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::styled(format!("[{}]{}", self.current_category().name(), save_indicator), Style::default().fg(self.theme.fg_dim)),
        ]);
        frame.render_widget(
            Paragraph::new(header_line)
                .block(Block::default().bg(self.theme.surface).borders(Borders::BOTTOM).border_style(Style::default().fg(self.theme.border_inactive))),
            chunks[0],
        );

        // ── Body: sidebar | content ──
        let body_chunks = Layout::horizontal([
            Constraint::Length(20),
            Constraint::Min(0),
        ]).split(chunks[1]);

        // Sidebar
        let mut categories = Vec::new();
        for (i, cat) in ConfigCategory::all().iter().enumerate() {
            let is_selected = i == self.selected_category;
            let style = if is_selected {
                if !self.focus_on_fields { self.theme.list_selected }
                else { Style::default().fg(self.theme.border_active).add_modifier(Modifier::BOLD) }
            } else { Style::default().fg(self.theme.fg) };
            categories.push(ListItem::new(Line::from(vec![
                Span::styled(format!(" {}  ", cat.icon()), Style::default().fg(self.theme.accent)),
                Span::styled(sanitize(cat.name()), style),
            ])).style(style));
        }
        let sidebar = List::new(categories)
            .block(Block::default().borders(Borders::RIGHT).border_style(Style::default().fg(self.theme.border_inactive)).bg(self.theme.surface));
        frame.render_widget(sidebar, body_chunks[0]);

        // Main content area
        let content_area = body_chunks[1];
        let inner = Block::default().padding(Padding::uniform(1));
        let inner_area = inner.inner(content_area);
        frame.render_widget(inner, content_area);

        let mut fields: Vec<(&str, String, &str)> = Vec::new();
        match self.current_category() {
            ConfigCategory::Editor => {
                fields.push(("Tab Size", format!("{}", self.config.editor.tab_size), "use ←/→ to adjust"));
                fields.push(("Vim Mode", if self.config.editor.vim_mode { "✓ on".to_string() } else { "✗ off".to_string() }, "Enter to toggle"));
                fields.push(("Line Numbers", if self.config.editor.line_numbers { "✓ on".to_string() } else { "✗ off".to_string() }, "Enter to toggle"));
                fields.push(("Auto-save", if self.config.editor.auto_save { "✓ on".to_string() } else { "✗ off".to_string() }, "Enter to toggle"));
                fields.push(("Wrap Text", if self.config.editor.wrap_text { "✓ on".to_string() } else { "✗ off".to_string() }, "Enter to toggle"));
            }
            ConfigCategory::Appearance => {
                fields.push(("Theme", self.config.editor.theme.clone(), "use ←/→ to cycle"));
            }
            ConfigCategory::Network => {
                fields.push(("Username", self.config.profile.username.clone(), "Enter to edit"));
                fields.push(("Bootstrap", self.config.network.bootstrap_node.clone(), "Enter to edit"));
            }
            ConfigCategory::Keymaps => {
                let keymap_lines = vec![
                    ("Ctrl+P", "Command Palette"),
                    ("Ctrl+F", "Search"),
                    ("Ctrl+M", "Macro Palette"),
                    ("Ctrl+T", "New Tab"),
                    ("Ctrl+PageDown", "Next Tab"),
                    ("Ctrl+PageUp", "Prev Tab"),
                    ("Ctrl+W", "Window Mode"),
                    ("Ctrl+Shift+W", "Close Tab"),
                    ("Ctrl+E", "Switch Focus"),
                    ("Ctrl+B", "Toggle Explorer"),
                    ("Ctrl+L", "Toggle Lodge ID"),
                    ("Ctrl+Y", "Toggle Lodge Format"),
                    ("Ctrl+S", "Save"),
                    ("Ctrl+C", "Copy"),
                    ("Ctrl+V", "Paste"),
                    ("Ctrl+G", "Git Status"),
                    ("Ctrl+Q", "Quit"),
                ];
                return frame.render_widget(
                    Paragraph::new(keymap_lines.iter().map(|(k, v)| {
                        Line::from(vec![
                            Span::styled(format!("  {:18}", k), self.theme.status_label),
                            Span::styled(format!(" {}", v), Style::default().fg(self.theme.fg)),
                        ])
                    }).collect::<Vec<_>>())
                    .block(Block::default().title(" Keybinds ").borders(Borders::ALL).border_style(Style::default().fg(self.theme.border_inactive))),
                    inner_area,
                );
            }
        }

        let mut field_widgets = Vec::new();
        for (i, (label, value, hint)) in fields.iter().enumerate() {
            let is_selected = self.focus_on_fields && i == self.selected_field;
            let row_bg = if is_selected { self.theme.selection_bg } else { self.theme.bg };
            let row_style = if is_selected { self.theme.list_selected } else { Style::default().fg(self.theme.fg) };

            let value_style = if label == &"Vim Mode" || label == &"Line Numbers"
                || label == &"Auto-save" || label == &"Wrap Text"
            {
                if value.starts_with("✓") { Style::default().fg(self.theme.success).add_modifier(Modifier::BOLD) }
                else { Style::default().fg(self.theme.fg_dim) }
            } else { Style::default().fg(self.theme.fg) };

            field_widgets.push(
                Paragraph::new(Line::from(vec![
                    Span::styled(format!("  {:<16}", sanitize(label)), self.theme.status_label),
                    Span::styled(format!(" {}", sanitize(value)), value_style),
                    Span::styled(format!("  {}", hint), Style::default().fg(self.theme.fg_dim)),
                ]))
                .style(row_style)
                .block(Block::default().bg(row_bg))
            );
            // Spacing
            field_widgets.push(Paragraph::new(Line::from(vec![Span::raw("")])));
        }
        for w in field_widgets {
            frame.render_widget(w, inner_area);
        }

        // ── Footer ──
        let footer = Line::from(vec![
            Span::styled("  [↑/↓] navigate  ", self.theme.footer_hint),
            Span::styled("[Tab/Enter] focus field  ", self.theme.footer_hint),
            Span::styled("[Esc] back  ", self.theme.footer_hint),
            Span::styled("[Ctrl+S] save & exit", self.theme.footer_hint),
        ]);
        frame.render_widget(
            Paragraph::new(footer).block(Block::default().bg(self.theme.surface).borders(Borders::TOP).border_style(Style::default().fg(self.theme.border_inactive))),
            chunks[2],
        );

        if self.input_modal.modal.active { self.input_modal.draw(frame, area, ctx); }
    }
}

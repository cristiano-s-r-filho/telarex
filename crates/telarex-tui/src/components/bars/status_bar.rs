use ratatui::prelude::Stylize;
use ratatui::{
    layout::{Alignment, Rect, Layout, Constraint},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Clear, Paragraph, Block, Padding},
    Frame,
};
use crate::tui_compat::{AppContext, Component, DrawContext, Event, EventResult};
use uuid::Uuid;

pub struct StatusBar {
    pub file_path: Option<String>,
    pub modified: bool,
    pub cursor_position: (usize, usize),
    pub language: Option<String>,
    pub selection_count: usize,
    pub editor_mode: String,
    pub lodge_id: Option<Uuid>,
    pub lodge_status: String,
    pub peer_count: usize,
    pub username: String,
    pub git_branch: Option<String>,
    pub git_dirty: usize,
}

impl Default for StatusBar {
    fn default() -> Self {
        Self {
            file_path: None,
            modified: false,
            cursor_position: (0, 0),
            language: None,
            selection_count: 0,
            editor_mode: "EDIT".to_string(),
            lodge_id: None,
            lodge_status: "Offline".to_string(),
            peer_count: 0,
            username: "User".to_string(),
            git_branch: None,
            git_dirty: 0,
        }
    }
}

impl Component for StatusBar {
    fn handle_event(&mut self, _event: &Event, _ctx: &mut AppContext) -> EventResult {
        EventResult::Unhandled
    }

    fn draw(&self, frame: &mut Frame, area: Rect, _ctx: &DrawContext) {
        frame.render_widget(Clear, area);
        
        let layout = Layout::horizontal([
            Constraint::Fill(1), // Path & Mode & Git
            Constraint::Length(45), // Lodge & Connectivity
            Constraint::Length(30), // Cursor & User
        ]).split(area);

        // 1. Path & Mode & Git Pill
        let path = self.file_path.as_deref().unwrap_or("Untitled");
        let mod_marker = if self.modified { " ●" } else { "" };
        let git_pill = self.git_branch.as_ref().map(|b| {
            if self.git_dirty > 0 {
                format!(" {} +{} ", b, self.git_dirty)
            } else {
                format!(" {} ", b)
            }
        }).unwrap_or_default();
        let left_pill = Line::from(vec![
            Span::styled(format!(" {} ", self.editor_mode), Style::default().bg(Color::Blue).fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::raw(format!(" {} ", path)),
            Span::styled(mod_marker, Style::default().fg(Color::Yellow)),
            Span::styled(git_pill, Style::default().fg(Color::DarkGray).bg(Color::Rgb(30, 30, 30))),
        ]);
        frame.render_widget(Paragraph::new(left_pill).block(Block::default().padding(Padding::new(1, 0, 0, 0))), layout[0]);

        // 2. Lodge Pill
        let lodge_color = if self.lodge_status == "Online" { Color::Green } else { Color::Red };
        let id_str = self.lodge_id.map(|id| id.to_string()[..8].to_string()).unwrap_or_else(|| "none".to_string());
        let lodge_pill = Line::from(vec![
            Span::styled(" 󰒄 ", Style::default().fg(lodge_color)),
            Span::styled(format!("LODGE:{} ", id_str), Style::default().fg(Color::White)),
            Span::styled(format!(" ({})", self.peer_count), Style::default().fg(Color::DarkGray)),
        ]);
        frame.render_widget(Paragraph::new(lodge_pill).alignment(Alignment::Center).block(Block::default().bg(Color::Rgb(30, 30, 30))), layout[1]);

        // 3. Cursor & User Pill
        let right_pill = Line::from(vec![
            Span::raw(format!(" {} ", self.username)),
            Span::styled(format!(" {}:{} ", self.cursor_position.0, self.cursor_position.1), Style::default().bg(Color::Rgb(50, 50, 50)).fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]).alignment(Alignment::Right);
        frame.render_widget(Paragraph::new(right_pill).block(Block::default().padding(Padding::new(0, 1, 0, 0))), layout[2]);
    }
}

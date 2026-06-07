use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Block, Borders, Clear},
    Frame,
    prelude::Stylize,
};
use crate::theme::Theme;
use crate::utils::{centered_rect_fixed, sanitize};

/// Shared modal base: renders the backdrop, bordered block with title,
/// and returns the inner content rect. Uses theme colors by default.
pub struct Modal {
    pub active: bool,
    pub title: String,
}

impl Modal {
    pub fn new(title: &str) -> Self {
        Self { active: false, title: title.to_string() }
    }

    pub fn show(&mut self) { self.active = true; }

    pub fn hide(&mut self) { self.active = false; }

    /// Render a modal with themed border/background.
    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme, width: u16, height: u16) -> Option<Rect> {
        if !self.active { return None; }
        let modal_area = centered_rect_fixed(width, height, area);
        frame.render_widget(Clear, modal_area);
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", sanitize(&self.title)))
            .border_style(Style::default().fg(theme.border_active))
            .bg(theme.bg);
        let inner = block.inner(modal_area);
        frame.render_widget(block, modal_area);
        Some(inner)
    }
}

//! Error modal — displays errors, warnings, and info messages with level-based styling.
use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{
    layout::{Rect, Layout, Constraint},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
    prelude::Stylize,
};
use crate::tui_compat::{AppContext, Component, DrawContext, Event, EventResult};
use telarex_core::errors::{TrexError, ErrorLevel};
use crate::utils::sanitize;
use crate::theme::Theme;

/// Error/info modal — displays a [`TrexError`] with level-appropriate styling.
pub struct ErrorModal {
    /// Whether the modal is currently visible.
    pub active: bool,
    /// The error to display, if any.
    pub current_error: Option<TrexError>,
    /// The current theme.
    pub theme: Theme,
}

impl ErrorModal {
    /// Creates a new hidden `ErrorModal`.
    pub fn new() -> Self {
        Self {
            active: false,
            current_error: None,
            theme: Theme::default(),
        }
    }

    /// Shows the modal with the given error.
    pub fn show(&mut self, error: TrexError) {
        self.active = true;
        self.current_error = Some(error);
    }

    /// Hides the modal and clears the current error.
    pub fn hide(&mut self) {
        self.active = false;
        self.current_error = None;
    }
}

impl Component for ErrorModal {
    fn handle_event(&mut self, event: &Event, _ctx: &mut AppContext) -> EventResult {
        if !self.active { return EventResult::Unhandled; }

        if let Event::Key(key) = event {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Enter | KeyCode::Esc | KeyCode::Char(' ') => {
                        self.hide();
                        return EventResult::Handled;
                    }
                    _ => {}
                }
            }
        }
        EventResult::Handled
    }

    fn draw(&self, frame: &mut Frame, area: Rect, _ctx: &DrawContext) {
        if !self.active { return; }
        let Some(error) = &self.current_error else { return; };

        let modal_area = crate::utils::centered_rect_fixed(60, 8, area);
        frame.render_widget(Clear, modal_area);

        let border_color = match error.level {
            ErrorLevel::Info => self.theme.info,
            ErrorLevel::Warning => self.theme.warning,
            ErrorLevel::Error => self.theme.error,
            ErrorLevel::Fatal => self.theme.error,
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .bg(self.theme.surface_alt)
            .title(format!(" Error: {} ", sanitize(&error.code)));

        let inner = block.inner(modal_area);
        frame.render_widget(block, modal_area);

        let layout = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ]).split(inner);

        frame.render_widget(Paragraph::new(sanitize(&error.message)).style(Style::default().fg(self.theme.fg).add_modifier(Modifier::BOLD)), layout[0]);

        let solution = Line::from(vec![
            Span::styled(" [Solution] ", Style::default().fg(border_color)),
            Span::styled(sanitize(&error.solution), Style::default().fg(self.theme.fg)),
        ]);
        frame.render_widget(Paragraph::new(solution), layout[1]);

        frame.render_widget(Paragraph::new(" Press Enter/Esc to dismiss ").style(Style::default().fg(Color::DarkGray)), layout[2]);
    }
}

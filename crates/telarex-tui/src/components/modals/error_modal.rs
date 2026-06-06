use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{
    layout::{Rect, Layout, Constraint},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use crate::tui_compat::{AppContext, Component, DrawContext, Event, EventResult};
use telarex_core::errors::{TrexError, ErrorLevel};
use crate::utils::sanitize;

pub struct ErrorModal {
    pub active: bool,
    pub current_error: Option<TrexError>,
}

impl ErrorModal {
    pub fn new() -> Self {
        Self {
            active: false,
            current_error: None,
        }
    }

    pub fn show(&mut self, error: TrexError) {
        self.active = true;
        self.current_error = Some(error);
    }

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

        let area = crate::utils::centered_rect_fixed(60, 8, area);
        
        let color = match error.level {
            ErrorLevel::Info => Color::Cyan,
            ErrorLevel::Warning => Color::Yellow,
            ErrorLevel::Error => Color::Red,
            ErrorLevel::Fatal => Color::LightRed,
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(color))
            .title(format!(" Error: {} ", sanitize(&error.code)));

        let inner = block.inner(area);
        frame.render_widget(Clear, area);
        frame.render_widget(block, area);

        let layout = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ]).split(inner);

        frame.render_widget(Paragraph::new(sanitize(&error.message)).style(Style::default().add_modifier(Modifier::BOLD)), layout[0]);
        
        let solution = Line::from(vec![
            Span::styled(" [Solution] ", Style::default().fg(color)),
            Span::raw(sanitize(&error.solution)),
        ]);
        frame.render_widget(Paragraph::new(solution), layout[1]);
        
        let footer = " Press Enter/Esc to dismiss ";
        frame.render_widget(Paragraph::new(footer).style(Style::default().fg(Color::DarkGray)), layout[2]);
    }
}

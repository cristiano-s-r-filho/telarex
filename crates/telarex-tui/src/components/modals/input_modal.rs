use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::prelude::Stylize;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Style, Modifier},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use crate::tui_compat::{AppContext, Component, DrawContext, Event, EventResult};
use crate::utils::sanitize;
use crate::theme::Theme;

pub struct InputModal {
    pub title: String,
    pub value: String,
    pub active: bool,
    pub theme: Theme,
}

impl InputModal {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            value: String::new(),
            active: false,
            theme: Theme::default(),
        }
    }

    pub fn show(&mut self) {
        self.active = true;
        self.value.clear();
    }

    pub fn hide(&mut self) {
        self.active = false;
    }

    pub fn take_value(&mut self) -> Option<String> {
        if !self.active && !self.value.is_empty() {
            let val = self.value.clone();
            self.value.clear();
            Some(val)
        } else {
            None
        }
    }
}

impl Component for InputModal {
    fn handle_event(&mut self, event: &Event, _ctx: &mut AppContext) -> EventResult {
        if !self.active { return EventResult::Unhandled; }

        if let Event::Key(KeyEvent { code, kind: KeyEventKind::Press, .. }) = event {
            match code {
                KeyCode::Esc => { self.hide(); }
                KeyCode::Enter => { self.active = false; }
                KeyCode::Backspace => { self.value.pop(); }
                KeyCode::Char(c) => { self.value.push(*c); }
                _ => {}
            }
            return EventResult::Handled;
        }
        EventResult::Unhandled
    }

    fn draw(&self, frame: &mut Frame, area: Rect, _ctx: &DrawContext) {
        if !self.active { return; }

        let modal_area = crate::utils::centered_rect(60, 20, area);
        frame.render_widget(Clear, modal_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(sanitize(&format!(" {} ", self.title)))
            .border_style(Style::default().fg(self.theme.border_active))
            .bg(self.theme.bg);
        
        let inner = block.inner(modal_area);
        frame.render_widget(block, modal_area);

        let input_style = Style::default().add_modifier(Modifier::BOLD);
        let p = Paragraph::new(sanitize(&self.value))
            .style(input_style)
            .alignment(Alignment::Left);
        
        frame.render_widget(p, inner);
        
        // Render cursor
        frame.set_cursor_position(ratatui::layout::Position::new(
            inner.x + self.value.len() as u16,
            inner.y
        ));
    }
}

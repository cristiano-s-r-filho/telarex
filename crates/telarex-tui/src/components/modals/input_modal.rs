use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Style, Modifier},
    widgets::Paragraph,
    Frame,
};
use crate::tui_compat::{AppContext, Component, DrawContext, Event, EventResult};
use crate::utils::sanitize;
use crate::theme::Theme;
use super::modal::Modal;

pub struct InputModal {
    pub modal: Modal,
    pub value: String,
    pub theme: Theme,
}

impl InputModal {
    pub fn new(title: &str) -> Self {
        Self {
            modal: Modal::new(title),
            value: String::new(),
            theme: Theme::default(),
        }
    }

    pub fn show(&mut self) {
        self.modal.show();
        self.value.clear();
    }

    pub fn hide(&mut self) {
        self.modal.hide();
    }

    pub fn take_value(&mut self) -> Option<String> {
        if !self.modal.active && !self.value.is_empty() {
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
        if !self.modal.active { return EventResult::Unhandled; }

        if let Event::Key(KeyEvent { code, kind: KeyEventKind::Press, .. }) = event {
            match code {
                KeyCode::Esc => { self.hide(); }
                KeyCode::Enter => { self.modal.active = false; }
                KeyCode::Backspace => { self.value.pop(); }
                KeyCode::Char(c) => { self.value.push(*c); }
                _ => {}
            }
            return EventResult::Handled;
        }
        EventResult::Unhandled
    }

    fn draw(&self, frame: &mut Frame, area: Rect, _ctx: &DrawContext) {
        let inner = match self.modal.render(frame, area, &self.theme, 50, 8) {
            Some(r) => r,
            None => return,
        };
        let input_style = Style::default().add_modifier(Modifier::BOLD);
        let p = Paragraph::new(sanitize(&self.value))
            .style(input_style)
            .alignment(Alignment::Left);
        frame.render_widget(p, inner);
        frame.set_cursor_position(ratatui::layout::Position::new(
            inner.x + self.value.len() as u16,
            inner.y
        ));
    }
}

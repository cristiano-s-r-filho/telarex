//! Input modal — simple text input overlay for folder names, paths, etc.
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

/// Simple text input modal — collects a single line of text from the user.
pub struct InputModal {
    /// The underlying modal widget.
    pub modal: Modal,
    /// The current value of the input field.
    pub value: String,
    /// The current theme.
    pub theme: Theme,
}

impl InputModal {
    /// Creates a new hidden `InputModal` with the given title.
    pub fn new(title: &str) -> Self {
        Self {
            modal: Modal::new(title),
            value: String::new(),
            theme: Theme::default(),
        }
    }

    /// Shows the modal and clears the input value.
    pub fn show(&mut self) {
        self.modal.show();
        self.value.clear();
    }

    /// Hides the modal.
    pub fn hide(&mut self) {
        self.modal.hide();
    }

    /// Returns the entered value if the modal was confirmed (closed via Enter).
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

use crate::components::modals::ConfigModal;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::Frame;
use ratatui::layout::Rect;
use crate::tui_compat::{AppContext, Component, DrawContext, Event, EventResult};
use crate::theme::Theme;

pub struct ConfigView {
    pub modal: ConfigModal,
    pub should_exit: bool,
}

impl ConfigView {
    pub fn new(session_id: Option<String>) -> Self {
        Self {
            modal: ConfigModal::new(session_id),
            should_exit: false,
        }
    }

    pub fn apply_theme(&mut self, theme: &Theme) {
        self.modal.apply_theme(theme);
    }

    pub fn show(&mut self) {
        self.modal.show();
        self.should_exit = false;
    }

    pub fn get_config(&self) -> telarex_core::config::TelaRexConfig {
        self.modal.get_config().clone()
    }
}

impl Component for ConfigView {
    fn draw(&self, frame: &mut Frame, area: Rect, ctx: &DrawContext) {
        self.modal.draw(frame, area, ctx);
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut AppContext) -> EventResult {
        let res = self.modal.handle_event(event, ctx);
        if res.is_handled() {
            return res;
        }

        if let Event::Key(KeyEvent { code: KeyCode::Esc, kind: KeyEventKind::Press, .. }) = event {
            self.should_exit = true;
            return EventResult::Handled;
        }

        EventResult::Unhandled
    }
}

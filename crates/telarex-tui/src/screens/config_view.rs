//! Configuration screen — wraps [`ConfigModal`] as a full-screen view.
use crate::components::modals::ConfigModal;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::Frame;
use ratatui::layout::Rect;
use crate::tui_compat::{AppContext, Component, DrawContext, Event, EventResult};
use crate::theme::Theme;

/// Configuration screen — wraps [`ConfigModal`] as a full-screen view.
pub struct ConfigView {
    /// The configuration modal.
    pub modal: ConfigModal,
    /// Set to `true` when the user exits the config screen.
    pub should_exit: bool,
}

impl ConfigView {
    /// Creates a new `ConfigView` with the given session ID.
    pub fn new(session_id: Option<String>) -> Self {
        Self {
            modal: ConfigModal::new(session_id),
            should_exit: false,
        }
    }

    /// Applies a theme to the configuration modal.
    pub fn apply_theme(&mut self, theme: &Theme) {
        self.modal.apply_theme(theme);
    }

    /// Shows the config modal and resets the exit flag.
    pub fn show(&mut self) {
        self.modal.show();
        self.should_exit = false;
    }

    /// Returns the current configuration from the modal.
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

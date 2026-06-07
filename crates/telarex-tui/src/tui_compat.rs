//! Compatibility layer — Component trait, event types, and shared context.
pub use crossterm::event::Event;
use ratatui::layout::Rect;
use ratatui::Frame;

/// Result of handling an event — either consumed or passed through.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventResult {
    /// The event was consumed and should not propagate further.
    Handled,
    /// The event was not handled and may be processed by a parent.
    Unhandled,
}

impl EventResult {
    /// Returns `true` if the result is [`EventResult::Handled`].
    pub fn is_handled(&self) -> bool {
        matches!(self, EventResult::Handled)
    }
}

/// Shared drawing context — currently a marker type for future expansion.
pub struct DrawContext;

/// Application-level context — carries the quit flag shared across components.
pub struct AppContext {
    should_quit: bool,
}

impl AppContext {
    /// Creates a new `AppContext` with the quit flag initially `false`.
    pub fn new() -> Self {
        Self { should_quit: false }
    }

    /// Signals that the application should exit on the next loop iteration.
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Returns `true` if [`quit`](Self::quit) has been called.
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }
}

/// A drawable, event-handling widget that participates in the TUI lifecycle.
pub trait Component {
    fn draw(&self, frame: &mut Frame, area: Rect, ctx: &DrawContext);
    fn handle_event(&mut self, event: &Event, ctx: &mut AppContext) -> EventResult;
}

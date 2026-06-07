pub use crossterm::event::Event;
use ratatui::layout::Rect;
use ratatui::Frame;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventResult {
    Handled,
    Unhandled,
}

impl EventResult {
    pub fn is_handled(&self) -> bool {
        matches!(self, EventResult::Handled)
    }
}

pub struct DrawContext;

pub struct AppContext {
    should_quit: bool,
}

impl AppContext {
    pub fn new() -> Self {
        Self { should_quit: false }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }
}

pub trait Component {
    fn draw(&self, frame: &mut Frame, area: Rect, ctx: &DrawContext);
    fn handle_event(&mut self, event: &Event, ctx: &mut AppContext) -> EventResult;
}

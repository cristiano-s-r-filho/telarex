//! Macro palette — record and replay keyboard macros.
use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    widgets::{List, ListItem, ListState},
    Frame,
};
use crate::theme::Theme;
use crate::tui_compat::{AppContext, Component, DrawContext, Event, EventResult};
use super::modal::Modal;

/// Macro palette — record and replay keyboard macros.
pub struct MacroPalette {
    /// The underlying modal widget.
    pub modal: Modal,
    /// Ratatui list state for action selection.
    pub list_state: ListState,
    committed_action: Option<MacroAction>,
    /// The current theme.
    pub theme: Theme,
}

/// A macro action selected by the user — record new or play existing.
#[derive(Clone, Debug)]
pub enum MacroAction {
    RecordNew,
    Play(String),
}

impl MacroPalette {
    /// Creates a new hidden `MacroPalette`.
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            modal: Modal::new(" Macros "),
            list_state,
            committed_action: None,
            theme: Theme::default(),
        }
    }

    /// Returns the action the user confirmed, if any.
    pub fn take_action(&mut self) -> Option<MacroAction> {
        self.committed_action.take()
    }
}

impl Component for MacroPalette {
    fn handle_event(&mut self, event: &Event, _ctx: &mut AppContext) -> EventResult {
        if !self.modal.active { return EventResult::Unhandled; }

        if let Event::Key(key) = event {
            if key.kind != KeyEventKind::Press { return EventResult::Handled; }

            match key.code {
                KeyCode::Esc => { self.modal.active = false; return EventResult::Handled; }
                KeyCode::Up => {
                    let i = self.list_state.selected().unwrap_or(0);
                    if i > 0 { self.list_state.select(Some(i - 1)); }
                    return EventResult::Handled;
                }
                KeyCode::Down => {
                    let i = self.list_state.selected().unwrap_or(0);
                    if i < 1 { self.list_state.select(Some(i + 1)); }
                    return EventResult::Handled;
                }
                KeyCode::Enter => {
                    match self.list_state.selected() {
                        Some(0) => self.committed_action = Some(MacroAction::RecordNew),
                        Some(1) => self.committed_action = Some(MacroAction::Play("default".to_string())),
                        _ => {}
                    }
                    self.modal.active = false;
                    return EventResult::Handled;
                }
                _ => {}
            }
        }
        EventResult::Handled
    }

    fn draw(&self, _frame: &mut Frame, _area: Rect, _ctx: &DrawContext) {
        // Handled by render helper
    }
}

impl MacroPalette {
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let inner = match self.modal.render(frame, area, &self.theme, 40, 6) {
            Some(r) => r,
            None => return,
        };

        let items = vec![
            ListItem::new(" [R] Record New Macro ").style(Style::default().fg(self.theme.fg)),
            ListItem::new(" [P] Play Last Macro ").style(Style::default().fg(self.theme.fg)),
        ];

        let list = List::new(items)
            .highlight_style(
                Style::default()
                    .bg(self.theme.selection_bg)
                    .fg(self.theme.selection_fg)
                    .add_modifier(Modifier::BOLD)
            )
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, inner, &mut self.list_state);
    }
}

use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style, Color},
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
    Frame,
};
use crate::tui_compat::{AppContext, Component, DrawContext, Event, EventResult};

pub struct MacroPalette {
    pub active: bool,
    pub list_state: ListState,
    committed_action: Option<MacroAction>,
}

#[derive(Clone, Debug)]
pub enum MacroAction {
    RecordNew,
    Play(String),
}

impl MacroPalette {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            active: false,
            list_state,
            committed_action: None,
        }
    }

    pub fn take_action(&mut self) -> Option<MacroAction> {
        self.committed_action.take()
    }
}

impl Component for MacroPalette {
    fn handle_event(&mut self, event: &Event, _ctx: &mut AppContext) -> EventResult {
        if !self.active { return EventResult::Unhandled; }

        if let Event::Key(key) = event {
            if key.kind != KeyEventKind::Press { return EventResult::Handled; }

            match key.code {
                KeyCode::Esc => { self.active = false; return EventResult::Handled; }
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
                    self.active = false;
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
        if !self.active { return; }

        let palette_area = crate::utils::centered_rect_fixed(40, 6, area);
        frame.render_widget(Clear, palette_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Macros ")
            .border_style(Style::default().fg(Color::Cyan));
        
        let inner = block.inner(palette_area);
        frame.render_widget(block, palette_area);

        let items = vec![
            ListItem::new(" [R] Record New Macro "),
            ListItem::new(" [P] Play Last Macro "),
        ];

        let list = List::new(items)
            .highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White).add_modifier(Modifier::BOLD))
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, inner, &mut self.list_state);
    }
}

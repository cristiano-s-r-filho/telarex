#[allow(unused_imports)]
use ratatui::{
    layout::{Rect, Constraint, Layout},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::components::tab_controller::TabController;
use crate::theme::Theme;
use crate::tui_compat::{AppContext, Component, DrawContext, Event, EventResult};

pub struct TabBar {
    pub theme: Theme,
}

impl Component for TabBar {
    fn draw(&self, _frame: &mut Frame, _area: Rect, _ctx: &DrawContext) {
        // TabBar needs a TabController which isn't in DrawContext.
        // Usually, the parent (EditorView) calls render() instead of draw().
    }

    fn handle_event(&mut self, _event: &Event, _ctx: &mut AppContext) -> EventResult {
        EventResult::Unhandled
    }
}

impl TabBar {
    pub fn render(&self, frame: &mut Frame, area: Rect, tabs: &TabController) {
        let mut spans = Vec::new();
        
        for (i, tab) in tabs.tabs.iter().enumerate() {
            let is_active = i == tabs.active_tab;
            let style = if is_active {
                self.theme.list_selected
            } else {
                self.theme.list_inactive
            };

            spans.push(Span::styled(format!(" [{}] ", tab.name), style));
            spans.push(Span::styled(" | ", self.theme.list_inactive));
        }

        let bar = Paragraph::new(Line::from(spans))
            .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(self.theme.border_inactive)));
        
        frame.render_widget(bar, area);
    }
}

//! Tab bar — renders tab names for the [`TabController`].
use ratatui::prelude::Stylize;
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame,
};
use crate::components::tab_controller::TabController;
use crate::theme::Theme;
use crate::tui_compat::{AppContext, Component, DrawContext, Event, EventResult};

/// Tab bar widget — renders tab names and highlights the active tab.
pub struct TabBar {
    /// The current theme.
    pub theme: Theme,
}

impl Component for TabBar {
    fn draw(&self, _frame: &mut Frame, _area: Rect, _ctx: &DrawContext) {
    }

    fn handle_event(&mut self, _event: &Event, _ctx: &mut AppContext) -> EventResult {
        EventResult::Unhandled
    }
}

impl TabBar {
    pub fn render(&self, frame: &mut Frame, area: Rect, tabs: &TabController) {
        let bg = self.theme.surface;
        let mut spans = Vec::new();

        for (i, tab) in tabs.tabs.iter().enumerate() {
            let is_active = i == tabs.active_tab;
            let style = if is_active {
                self.theme.tab_active
            } else {
                self.theme.tab_inactive
            };

            let pane_count: usize = tab.layout.nodes.iter()
                .filter(|n| matches!(n.kind, crate::components::NodeKind::Pane(_)))
                .count();
            let name = &tab.name;

            if is_active {
                spans.push(Span::styled(format!(" {} ", name), style));
                if pane_count > 1 {
                    spans.push(Span::styled(format!("[{}]", pane_count), style));
                }
                spans.push(Span::styled(" ", Style::default().bg(self.theme.accent)));
            } else {
                spans.push(Span::styled(format!(" {} ", name), style));
                spans.push(Span::styled(" ", Style::default().bg(bg)));
            }
        }

        if tabs.tabs.is_empty() {
            spans.push(Span::styled(" ", Style::default().bg(bg)));
        }

        let line = Line::from(spans);
        let bar = Paragraph::new(line).block(Block::default().bg(bg));
        frame.render_widget(bar, area);
    }
}

use telarex_core::command::Command;
use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Rect, Layout, Constraint},
    style::{Modifier, Style, Color},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};
use crate::tui_compat::{AppContext, Component, DrawContext, Event, EventResult};

use crate::theme::Theme;
use ratatui::prelude::Stylize;

pub struct CommandPalette {
    input: String,
    all_commands: Vec<Command>,
    filtered_commands: Vec<Command>,
    pub list_state: ListState,
    pub active: bool,
    pub theme: Theme,
    committed_command: Option<Command>,
}

impl CommandPalette {
    pub fn new() -> Self {
        let all_commands = Command::all();
        let mut state = ListState::default();
        if !all_commands.is_empty() {
            state.select(Some(0));
        }
        Self {
            input: String::new(),
            filtered_commands: all_commands.clone(),
            all_commands,
            list_state: state,
            active: false,
            theme: Theme::default(),
            committed_command: None,
        }
    }

    pub fn show(&mut self) {
        self.active = true;
        self.input.clear();
        self.update_filter();
        if !self.filtered_commands.is_empty() {
            self.list_state.select(Some(0));
        }
        self.committed_command = None;
    }

    pub fn hide(&mut self) {
        self.active = false;
        self.committed_command = None;
    }

    pub fn take_selected(&mut self) -> Option<Command> {
        self.committed_command.take()
    }

    fn update_filter(&mut self) {
        let pattern = self.input.to_lowercase();
        self.filtered_commands = self
            .all_commands
            .iter()
            .filter(|cmd| {
                cmd.name().to_lowercase().contains(&pattern)
                    || cmd.description().to_lowercase().contains(&pattern)
            })
            .copied()
            .collect();

        if !self.filtered_commands.is_empty() {
            if self.list_state.selected().is_none() || self.list_state.selected().unwrap() >= self.filtered_commands.len() {
                self.list_state.select(Some(0));
            }
        } else {
            self.list_state.select(None);
        }
    }
}

impl Component for CommandPalette {
    fn handle_event(&mut self, event: &Event, _ctx: &mut AppContext) -> EventResult {
        if !self.active {
            return EventResult::Unhandled;
        }

        if let Event::Key(key_event) = event {
            if key_event.kind != KeyEventKind::Press {
                return EventResult::Handled;
            }

            match key_event.code {
                KeyCode::Esc => {
                    self.hide();
                    return EventResult::Handled;
                }
                KeyCode::Enter => {
                    if let Some(i) = self.list_state.selected() {
                        if let Some(cmd) = self.filtered_commands.get(i) {
                            self.committed_command = Some(*cmd);
                            self.active = false;
                        }
                    }
                    return EventResult::Handled;
                }
                KeyCode::Up => {
                    let i = self.list_state.selected().unwrap_or(0);
                    if i > 0 {
                        self.list_state.select(Some(i - 1));
                    }
                    return EventResult::Handled;
                }
                KeyCode::Down => {
                    let i = self.list_state.selected().unwrap_or(0);
                    if i + 1 < self.filtered_commands.len() {
                        self.list_state.select(Some(i + 1));
                    }
                    return EventResult::Handled;
                }
                KeyCode::Char(c) => {
                    if !key_event.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) {
                        self.input.push(c);
                        self.update_filter();
                        return EventResult::Handled;
                    }
                }
                KeyCode::Backspace => {
                    self.input.pop();
                    self.update_filter();
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

impl CommandPalette {
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        if !self.active {
            return;
        }

        let palette_area = crate::utils::centered_rect_fixed(70, 15, area);

        frame.render_widget(Clear, palette_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Command Palette ")
            .border_style(Style::default().fg(self.theme.border_active))
            .bg(self.theme.bg);
        
        let inner = block.inner(palette_area);
        frame.render_widget(block, palette_area);

        let input_line = Line::from(vec![
            Span::styled("   ", Style::default().fg(self.theme.border_active)),
            Span::styled(&self.input, Style::default().fg(self.theme.fg).add_modifier(Modifier::BOLD)),
            Span::styled("█", Style::default().fg(self.theme.border_active)),
        ]);
        
        let chunks = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(inner);

        frame.render_widget(Paragraph::new(input_line).bg(self.theme.bg), chunks[0]);

        let items: Vec<ListItem> = self
            .filtered_commands
            .iter()
            .map(|cmd| {
                let name = cmd.name();
                let pattern = self.input.to_lowercase();
                
                let mut spans = vec![Span::raw(" 󰘳  ")];
                
                if !pattern.is_empty() {
                    if let Some(pos) = name.to_lowercase().find(&pattern) {
                        spans.push(Span::raw(&name[..pos]));
                        spans.push(Span::styled(
                            &name[pos..pos + pattern.len()],
                            Style::default().fg(self.theme.border_active).add_modifier(Modifier::BOLD)
                        ));
                        spans.push(Span::raw(&name[pos + pattern.len()..]));
                    } else {
                        spans.push(Span::raw(name));
                    }
                } else {
                    spans.push(Span::raw(name));
                }
                
                let name_len = spans.iter().map(|s| s.content.chars().count()).sum::<usize>();
                if name_len < 20 {
                    spans.push(Span::raw(" ".repeat(20 - name_len)));
                }
                
                spans.push(Span::raw("  "));
                spans.push(Span::styled(cmd.description(), Style::default().fg(Color::Gray)));

                ListItem::new(Line::from(spans)).style(Style::default().fg(self.theme.fg))
            })
            .collect();

        let list = List::new(items)
            .highlight_style(
                Style::default()
                    .bg(self.theme.selection_bg)
                    .fg(self.theme.fg)
                    .add_modifier(Modifier::BOLD)
            )
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, chunks[1], &mut self.list_state);
    }
}

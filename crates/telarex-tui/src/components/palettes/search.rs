//! Search palette — project-wide file content search with result navigation.
use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Rect, Layout, Constraint},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};
use crate::tui_compat::{AppContext, Component, DrawContext, Event, EventResult};
use crate::theme::Theme;
use ratatui::prelude::Stylize;
use std::path::PathBuf;

/// Project search palette — searches file contents and navigates results.
pub struct SearchPalette {
    /// Current search query text.
    pub input: String,
    /// Whether the palette is currently open.
    pub active: bool,
    /// Accumulated search results.
    pub results: Vec<SearchResult>,
    /// Ratatui list state for result navigation.
    pub list_state: ListState,
    /// The current theme.
    pub theme: Theme,
    /// Set to `true` when the user requests a project-wide search.
    pub search_requested: bool,
    committed_result: Option<SearchResult>,
}

/// A single search result — file, line number, and matching content.
#[derive(Clone, Debug)]
pub struct SearchResult {
    /// The file containing the match.
    pub file: PathBuf,
    /// 1-based line number of the match.
    pub line_number: usize,
    /// The text of the matching line.
    pub content: String,
}

impl SearchPalette {
    /// Creates a new empty `SearchPalette`.
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(None);
        Self {
            input: String::new(),
            active: false,
            results: Vec::new(),
            list_state,
            theme: Theme::default(),
            search_requested: false,
            committed_result: None,
        }
    }

    /// Opens the search palette and clears the previous query and results.
    pub fn show(&mut self) {
        self.active = true;
        self.input.clear();
        self.results.clear();
        self.list_state.select(None);
        self.search_requested = false;
        self.committed_result = None;
    }

    /// Closes the search palette.
    pub fn hide(&mut self) {
        self.active = false;
        self.search_requested = false;
        self.committed_result = None;
    }

    /// Returns the result the user confirmed, if any.
    pub fn take_selected(&mut self) -> Option<SearchResult> {
        self.committed_result.take()
    }

    /// Returns and clears the search-requested flag.
    pub fn take_search_request(&mut self) -> bool {
        let req = self.search_requested;
        self.search_requested = false;
        req
    }

    /// Returns the current search query text.
    pub fn get_query(&self) -> String {
        self.input.clone()
    }

    /// Adds a single search result to the palette.
    pub fn add_result(&mut self, result: SearchResult) {
        self.results.push(result);
        if self.results.len() == 1 {
            self.list_state.select(Some(0));
        }
    }

    /// Replaces all search results with a new batch.
    pub fn update_results(&mut self, results: Vec<SearchResult>) {
        self.results = results;
        if !self.results.is_empty() {
            self.list_state.select(Some(0));
        }
    }
}

impl Component for SearchPalette {
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
                    if self.results.is_empty() {
                        self.search_requested = true;
                    } else if let Some(i) = self.list_state.selected() {
                        if let Some(res) = self.results.get(i) {
                            self.committed_result = Some(res.clone());
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
                    if i + 1 < self.results.len() {
                        self.list_state.select(Some(i + 1));
                    }
                    return EventResult::Handled;
                }
                KeyCode::Char(c) => {
                    if !key_event.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) {
                        self.input.push(c);
                        return EventResult::Handled;
                    }
                }
                KeyCode::Backspace => {
                    self.input.pop();
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

impl SearchPalette {
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        if !self.active {
            return;
        }

        let palette_area = crate::utils::centered_rect_fixed(80, 15, area);

        frame.render_widget(Clear, palette_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Project Search ")
            .border_style(Style::default().fg(self.theme.border_active))
            .bg(self.theme.surface_alt);
        
        let inner = block.inner(palette_area);
        frame.render_widget(block, palette_area);

        let input_line = Line::from(vec![
            Span::styled(" > ", Style::default().fg(self.theme.border_active)),
            Span::styled(&self.input, Style::default().fg(self.theme.fg).add_modifier(Modifier::BOLD)),
            Span::styled("_", Style::default().fg(self.theme.border_active)),
        ]);
        
        let chunks = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(inner);

        frame.render_widget(Paragraph::new(input_line).bg(self.theme.surface_alt), chunks[0]);

        let list = if self.results.is_empty() && !self.input.is_empty() {
            List::new(vec![ListItem::new("  No results found. Press Enter to search ")
                .style(Style::default().fg(self.theme.fg_dim))])
        } else if self.results.is_empty() {
            List::new(vec![ListItem::new("  Type and press Enter to search project-wide ")
                .style(Style::default().fg(self.theme.fg_dim))])
        } else {
            let items: Vec<ListItem> = self
                .results
                .iter()
                .map(|res| {
                    let file_name = res.file.file_name().unwrap_or_default().to_string_lossy();
                    let pattern = self.input.to_lowercase();
                    
                    let mut spans = vec![
                        Span::styled(" [F] ", Style::default().fg(self.theme.accent)),
                        Span::styled(format!("{}:{}  ", file_name, res.line_number), Style::default().fg(self.theme.fg_dim)),
                    ];

                    let content = res.content.trim();
                    if !pattern.is_empty() {
                        if let Some(pos) = content.to_lowercase().find(&pattern) {
                            spans.push(Span::raw(&content[..pos]));
                            spans.push(Span::styled(
                                &content[pos..pos + pattern.len()],
                                Style::default().fg(self.theme.warning).add_modifier(Modifier::BOLD)
                            ));
                            spans.push(Span::raw(&content[pos + pattern.len()..]));
                        } else {
                            spans.push(Span::raw(content));
                        }
                    } else {
                        spans.push(Span::raw(content));
                    }

                    ListItem::new(Line::from(spans)).style(Style::default().fg(self.theme.fg))
                })
                .collect();

            List::new(items)
                .highlight_style(
                    Style::default()
                        .bg(self.theme.selection_bg)
                        .fg(self.theme.fg)
                        .add_modifier(Modifier::BOLD)
                )
                .highlight_symbol("> ")
        };

        frame.render_stateful_widget(list, chunks[1], &mut self.list_state);
    }
}

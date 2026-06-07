//! Status bar — displays file info, cursor position, git state, and network status.
use ratatui::prelude::Stylize;
use ratatui::{
    layout::Rect,
    style::{Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame,
};
use crate::theme::Theme;
use crate::tui_compat::{AppContext, Component, DrawContext, Event, EventResult};
use uuid::Uuid;

/// Status bar widget — displays file path, cursor, mode, git, and network info.
pub struct StatusBar {
    /// Path of the active file, if any.
    pub file_path: Option<String>,
    /// Whether the active document has unsaved changes.
    pub modified: bool,
    /// 1-based (line, column) cursor position.
    pub cursor_position: (usize, usize),
    /// Detected programming language of the active document.
    pub language: Option<String>,
    /// Number of selected characters (0 if no selection).
    pub selection_count: usize,
    /// Current editing mode string (e.g. "EDIT", "INSERT").
    pub editor_mode: String,
    /// The active lodge UUID, if connected.
    pub lodge_id: Option<Uuid>,
    /// Lodge connection status string (e.g. "Online", "Offline").
    pub lodge_status: String,
    /// Number of connected peers in the lodge.
    pub peer_count: usize,
    /// Display name of the current user.
    pub username: String,
    /// Current git branch name, if any.
    pub git_branch: Option<String>,
    /// Number of dirty (modified/untracked/staged) files.
    pub git_dirty: usize,
    /// Index of the active tab (0-based).
    pub tab_index: usize,
    /// Total number of open tabs.
    pub tab_count: usize,
    /// The current theme.
    pub theme: Theme,
}

impl Default for StatusBar {
    fn default() -> Self {
        Self {
            file_path: None,
            modified: false,
            cursor_position: (0, 0),
            language: None,
            selection_count: 0,
            editor_mode: "EDIT".to_string(),
            lodge_id: None,
            lodge_status: "Offline".to_string(),
            peer_count: 0,
            username: "User".to_string(),
            git_branch: None,
            git_dirty: 0,
            tab_index: 0,
            tab_count: 0,
            theme: Theme::default(),
        }
    }
}

impl Component for StatusBar {
    fn handle_event(&mut self, _event: &Event, _ctx: &mut AppContext) -> EventResult {
        EventResult::Unhandled
    }

    fn draw(&self, frame: &mut Frame, area: Rect, _ctx: &DrawContext) {
        let bg = self.theme.status_bar_bg;
        let fg = self.theme.status_bar_fg;
        let success = self.theme.success;
        let warning = self.theme.warning;

        let path = self.file_path.as_deref().unwrap_or("Untitled");
        let mod_marker = if self.modified { " ●" } else { "" };

        let tab_pill = if self.tab_count > 0 {
            format!(" {}/{} ", self.tab_index + 1, self.tab_count)
        } else {
            String::new()
        };

        let git_pill = self.git_branch.as_ref().map(|b| {
            if self.git_dirty > 0 {
                format!(" {} +{} ", b, self.git_dirty)
            } else {
                format!(" {} ", b)
            }
        }).unwrap_or_default();

        let lodge_color = if self.lodge_status == "Online" { success } else { self.theme.error };
        let id_str = self.lodge_id.map(|id| shorten_uuid(id)).unwrap_or_else(|| "--".to_string());

        let selection_info = if self.selection_count > 0 {
            format!(" sel({}) ", self.selection_count)
        } else {
            String::new()
        };

        let lang_str = self.language.as_deref().unwrap_or("");

        let mode_color = match self.editor_mode.as_str() {
            "INSERT" => self.theme.mode_insert,
            "VISUAL" => self.theme.mode_visual,
            _ => self.theme.mode_normal,
        };

        let left_spans: Vec<Span> = vec![
            Span::styled(format!(" {} ", self.editor_mode), Style::default().bg(mode_color).fg(self.theme.status_bar_bg).add_modifier(Modifier::BOLD)),
        ];

        let center_spans: Vec<Span> = vec![
            Span::styled(path, Style::default().fg(fg)),
            Span::styled(mod_marker, Style::default().fg(warning)),
            Span::styled(tab_pill, Style::default().fg(self.theme.info)),
        ];

        let git_empty = git_pill.is_empty();

        let git_span = if !git_empty {
            vec![Span::styled(git_pill, Style::default().fg(fg))]
        } else {
            vec![]
        };

        let git_extra = if !git_empty {
            vec![Span::raw(" ")]
        } else {
            vec![]
        };

        let right_spans: Vec<Span> = vec![
            Span::styled(" ● ", Style::default().fg(lodge_color)),
            Span::styled(id_str, Style::default().fg(fg)),
            Span::styled(format!(" {}", self.peer_count), Style::default().fg(self.theme.info)),
        ];

        let far_right_spans: Vec<Span> = vec![
            if !lang_str.is_empty() {
                Span::styled(format!(" {} ", lang_str), Style::default().fg(self.theme.info))
            } else {
                Span::raw("")
            },
            Span::styled(selection_info, Style::default().fg(self.theme.info)),
            Span::styled(format!(" {}:{} ", self.cursor_position.0, self.cursor_position.1), Style::default().fg(self.theme.info)),
            Span::styled(format!(" {} ", self.username), Style::default().fg(fg)),
        ];

        let mut all_spans: Vec<Span> = Vec::new();
        all_spans.push(Span::styled(" ", Style::default().bg(bg)));

        for s in left_spans {
            all_spans.push(s);
        }

        all_spans.push(Span::styled(" ", Style::default().bg(bg)));

        for s in center_spans {
            all_spans.push(s);
        }

        for s in git_extra {
            all_spans.push(s);
        }

        for s in git_span {
            all_spans.push(s);
        }

        all_spans.push(Span::styled(" ", Style::default().bg(bg).fg(fg)));

        for s in right_spans {
            all_spans.push(s);
        }

        all_spans.push(Span::styled(" ", Style::default().bg(bg)));

        for s in far_right_spans {
            all_spans.push(s);
        }

        all_spans.push(Span::styled(" ", Style::default().bg(bg)));

        let line = Line::from(all_spans);
        let paragraph = Paragraph::new(line).block(Block::default().bg(bg));
        frame.render_widget(paragraph, area);
    }
}

fn shorten_uuid(id: Uuid) -> String {
    let s = id.to_string();
    format!("{}..{}", &s[..4], &s[s.len()-4..])
}

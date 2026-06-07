//! File explorer component — directory tree navigation and file selection.
use std::path::{Path, PathBuf};
use std::cell::RefCell;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
    prelude::Stylize,
};
use crate::tui_compat::{AppContext, Component, DrawContext, Event, EventResult};
use crate::theme::Theme;
use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
use crate::components::modals::InputModal;

/// File system explorer widget — navigates directories and selects files.
pub struct FileTree {
    /// The root directory being displayed.
    pub root: PathBuf,
    /// Sorted directory entries for the current root.
    pub entries: Vec<DirEntry>,
    /// Ratatui list state for selection tracking.
    pub state: RefCell<ListState>,
    /// Whether the file tree has keyboard focus.
    pub focused: bool,
    /// The current theme.
    pub theme: Theme,
    input_modal: InputModal,
    file_to_open: Option<PathBuf>,
}

/// A single entry in the file tree — file or directory.
#[derive(Clone, Debug)]
pub struct DirEntry {
    /// Display name of the entry.
    pub name: String,
    /// Full filesystem path.
    pub path: PathBuf,
    /// Whether this entry is a directory.
    pub is_dir: bool,
}

impl FileTree {
    /// Creates a new `FileTree` rooted at the current working directory.
    pub fn new() -> Self {
        let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let root = if let Ok(abs) = root.canonicalize() { abs } else { root };
        let mut tree = Self {
            root,
            entries: Vec::new(),
            state: RefCell::new(ListState::default()),
            focused: false,
            theme: Theme::default(),
            input_modal: InputModal::new("New Folder"),
            file_to_open: None,
        };
        tree.refresh();
        if !tree.entries.is_empty() {
            tree.state.borrow_mut().select(Some(0));
        }
        tree
    }

    /// Re-reads the filesystem for the current root directory.
    pub fn refresh(&mut self) {
        let mut entries = Vec::new();
        // Add parent directory link if not at filesystem root
        if let Some(parent) = self.root.parent() {
            entries.push(DirEntry { name: "..".to_string(), path: parent.to_path_buf(), is_dir: true });
        }
        if let Ok(read_dir) = std::fs::read_dir(&self.root) {
            let mut items: Vec<DirEntry> = read_dir.filter_map(|e| e.ok()).map(|e| DirEntry {
                name: e.file_name().to_string_lossy().to_string(),
                path: e.path(),
                is_dir: e.path().is_dir(),
            }).collect();
            items.sort_by(|a, b| if a.is_dir != b.is_dir { b.is_dir.cmp(&a.is_dir) } else { a.name.to_lowercase().cmp(&b.name.to_lowercase()) });
            entries.extend(items);
        }
        self.entries = entries;
    }

    /// Returns the full path of the currently selected entry.
    pub fn selected_file(&self) -> Option<PathBuf> {
        self.state.borrow().selected().and_then(|i| self.entries.get(i).map(|e| e.path.clone()))
    }

    pub fn take_file_to_open(&mut self) -> Option<PathBuf> {
        self.file_to_open.take()
    }

    /// Changes the root directory to `path` and refreshes the listing.
    pub fn change_dir(&mut self, path: &Path) -> std::io::Result<()> {
        let abs_path = if let Ok(abs) = path.canonicalize() { abs } else { path.to_path_buf() };
        self.root = abs_path;
        self.refresh();
        self.state.borrow_mut().select(Some(0));
        Ok(())
    }

    fn create_folder(&mut self, name: &str) -> std::io::Result<()> {
        let path = self.root.join(name);
        std::fs::create_dir_all(path)?;
        self.refresh();
        Ok(())
    }

    fn delete_selected(&mut self) -> std::io::Result<()> {
        if let Some(path) = self.selected_file() {
            // Safety: Never delete parent link or current root
            if path.file_name().map(|n| n == "..").unwrap_or(false) { return Ok(()); }
            if path == self.root { return Ok(()); }

            log::warn!("[EXPLORER] Deleting: {:?}", path);
            if path.is_dir() {
                std::fs::remove_dir_all(&path)?;
            } else {
                std::fs::remove_file(&path)?;
            }
            self.refresh();
        }
        Ok(())
    }

    /// Shows the input modal with the given title for creating a new folder.
    pub fn input_modal_show(&mut self, title: &str, _id: &str) {
        self.input_modal.modal.title = title.to_string();
        self.input_modal.show();
    }

    /// Retrieves the value entered in the input modal, if any.
    pub fn take_input_value(&mut self, _id: &str) -> Option<String> {
        self.input_modal.take_value()
    }

    fn move_up(&mut self) {
        let mut state = self.state.borrow_mut();
        let i = match state.selected() {
            Some(i) => if i == 0 { self.entries.len().saturating_sub(1) } else { i - 1 },
            None => 0,
        };
        state.select(Some(i));
    }

    fn move_down(&mut self) {
        let mut state = self.state.borrow_mut();
        let i = match state.selected() {
            Some(i) => if i >= self.entries.len().saturating_sub(1) { 0 } else { i + 1 },
            None => 0,
        };
        state.select(Some(i));
    }
}

impl Component for FileTree {
    fn handle_event(&mut self, event: &Event, ctx: &mut AppContext) -> EventResult {
        if self.input_modal.modal.active {
            let res = self.input_modal.handle_event(event, ctx);
            if let Some(name) = self.input_modal.take_value() { let _ = self.create_folder(&name); }
            return res;
        }
        if !self.focused { return EventResult::Unhandled; }
        if let Event::Key(key_event) = event {
            if key_event.kind != KeyEventKind::Press { return EventResult::Handled; }
            
            // DELETION HARDENING: Support plain Delete and Ctrl+D
            if key_event.code == KeyCode::Delete || (key_event.code == KeyCode::Char('d') && key_event.modifiers.contains(KeyModifiers::CONTROL)) {
                let _ = self.delete_selected();
                return EventResult::Handled;
            }

            match key_event.code {
                KeyCode::Up => { self.move_up(); EventResult::Handled }
                KeyCode::Down => { self.move_down(); EventResult::Handled }
                KeyCode::Enter => {
                    if let Some(path) = self.selected_file() {
                        if path.is_dir() {
                            let _ = self.change_dir(&path);
                        } else {
                            self.file_to_open = Some(path);
                        }
                    }
                    EventResult::Handled
                }
                KeyCode::Char('f') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.input_modal.show();
                    EventResult::Handled
                }
                KeyCode::Backspace => {
                    if let Some(parent) = self.root.parent() {
                        let _ = self.change_dir(&parent.to_path_buf());
                        return EventResult::Handled;
                    }
                    EventResult::Unhandled
                }
                _ => EventResult::Unhandled,
            }
        } else { EventResult::Unhandled }
    }

    fn draw(&self, frame: &mut Frame, area: Rect, ctx: &DrawContext) {
        let border_style = if self.focused { Style::default().fg(self.theme.border_active) } else { Style::default().fg(self.theme.border_inactive) };
        let block = Block::default().borders(Borders::ALL).title(format!(" Explorer: {} ", self.root.display())).border_style(border_style).bg(self.theme.surface);
        let items: Vec<ListItem> = self.entries.iter().map(|e| {
            let icon = if e.is_dir { "▸ " } else { "  " };
            let style = if e.is_dir { Style::default().fg(self.theme.border_active).add_modifier(Modifier::BOLD) } else { Style::default().fg(self.theme.fg) };
            ListItem::new(format!("{}{}", icon, e.name)).style(style)
        }).collect();
        let list = List::new(items).block(block).highlight_style(Style::default().bg(self.theme.surface_alt).fg(self.theme.accent).add_modifier(Modifier::BOLD)).highlight_symbol("▶ ");
        frame.render_stateful_widget(list, area, &mut self.state.borrow_mut());
        if self.input_modal.modal.active { self.input_modal.draw(frame, frame.area(), ctx); }
    }
}

//! Tab controller — manages multiple tabs, each containing a [`LayoutTree`].
use crate::components::layout::LayoutTree;
use crate::components::Editor;
use std::sync::{Arc, Mutex};
use telarex_core::buffer::ManagedBuffer as Document;
use std::path::PathBuf;

/// An open tab — has a name, optional file path, and a pane layout tree.
pub struct Tab {
    /// Display name of the tab.
    pub name: String,
    /// File path associated with this tab, if any.
    pub path: Option<PathBuf>,
    /// The layout tree containing one or more editor panes.
    pub layout: LayoutTree,
}

/// Manages a collection of tabs — creation, removal, navigation, and focus sync.
pub struct TabController {
    /// All open tabs.
    pub tabs: Vec<Tab>,
    /// Index of the currently active tab.
    pub active_tab: usize,
}

impl TabController {
    /// Creates a new `TabController` with a single initial tab containing the given editor.
    pub fn new(editor: Editor) -> Self {
        Self {
            tabs: vec![Tab {
                name: "Main".to_string(),
                path: None,
                layout: LayoutTree::new(editor),
            }],
            active_tab: 0,
        }
    }

    /// Returns a mutable reference to the active tab.
    pub fn active_tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.active_tab]
    }

    /// Returns a shared reference to the active tab.
    pub fn active_tab_ref(&self) -> &Tab {
        &self.tabs[self.active_tab]
    }

    /// Finds the index of a tab by its file path, if any.
    pub fn find_tab_by_path(&self, path: &PathBuf) -> Option<usize> {
        self.tabs.iter().position(|t| t.path.as_ref() == Some(path))
    }

    /// Adds a new tab for the given file path and document.
    pub fn add_tab(&mut self, path: PathBuf, doc: Arc<Mutex<Document>>) {
        let mut editor = Editor::new();
        editor.load_document(path.clone(), doc);
        
        let name = path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Untitled".to_string());

        self.tabs.push(Tab {
            name,
            path: Some(path),
            layout: LayoutTree::new(editor),
        });
        self.active_tab = self.tabs.len() - 1;
    }

    /// Creates a new empty tab and switches to it.
    pub fn new_tab(&mut self) {
        let editor = Editor::new();
        self.tabs.push(Tab {
            name: format!("Tab {}", self.tabs.len() + 1),
            path: None,
            layout: LayoutTree::new(editor),
        });
        self.active_tab = self.tabs.len() - 1;
    }

    /// Removes the active tab. If it is the last tab, it is kept.
    pub fn remove_active_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.tabs.remove(self.active_tab);
            if self.active_tab >= self.tabs.len() {
                self.active_tab = self.tabs.len() - 1;
            }
        }
    }

    /// Switches to the next tab (wrapping).
    pub fn next_tab(&mut self) {
        self.active_tab = (self.active_tab + 1) % self.tabs.len();
    }

    /// Switches to the previous tab (wrapping).
    pub fn prev_tab(&mut self) {
        if self.active_tab == 0 { self.active_tab = self.tabs.len() - 1; }
        else { self.active_tab -= 1; }
    }

    /// Syncs the focus state of all panes in the active tab.
    pub fn sync_focus(&mut self, focused: bool) {
        self.active_tab_mut().layout.sync_focus(focused);
    }
}

use crate::tui_compat::{AppContext, Component, DrawContext, Event, EventResult};
use ratatui::layout::Rect;
use ratatui::Frame;

impl Component for TabController {
    fn draw(&self, frame: &mut Frame, area: Rect, ctx: &DrawContext) {
        self.active_tab_ref().layout.draw(frame, area, ctx);
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut AppContext) -> EventResult {
        self.active_tab_mut().layout.handle_event(event, ctx)
    }
}

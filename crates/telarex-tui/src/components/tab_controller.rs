use crate::components::layout::LayoutTree;
use crate::components::Editor;
use std::sync::{Arc, Mutex};
use telarex_core::buffer::ManagedBuffer as Document;
use std::path::PathBuf;

pub struct Tab {
    pub name: String,
    pub path: Option<PathBuf>,
    pub layout: LayoutTree,
}

pub struct TabController {
    pub tabs: Vec<Tab>,
    pub active_tab: usize,
}

impl TabController {
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

    pub fn active_tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.active_tab]
    }

    pub fn active_tab_ref(&self) -> &Tab {
        &self.tabs[self.active_tab]
    }

    pub fn find_tab_by_path(&self, path: &PathBuf) -> Option<usize> {
        self.tabs.iter().position(|t| t.path.as_ref() == Some(path))
    }

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

    pub fn new_tab(&mut self) {
        let editor = Editor::new();
        self.tabs.push(Tab {
            name: format!("Tab {}", self.tabs.len() + 1),
            path: None,
            layout: LayoutTree::new(editor),
        });
        self.active_tab = self.tabs.len() - 1;
    }

    pub fn remove_active_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.tabs.remove(self.active_tab);
            if self.active_tab >= self.tabs.len() {
                self.active_tab = self.tabs.len() - 1;
            }
        }
    }

    pub fn next_tab(&mut self) {
        self.active_tab = (self.active_tab + 1) % self.tabs.len();
    }

    pub fn prev_tab(&mut self) {
        if self.active_tab == 0 { self.active_tab = self.tabs.len() - 1; }
        else { self.active_tab -= 1; }
    }

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

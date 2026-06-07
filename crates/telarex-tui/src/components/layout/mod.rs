use ratatui::layout::{Rect, Constraint, Direction, Layout};
use ratatui::style::Color;
use ratatui::widgets::Paragraph;
use ratatui::Frame;
use ratatui::prelude::Stylize;
use std::cell::RefCell;
use std::collections::HashMap;
use uuid::Uuid;
use crate::components::Editor;
use crate::tui_compat::{AppContext, Component, DrawContext, Event, EventResult};

#[derive(Debug)]
pub enum NodeKind {
    Pane(Editor),
    Split,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavDir { Left, Right, Up, Down }

pub struct LayoutNode {
    pub id: Uuid,
    pub kind: NodeKind,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
    pub split_ratio: f32, 
    pub direction: Direction,
}

pub struct LayoutTree {
    pub nodes: Vec<LayoutNode>,
    pub root: usize,
    pub active_pane: Uuid,
    pub last_area: RefCell<Rect>,
}

impl LayoutTree {
    pub fn new(editor: Editor) -> Self {
        let pane_id = Uuid::new_v4();
        let root_node = LayoutNode {
            id: pane_id,
            kind: NodeKind::Pane(editor),
            parent: None,
            children: Vec::new(),
            split_ratio: 0.5,
            direction: Direction::Horizontal,
        };
        
        Self {
            nodes: vec![root_node],
            root: 0,
            active_pane: pane_id,
            last_area: RefCell::new(Rect::default()),
        }
    }

    pub fn split_pane(&mut self, id: Uuid, direction: Direction) {
        let idx = self.nodes.iter().position(|n| n.id == id).expect("Node not found");
        let old_editor = if let NodeKind::Pane(editor) = std::mem::replace(&mut self.nodes[idx].kind, NodeKind::Split) {
            editor
        } else {
            return;
        };

        self.nodes[idx].direction = direction;
        
        let pane1_id = Uuid::new_v4();
        let pane2_id = Uuid::new_v4();
        
        let p1_idx = self.nodes.len();
        let pane1 = LayoutNode {
            id: pane1_id,
            kind: NodeKind::Pane(old_editor),
            parent: Some(idx),
            children: Vec::new(),
            split_ratio: 0.5,
            direction: Direction::Horizontal,
        };
        self.nodes.push(pane1);
        
        let p2_idx = self.nodes.len();
        let pane2 = LayoutNode {
            id: pane2_id,
            kind: NodeKind::Pane(Editor::new()),
            parent: Some(idx),
            children: Vec::new(),
            split_ratio: 0.5,
            direction: Direction::Horizontal,
        };
        self.nodes.push(pane2);
        
        self.nodes[idx].children = vec![p1_idx, p2_idx];
        self.active_pane = pane2_id;
    }

    pub fn close_pane(&mut self, id: Uuid) {
        let idx = if let Some(i) = self.nodes.iter().position(|n| n.id == id) { i } else { return };
        
        let pane_count = self.nodes.iter().filter(|n| matches!(n.kind, NodeKind::Pane(_))).count();
        if pane_count <= 1 { return; }

        let parent_idx = if let Some(p) = self.nodes[idx].parent { p } else { return };
        
        let sibling_idx = if self.nodes[parent_idx].children[0] == idx {
            self.nodes[parent_idx].children[1]
        } else {
            self.nodes[parent_idx].children[0]
        };

        let sibling_kind = std::mem::replace(&mut self.nodes[sibling_idx].kind, NodeKind::Split);
        let sibling_children = std::mem::replace(&mut self.nodes[sibling_idx].children, Vec::new());
        let sibling_ratio = self.nodes[sibling_idx].split_ratio;
        let sibling_dir = self.nodes[sibling_idx].direction;
        let sibling_id = self.nodes[sibling_idx].id;

        let parent = &mut self.nodes[parent_idx];
        parent.kind = sibling_kind;
        parent.children = sibling_children;
        parent.split_ratio = sibling_ratio;
        parent.direction = sibling_dir;
        parent.id = sibling_id;

        let new_children = parent.children.clone();
        for child_idx in new_children {
            self.nodes[child_idx].parent = Some(parent_idx);
        }

        self.nodes[idx].kind = NodeKind::Split;
        self.active_pane = self.find_first_pane_id(parent_idx);
    }

    fn find_first_pane_id(&self, idx: usize) -> Uuid {
        match &self.nodes[idx].kind {
            NodeKind::Pane(_) => self.nodes[idx].id,
            NodeKind::Split => self.find_first_pane_id(self.nodes[idx].children[0]),
        }
    }

    pub fn navigate(&mut self, direction: NavDir) {
        let active_id = self.active_pane;
        let area = *self.last_area.borrow();
        let rects = self.compute_rects(area);
        let active_rect = if let Some(r) = rects.get(&active_id) { *r } else { return; };
        
        let mut best_pane = None;
        let mut min_dist = f32::MAX;
        
        for node in &self.nodes {
            if let NodeKind::Pane(_) = node.kind {
                if node.id == active_id { continue; }
                if let Some(rect) = rects.get(&node.id) {
                    let is_in_direction = match direction {
                        NavDir::Left => rect.x + rect.width <= active_rect.x,
                        NavDir::Right => rect.x >= active_rect.x + active_rect.width,
                        NavDir::Up => rect.y + rect.height <= active_rect.y,
                        NavDir::Down => rect.y >= active_rect.y + active_rect.height,
                    };
                    
                    if is_in_direction {
                        let dx = (rect.x as f32 + rect.width as f32 / 2.0) - (active_rect.x as f32 + active_rect.width as f32 / 2.0);
                        let dy = (rect.y as f32 + rect.height as f32 / 2.0) - (active_rect.y as f32 + active_rect.height as f32 / 2.0);
                        let dist = (dx.powi(2) + dy.powi(2)).sqrt();
                        
                        if dist < min_dist {
                            min_dist = dist;
                            best_pane = Some(node.id);
                        }
                    }
                }
            }
        }
        
        if let Some(id) = best_pane {
            self.active_pane = id;
        }
    }

    pub fn sync_focus(&self, group_focused: bool) {
        for node in &self.nodes {
            if let NodeKind::Pane(editor) = &node.kind {
                if let Ok(mut focused) = editor.focused.try_borrow_mut() {
                    *focused = group_focused && node.id == self.active_pane;
                }
            }
        }
    }

    pub fn compute_rects(&self, area: Rect) -> HashMap<Uuid, Rect> {
        let mut map = HashMap::new();
        self.recurse_compute(self.root, area, &mut map);
        map
    }

    fn recurse_compute(&self, idx: usize, area: Rect, map: &mut HashMap<Uuid, Rect>) {
        let node = &self.nodes[idx];
        match node.kind {
            NodeKind::Pane(_) => { map.insert(node.id, area); }
            NodeKind::Split => {
                let ratio = node.split_ratio.clamp(0.1, 0.9);
                let chunks = Layout::default()
                    .direction(node.direction)
                    .constraints([
                        Constraint::Fill((ratio * 100.0) as u16),
                        Constraint::Length(1),
                        Constraint::Fill(((1.0 - ratio) * 100.0) as u16),
                    ])
                    .split(area);
                
                self.recurse_compute(node.children[0], chunks[0], map);
                self.recurse_compute(node.children[1], chunks[2], map);
            }
        }
    }

    pub fn handle_event(&mut self, event: &Event, ctx: &mut AppContext) -> EventResult {
        if let Event::Mouse(mouse) = event {
            if mouse.kind == crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left) {
                let area = *self.last_area.borrow();
                let rects = self.compute_rects(area);
                for (id, rect) in rects {
                    if rect.contains(ratatui::layout::Position::new(mouse.column, mouse.row)) {
                        self.active_pane = id;
                        self.sync_focus(true);
                        break;
                    }
                }
            }
        }

        for node in self.nodes.iter_mut() {
            if let NodeKind::Pane(ref mut editor) = node.kind {
                if node.id == self.active_pane {
                    let res = editor.handle_event(event, ctx);
                    if res.is_handled() { return res; }
                }
            }
        }
        EventResult::Unhandled
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect, ctx: &DrawContext) {
        *self.last_area.borrow_mut() = area;
        self.recurse_draw(self.root, area, frame, ctx);
    }

    fn recurse_draw(&self, idx: usize, area: Rect, frame: &mut Frame, ctx: &DrawContext) {
        let node = &self.nodes[idx];
        match &node.kind {
            NodeKind::Pane(editor) => {
                editor.draw(frame, area, ctx);
            }
            NodeKind::Split => {
                let ratio = node.split_ratio.clamp(0.1, 0.9);
                let chunks = Layout::default()
                    .direction(node.direction)
                    .constraints([
                        Constraint::Fill((ratio * 100.0) as u16),
                        Constraint::Length(1),
                        Constraint::Fill(((1.0 - ratio) * 100.0) as u16),
                    ])
                    .split(area);
                
                let [p1_area, divider_area, p2_area] = [chunks[0], chunks[1], chunks[2]];

                let divider = match node.direction {
                    Direction::Horizontal => "|",
                    Direction::Vertical => "-",
                };
                frame.render_widget(Paragraph::new(divider).fg(Color::DarkGray), divider_area);

                self.recurse_draw(node.children[0], p1_area, frame, ctx);
                self.recurse_draw(node.children[1], p2_area, frame, ctx);
            }
        }
    }
}

impl Component for LayoutTree {
    fn draw(&self, frame: &mut Frame, area: Rect, ctx: &DrawContext) {
        self.recurse_draw(self.root, area, frame, ctx);
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut AppContext) -> EventResult {
        let pane_idx = self.nodes.iter().position(|n| n.id == self.active_pane);
        if let Some(idx) = pane_idx {
            if let NodeKind::Pane(ref mut editor) = self.nodes[idx].kind {
                return editor.handle_event(event, ctx);
            }
        }
        EventResult::Unhandled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn single_pane_tree() -> LayoutTree {
        LayoutTree::new(Editor::new())
    }

    fn split_tree() -> LayoutTree {
        let mut tree = LayoutTree::new(Editor::new());
        let id = tree.active_pane;
        tree.split_pane(id, Direction::Horizontal);
        tree
    }

    #[test]
    fn test_new_tree_has_one_pane() {
        let tree = single_pane_tree();
        assert_eq!(tree.nodes.len(), 1);
        assert!(matches!(tree.nodes[0].kind, NodeKind::Pane(_)));
    }

    #[test]
    fn test_split_pane_creates_two_children() {
        let mut tree = single_pane_tree();
        let id = tree.active_pane;
        tree.split_pane(id, Direction::Horizontal);

        assert_eq!(tree.nodes.len(), 3);
        assert!(matches!(tree.nodes[0].kind, NodeKind::Split));
        assert_eq!(tree.nodes[0].children.len(), 2);
    }

    #[test]
    fn test_split_pane_sets_active_to_new_pane() {
        let mut tree = single_pane_tree();
        let original = tree.active_pane;
        tree.split_pane(original, Direction::Horizontal);

        let last_idx = tree.nodes.len() - 1;
        assert_eq!(tree.active_pane, tree.nodes[last_idx].id);
    }

    #[test]
    fn test_close_pane_with_single_pane_noop() {
        let mut tree = single_pane_tree();
        let id = tree.active_pane;
        tree.close_pane(id);

        assert_eq!(tree.nodes.len(), 1);
    }

    #[test]
    fn test_navigate_between_split_panes() {
        let mut tree = split_tree();
        let left_id = tree.active_pane;

        tree.navigate(NavDir::Right);
        let right_id = tree.active_pane;
        assert_ne!(left_id, right_id);

        tree.navigate(NavDir::Left);
        assert_eq!(tree.active_pane, left_id);
    }

    #[test]
    fn test_sync_focus_activates_only_active_pane() {
        let mut tree = split_tree();
        let original = tree.active_pane;

        tree.split_pane(original, Direction::Horizontal);
        let new_id = tree.active_pane;

        tree.sync_focus(true);

        for node in &tree.nodes {
            if let NodeKind::Pane(editor) = &node.kind {
                let focused = *editor.focused.borrow();
                if node.id == new_id {
                    assert!(focused, "active pane should be focused");
                } else {
                    assert!(!focused, "inactive pane should not be focused");
                }
            }
        }
    }

    #[test]
    fn test_sync_focus_unfocused_when_group_unfocused() {
        let tree = split_tree();
        tree.sync_focus(false);

        for node in &tree.nodes {
            if let NodeKind::Pane(editor) = &node.kind {
                assert!(!*editor.focused.borrow());
            }
        }
    }

    #[test]
    fn test_compute_rects_returns_all_panes() {
        let tree = split_tree();
        let area = Rect::new(0, 0, 100, 50);
        let rects = tree.compute_rects(area);

        let pane_count = tree.nodes.iter().filter(|n| matches!(n.kind, NodeKind::Pane(_))).count();
        assert_eq!(rects.len(), pane_count);
    }

    #[test]
    fn test_navigate_noop_when_single_pane() {
        let mut tree = single_pane_tree();
        let id = tree.active_pane;

        tree.navigate(NavDir::Left);
        tree.navigate(NavDir::Right);
        tree.navigate(NavDir::Up);
        tree.navigate(NavDir::Down);

        assert_eq!(tree.active_pane, id);
    }

    #[test]
    fn test_close_pane_merges_into_parent() {
        let mut tree = split_tree();
        let right_id = tree.active_pane;

        tree.navigate(NavDir::Left);
        let left_id = tree.active_pane;

        tree.close_pane(right_id);
        assert!(tree.nodes.iter().any(|n| n.id == left_id));
        let pane_count = tree.nodes.iter().filter(|n| matches!(n.kind, NodeKind::Pane(_))).count();
        assert_eq!(pane_count, 1);
    }
}

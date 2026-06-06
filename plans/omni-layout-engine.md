# Plan: The Omni-Layout Split Engine

We are moving from a single `Editor` pane to a recursive `LayoutTree`. This engine manages pane split-ratios, focus routing, and deterministic rendering.

## 1. Architectural Model: The Arena Tree
Instead of recursive structs, we use a `LayoutTree` (flat `Vec<LayoutNode>`). This is memory-efficient and prevents borrow checker conflicts.

```rust
pub struct LayoutNode {
    pub id: Uuid,
    pub kind: NodeKind,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
    pub split_ratio: f32, // 0.0 - 1.0 (divider position)
    pub direction: Direction, // Horizontal or Vertical
}

pub enum NodeKind {
    Pane(Editor),
    Split,
}
```

## 2. Key Workflow
1.  **Split Trigger**: `EditorView` intercepts `Ctrl+W v` or `Ctrl+W s`.
2.  **Tree Mutation**: 
    - The current `Pane` node is turned into a `Split` node.
    - Two new `Pane` children are added to the `Arena`.
    - `LayoutEngine` recalculates `Rect`s for the new tree.
3.  **Spatial Routing**: Mouse/Key events are intercepted by `EditorView`, which queries the `LayoutEngine` to determine the active `PaneID` based on the current `Rect` configuration.

## 3. Implementation Phasing
### Phase A: Core Engine
- Create `crates/telarex-tui/src/components/layout/mod.rs` with `LayoutNode`, `LayoutTree`, and `Direction`.
- Implement `LayoutTree::split_pane(id, direction)` and `LayoutTree::compute_rects(area)`.

### Phase B: Integration
- Refactor `EditorView` to hold a `LayoutTree` instead of a single `Editor`.
- Update `draw` to recursively render the `LayoutTree`.
- Implement `Navigation` (`Ctrl+W h/j/k/l`).

### Phase C: Sync & Polish
- Ensure the `SyncEngine` and `StatusBar` interact with the *currently focused* pane.
- Add TachyonFX "Slide-in" animation for new panes.

## 4. Safety & Stability (The "Guardrails")
- **Max Depth**: Hard limit to 8 split-levels.
- **Panic Protection**: Every split-ratio computation is sanitized (clamp 0.0 - 1.0).
- **Test Suite**: Add `test_split_integrity` to verify leaf nodes never overlap after a split.

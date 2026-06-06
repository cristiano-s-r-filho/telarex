# Track: Editor Core Evolution (Splits & Motions)

This track focuses on elevating the TelaRex editor core from a basic text buffer to a professional-grade "daily driver" with multi-pane support and advanced movement.

## Objectives
1.  **Multi-Pane Splits**: Implement vertical and horizontal splits (`Ctrl+W v`, `Ctrl+W s`).
2.  **Vim-Lite Motions**: Implement standard word-based motions (`w`, `b`, `e`) and paragraph jumps.
3.  **Visual Mode**: Standardize selection into a formal "Visual Mode" with block selection support.

## Architectural Design

### 1. The Container Model (Splits)
We need to move from a single `Editor` in `EditorView` to a `LayoutNode` tree:
```rust
enum LayoutNode {
    Leaf(Editor),
    Vertical(Vec<LayoutNode>),
    Horizontal(Vec<LayoutNode>),
}
```
This allows recursive splitting. `EditorView` will track the "Active Leaf" for focus.

### 2. Motion Engine
Move movement logic into `telarex-core/src/buffer/motions.rs`. 
- `find_word_start(rope, pos)`
- `find_word_end(rope, pos)`
- `find_paragraph_jump(rope, pos, direction)`

### 3. Key Grammar
Introduce a `CommandInterpreter` that handles multi-key sequences:
- `d` + `w` = Delete Word.
- `y` + `i` + `w` = Yank Inside Word.

## Phased Implementation

### Phase 1: Core Motions & Selection
- Implement word-wise movement logic in `telarex-core`.
- Update `Editor` to use these motions.
- Formalize "Visual Mode" visuals.

### Phase 2: Split Infrastructure
- Refactor `EditorView` to support multiple editor instances.
- Implement "Focus Switch" logic between panes.

### Phase 3: Composite Commands
- Implement the grammar engine for `Verb + Object` commands.

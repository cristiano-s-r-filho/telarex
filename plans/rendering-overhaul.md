# Plan: Absolute Rendering Reconstruction

This plan describes a total rewrite of the TelaRex rendering pipeline from scratch to eliminate the persistent ladder artifacts. We will build the UI stage-by-stage, verifying stability at each step.

## 1. Design Principles
- **No Manual Math**: Zero arithmetic on `area.x` or `area.y`.
- **Absolute Partitioning**: Use `ratatui::Layout` to divide areas into final, absolute `Rect`s before rendering.
- **Control Character Purge**: Global utility to strip `\n`, `\r`, and all non-visible control characters from every string.
- **No Floating Windows**: All elements (except transient modals) must occupy a discrete, non-overlapping segment of the root layout.

## 2. Implementation Stages

### Stage 1: The Root Canvas (`App` & `LayoutEngine`)
- Reset `TelaRexLayoutEngine` to a simple vertical split (TabBar, Workspace, StatusBar).
- Update `App::draw` to perform a "Nuclear Clear": Fill the terminal with the background color before any component draws.

### Stage 2: WelcomeView (Static Reconstruction)
- Re-implement `WelcomeView::draw` using only simple `Layout::vertical` and `Layout::horizontal` calls.
- No `Flex` alignments or `areas()` usage; use stable `split()` only.
- Strict ASCII-only labels.

### Stage 3: ConfigView (Structural Reconstruction)
- Re-implement `ConfigView` (and `ConfigModal`) with a fixed 30/70 horizontal split.
- Use a single `Paragraph` for each pane to ensure internal coordinate consistency.

### Stage 4: Popups (Modal Reconstruction)
- Re-implement `InputModal` and `AutocompletePopup`.
- Use `ratatui::widgets::Clear` but anchor them to a central layout calculated from the *root* area, not a relative sub-area.

### Stage 5: Single Editor (Core Reconstruction)
- Re-implement `Editor::draw`.
- Split into exactly two horizontal constraints (Gutter, Content).
- Strip every line of text explicitly in the draw loop.

### Stage 6: LayoutTree (Recursive Reconstruction)
- Re-integrate the `LayoutTree`.
- Ensure the recursion passes absolute, pre-clipped `Rect`s to each Editor instance.
- Simplify dividers to a single character line.

## 3. Global String Sanitizer
We will add a helper to `utils/mod.rs` that MUST be used by all components:
```rust
pub fn sanitize(s: &str) -> String {
    s.chars().filter(|c| !c.is_control() && *c != '\n' && *c != '\r').collect()
}
```

## 4. Verification Plan
After each stage, we will ask the user to verify the rendering.
We will not proceed to the next stage until the current one is confirmed pixel-perfect.

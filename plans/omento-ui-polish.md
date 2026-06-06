# Plan: Bento UI Polish & Omni-Alignment

This plan refines the "Absolute Rendering" foundation into a professional "Bento" design, using segmented layouts, consistent spacers, and high-stability Unicode symbols.

## 1. Objectives
- **Bento Segmentation**: Divide screens into visually distinct "boxes" (Bento cells) with consistent margins and background shifts.
- **Stable Pill Design**: Re-introduce high-aesthetic pill delimiters (``, ``) with mandatory visual-width guards.
- **Center-Start Alignment**: Ensure that while elements feel centered, they follow a "Center-Start" rule (centered container, left-aligned content) to prevent jitter.

## 2. Implementation Steps

### Phase 1: Global Aesthetic Helpers (`utils/mod.rs`)
- Implement a `draw_bento_box` helper that renders a Block with specific margins and theme-aware borders.
- Implement a `pill_spans` helper that returns a `Vec<Span>` with safe Unicode delimiters, ensuring they are always treated as 1-column wide.

### Phase 2: Welcome View "Hero" Alignment
- Refactor the Welcome screen to use a 3x3 Bento grid.
- The "Hero" (ASCII Art) will occupy the top-center cell.
- The "Menu" will occupy the bottom-center cell.
- Add a "Tips/Stats" cell on the right if space permits.

### Phase 3: Status Bar "Bento" Overhaul
- Refactor the Status Bar into a single line of adjacent "Pills".
- Use background color shifts (`bg(mode_color).fg(bg_color)`) to create the segmented Bento look without relying on character offsets.

### Phase 4: Editor Content Alignment
- Ensure the Gutter is perfectly vertical and doesn't "leak" into the content area.
- Use `ratatui::widgets::Block` with `Borders::RIGHT` for the gutter divider to ensure a pixel-perfect vertical line.

## 3. Visual Width Guard
We will use `unicode-width` to strictly validate every non-ASCII character in the Bento UI. If a character reports a width other than 1, we will wrap it in a spacer or use an ASCII fallback to maintain coordinate integrity.

## 4. Verification
- **Visual Audit**: No lines should "jump" or "ladder".
- **Resize Robustness**: Segments must scale proportionally without overlapping.

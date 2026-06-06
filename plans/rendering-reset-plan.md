# Plan: Global Rendering Architecture Reset

This plan describes a complete migration of the TelaRex rendering pipeline to a unified, declarative architecture. We will eliminate fragmented component-level coordinate calculations and adopt a "Top-Down Absolute Rendering" model.

## 1. Problem Definition
The current "ladder" artifact stems from inconsistent coordinate origins where each component makes local assumptions about its area, leading to additive offset errors.

## 2. Proposed Architecture: Unified Rendering
- **Main Loop**: The `App` will now own a single `Frame` context and execute a strictly ordered rendering pass.
- **Declarative Layouts**: Replace all manual `Rect` adjustments (e.g., `x + offset`) with `ratatui::layout` constraints exclusively.
- **Absolute Coordinate Mapping**: All components will receive an absolute `Rect` from the root level and must not perform local coordinate transformations without explicitly declaring them in the `Layout` constraint.

## 3. Implementation Phases

### Phase 1: Global Layout Engine
- Centralize all `Layout` logic in `crates/telarex-tui/src/render/engine.rs`.
- Define a `TelaRexLayout` struct that computes a single, deterministic `AreaMap` for the entire screen.

### Phase 2: Screen Redesign
- Redesign `WelcomeView`, `EditorView`, and `ConfigView` to work with the new `Engine`.
- Components will no longer implement `Component::draw` taking a `Rect`. They will implement a `Renderable` trait that takes a context describing their absolute bounds.

### Phase 3: LayoutTree Integration
- The `LayoutTree` will be refactored to register its `Editor` panes within the `AreaMap` during the engine's layout pass.

### Phase 4: Verification
- Verify that every element (Editor, Status Bar, Tab Bar) is anchored to the global engine, eliminating the possibility of local coordinate drift.

## 4. Verification Plan
- **Resize Stress Test**: Rapidly resize the terminal to ensure no coordinate jitter.
- **Visual Audit**: Verify the layout is pixel-perfect and laddering is gone.

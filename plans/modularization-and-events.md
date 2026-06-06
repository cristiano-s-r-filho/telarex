# Plan: Modularization and Hierarchical Event Handling

This plan aims to refactor the TelaRex TUI architecture to improve maintainability, fix persistent input bugs, and establish a robust hierarchical event system.

## Objectives
1.  **Modularization**: Decouple event logic from component rendering.
2.  **Hierarchical Routing**: Implement a top-down event propagation system.
3.  **Input Bug Resolution**: Ensure character input (like `//`) is never intercepted by hidden logic.
4.  **Clean State Management**: Use a shared context for themes and configurations.

## Proposed Changes

### 1. New Event System (`crates/telarex-tui/src/events/`)
-   **`Action`**: Enum representing high-level user intents (e.g., `SaveFile`, `ToggleExplorer`, `EnterCommandMode`).
-   **`Dispatcher`**: Logic to map raw key events to `Action`s based on the current context.

### 2. State & Context (`crates/telarex-tui/src/context.rs`)
-   **`UIContext`**: A struct containing the current `Theme`, `Config`, and a way to emit `Action`s.

### 3. Screen Refactoring
-   **`WelcomeView`**: Use the new `Action` system for menu navigation.
-   **`EditorView`**: Centralize event routing between the Explorer, Editor, and Palettes. Ensure that if an event is not an `Action`, it is passed directly to the focused component's raw input handler.

### 4. Component Refactoring
-   **`Editor`**: Simplify `handle_event` to focus primarily on text manipulation.
-   **`SearchPalette` & `CommandPalette`**: Improve responsiveness and selection logic.

## Implementation Steps

### Phase 1: Foundation
1.  Create `crates/telarex-tui/src/events/mod.rs` and `crates/telarex-tui/src/events/actions.rs`.
2.  Implement `UIAction` enum.

### Phase 2: Hierarchical Routing
1.  Modify `App::handle_event` to delegate to a dedicated dispatcher.
2.  Refactor `EditorView::handle_event` to separate "Global Editor Shortcuts" from "Active Component Input".

### Phase 3: Bug Fixes & Verification
1.  Trace the `/` character through the new routing system to ensure it reaches `Editor::insert_char`.
2.  Verify `SearchPalette` results visibility and selection.
3.  Ensure `StatusBar` updates are triggered by `Action` completion.

## Verification Plan
1.  **Build**: Run `cargo check` after each phase.
2.  **Manual Test**: 
    - Type `//` in a Rust file.
    - Use `Ctrl+F` to search and `Enter` to select a result.
    - Switch themes and verify all UI elements update immediately.

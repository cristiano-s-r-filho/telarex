# Implementation Plan: Restoration & Async Hardening

## Objective
To resolve critical rendering and input failures in code documents, implement missing LodgeNet connectivity features (Join by ID, Self-Presence), and refactor the architecture to use asynchronous background tasks for performance-intensive operations like syntax highlighting.

## Background & Motivation
The current "editing hell" is caused by synchronous tree-sitter highlighting that performs a full document copy and re-parse every frame. Additionally, a line-count mismatch in the highlighter causes the last lines of code documents to become invisible, appearing as "blocked input" for characters like `/` and `?`. LodgeNet status is currently non-reactive, and users cannot join lodges by their ID.

## Key Files & Context
- `crates/telarex-core/src/syntax/tree_highlighter.rs`: Root of the line-count mismatch bug.
- `crates/telarex-tui/src/components/editor/mod.rs`: Needs fail-safe rendering and background highlight integration.
- `crates/telarex-tui/src/app.rs`: Needs optimistic status updates and Join-by-ID orchestration.
- `crates/telarex-tui/src/screens/welcome_view.rs`: Needs UI for Join-by-ID.
- `crates/telarex-tui/src/events/actions.rs`: Needs new actions for the async flow.

## Phased Implementation Plan

### Phase 1: Robust Rendering & Input Restoration
1.  **Highlighter Hardening**: Update `highlight_rope` in `telarex-core` to correctly handle trailing newlines and return a `Vec<Line>` matching the `Rope` length.
2.  **Fail-Safe Draw**: Modify `Editor::draw` to fallback to raw `Rope` lines if highlight data is out-of-sync or missing. This prevents characters from "disappearing."
3.  **Visual Column Preservation**: Ensure `preferred_visual_col` is correctly utilized to prevent cursor drift in all file types.

### Phase 2: LodgeNet 2.0 (Connectivity & Presence)
1.  **Join by ID Flow**:
    - Add `Join by ID` option to `WelcomeView`.
    - Implement `JoinByIdModal` to capture UUID.
    - Update `App::handle_event` to initiate network join.
2.  **Self-Presence & Optimistic UI**:
    - Update `status_bar` to show `1` peer immediately upon sharing.
    - Set status to `Online` as soon as the join/share command is issued.
3.  **Lodge Registry**: Implement a `HashMap<Uuid, LodgeMetadata>` in `App` to track all connected lodges.

### Phase 3: Async Performance Refactor
1.  **Background Highlighting**:
    - Add a `mpsc` channel to `Editor` to receive highlight updates.
    - Spawn a tokio task to run `TreeHighlighter` off-thread.
    - Update the draw loop to use the latest available highlights.
2.  **Tokio-Native IO**: Refactor `Document::save` and `Document::load` to use `tokio::fs` for non-blocking disk access.

### Phase 4: Activation of Dead Features
1.  **Autocomplete Integration**: Wire `AutocompletePopup` into `EditorView::handle_event` and `EditorView::draw`.

## Verification & Testing
- **Visual Check**: Type `/` and `?` at the end of a Rust file; confirm they are visible.
- **Performance Check**: Open a 1000+ line file and verify no "stutter" during rapid typing.
- **Networking Check**: Join a lodge using a UUID; verify the status bar shows `LODGE:Online (1)`.
- **Integrity Check**: Ensure `cargo check` remains at 0 warnings.

# Plan: Macro System & LSP Synchronization

This phase implements the Macro System for recording and replaying tasks, and ensures the LSP is synchronized with buffer changes for accurate autocompletion.

## Objectives
1.  **LSP Sync**: Implement `textDocument/didChange` notifications in `EditorView`.
2.  **Macro Recording**: Capture a sequence of key events and store them under a name.
3.  **Macro Playback**: Inject recorded events back into the event loop.
4.  **Macro Management**: UI to Save, Reuse, and Erase macros.

## Proposed Changes

### 1. LSP Synchronization (`telarex-tui/src/screens/editor_view.rs`)
-   Add `doc_version: i32` to tracking state.
-   In `handle_event`, if the editor handled an insertion or deletion, call `client.did_change`.

### 2. Macro Core (`telarex-core/src/buffer/macro.rs`)
-   Define `Macro` struct: `name: String`, `events: Vec<crossterm::event::KeyEvent>`.
-   Implement `MacroStore`: Load/Save to `macros.toml`.

### 3. Macro UI (`telarex-tui/src/components/modals/macro_palette.rs`)
-   A popup similar to the Command Palette for managing macros.
-   Actions: `Record`, `Play`, `Delete`, `Rename`.

### 4. Integration (`telarex-tui/src/screens/editor_view.rs`)
-   Add `macro_state` to `EditorView`.
-   Intercept and record events when state is `Recording`.
-   Recursive playback logic (loop through events and call `handle_event`).

## Implementation Steps

### Phase 1: LSP Reliability
1.  Update `EditorView` to send `didChange` on text entry.
2.  Verify autocomplete results improve with context.

### Phase 2: Macro Foundation
1.  Implement `Macro` and `MacroStore` in `telarex-core`.
2.  Add `UIAction::ToggleMacroPalette`.

### Phase 3: Recording & Playback
1.  Implement recording logic in `EditorView`.
2.  Implement playback by iterating over recorded events.

### Phase 4: UI Polish
1.  Create `MacroPalette`.
2.  Add status bar indicator "⏺ REC" when recording.

## Verification Plan
1.  **Sync**: Type `let x = 10;` then `x.` and see if LSP suggests methods for integer.
2.  **Macro**: Record "Insert 'hello', Save, Next Tab", then replay it and verify all actions occurred.
3.  **Persistence**: Restart the editor and verify the macro is still available.

# Plan: LSP-Powered Autocomplete

This phase implements the first item on the roadmap: a robust, asynchronous autocomplete system powered by Language Servers (LSP).

## Objectives
1.  **Request Completion**: Trigger `textDocument/completion` requests based on editor input.
2.  **Suggestion UI**: Display an asynchronous, scrollable popup with completion items.
3.  **Completion Application**: Insert selected suggestions into the buffer, handling partial word replacement.

## Proposed Changes

### 1. LSP Enhancements (`telarex-core/src/lsp/mod.rs`)
-   Define `CompletionItem` and `CompletionList` structures.
-   Implement `request_completion(uri, line, character)` in `LspClient`.
-   Handle response ID mapping to ensure responses are matched to the correct request.

### 2. Autocomplete Component (`telarex-tui/src/components/modals/autocomplete.rs`)
-   Create `AutocompletePopup` component.
-   Support navigation (Up/Down), selection (Enter), and cancellation (Esc).
-   Include icons for different completion types (Function, Variable, Keyword).

### 3. Editor Integration (`telarex-tui/src/components/editor.rs`)
-   Add `insert_completion(text)` method to handle replacing the currently typed prefix.
-   Emit a "trigger completion" signal when certain characters (like `.`, `:`, or after 3 letters) are typed.

### 4. Orchestration (`telarex-tui/src/screens/editor_view.rs`)
-   Update `poll_lsp` to parse `CompletionList` responses.
-   Manage the `AutocompletePopup` visibility and state.
-   Bridge the selection from the popup back to the editor.

## Implementation Steps

### Phase 1: Core LSP Logic
1.  Extend `LspClient` in `telarex-core` with completion request methods.
2.  Add JSON-RPC response tracking.

### Phase 2: UI Component
1.  Create `crates/telarex-tui/src/components/modals/autocomplete.rs`.
2.  Register it in `modals/mod.rs`.

### Phase 3: Wiring
1.  Update `EditorView` to trigger searches and display the results.
2.  Implement the buffer manipulation logic in `Editor` to apply the completion.

## Verification Plan
1.  **Manual Test**: Open a Rust file, type `std::`, and verify the completion popup appears with valid modules/functions.
2.  **Stability**: Ensure the editor remains responsive even if the LSP server is slow or crashes.
3.  **UI Consistency**: Verify the popup uses colors from the `StyleSheet`.

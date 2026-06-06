# Plan: Next-Gen Syntax & Intelligence (Tree-sitter & LSP)

This phase focuses on upgrading TelaRex from "basic editor" to "intelligent workspace" by adopting 2026 industry standards.

## Objectives
1.  **Structural Syntax**: Transition from Syntect (Regex) to Tree-sitter (Incremental Parsing).
2.  **LSP Foundation**: Implement a robust JSON-RPC client to communicate with Language Servers.
3.  **Semantic Theming**: Fully map the new `StyleSheet` system to Tree-sitter query captures.

## Proposed Changes

### 1. Tree-sitter Integration (`telarex-core`)
-   Add `tree-sitter`, `tree-sitter-highlight`, and specific grammar crates (e.g., `tree-sitter-rust`).
-   Create `TreeHighlighter` which uses structural queries instead of regex patterns.
-   Map captures like `function.method`, `variable.parameter`, `keyword.control` to `StyleSheet` tokens.

### 2. LSP Orchestrator (`telarex-core/src/lsp/`)
-   Implement `LspClient`: An async client for managing subprocesses (like `rust-analyzer`).
-   Handle basic JSON-RPC messaging (Initialize, Open, Change).

### 3. Integrated UI Refinement (`telarex-tui`)
-   Update `Editor` to consume the new `TreeHighlighter`.
-   Add an "Intelligence" status indicator to the Status Bar (e.g., showing if LSP is connected/indexing).

## Implementation Steps

### Phase 1: Tree-sitter Foundation
1.  Update `Cargo.toml` with Tree-sitter dependencies.
2.  Implement `crates/telarex-core/src/syntax/tree_highlighter.rs`.
3.  Modify `Editor` to use `TreeHighlighter` for supported languages, falling back to `Syntect` or plain text.

### Phase 2: Theme Mapping
1.  Extend `StyleSheet` schema to include standard Tree-sitter scopes.
2.  Update `Theme::from_stylesheet` to handle these extended tokens.

### Phase 3: LSP Kickstart
1.  Create `crates/telarex-core/src/lsp/mod.rs`.
2.  Implement process spawning and basic initialization handshake.

## Verification Plan
1.  **Syntax Accuracy**: Verify that complex Rust code (macros, lifetimes) is correctly highlighted structure-by-structure.
2.  **Performance**: Ensure horizontal scrolling and typing remains fluid with structural parsing.
3.  **Stability**: Run `cargo check` to ensure C-bindings and parsers are correctly linked.

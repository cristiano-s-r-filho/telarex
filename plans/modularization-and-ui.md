# Plan: Modularization, Layered Keymapping, and UI/UX Overhaul

This plan addresses architectural weaknesses in TelaRex, focusing on modular configuration, a modern "Bento" Status Bar, and robust P2P document synchronization.

## Objectives
1.  **Modular Configuration**: Transition to TOML-based config for Keymaps and Themes.
2.  **Layered Keymapping**: Implement a hierarchical resolution system (Component -> Mode -> Global).
3.  **Modern UI/UX**: Redesign the Status Bar using segmented "pills" and integrate TachyonFX for responsiveness.
4.  **Robust P2P Sync**: Fix the Automerge/libp2p bridge to ensure changes are propagated and merged correctly.
5.  **Standardized Error Handling**: Define a unified `TrexError` system with clear descriptions and solutions.

## Key Files & Context
- `telarex-core/src/config/`: Config manager and TOML schema.
- `telarex-tui/src/events/`: Refactor `actions.rs` and `mod.rs` for layered mapping.
- `telarex-tui/src/components/status_bar.rs`: Complete visual redesign.
- `telarex-core/src/crdt/sync_engine.rs`: Fix binary sync reception logic.

## Implementation Steps

### Phase 1: Configuration & Theming
1.  **TOML Integration**: Use `toml` and `serde` to load complex configurations.
2.  **CSS-like Stylesheet**: Define a semantic `StyleSheet` (e.g., `ui.border`, `editor.selection`) that can be loaded from a theme file.
3.  **App-wide Theming**: Ensure all components (Welcome, Config, FileTree) consume the same global theme.

### Phase 2: Layered Keymapping
1.  **Action Registry**: Map string identifiers in TOML to `UIAction` enum variants.
2.  **Resolution Engine**:
    - Check if the current component handles the key.
    - Check if the current editor mode (Insert/Normal) handles the key.
    - Fallback to global shortcuts.
3.  **Sequence Support**: Add a buffer to support multi-key chords (e.g., `g g`).

### Phase 3: "Bento" Status Bar & UI Polish
1.  **Visual Redesign**: Use segmented cards with rounded separators (``, ``).
2.  **Interactive Elements**: Color-code segments based on state (e.g., Red for recording, Green for active LSP).
3.  **Containment**: Ensure the bottom bar feels like a distinct section, not just floating text.

### Phase 4: P2P Sync & Reliability
1.  **Sync Bridge**: Fix `poll_network` in `App.rs` to correctly pass incoming CRDT patches to the `SyncEngine`.
2.  **Automatic Convergence**: Ensure that when a patch is received, the TUI buffer is updated without user input (Responsivity).
3.  **Lodge Discovery Fix**: Ensure "Share Workspace" correctly registers and broadcasts to the Gossipsub topic.

### Phase 5: Standardized Error Handling
1.  **Error Code System**: Implement `TRX-001`, `TRX-002` style codes.
2.  **Contextual Solutions**: Every error modal should display a "Recommended Action."

## Verification & Testing
1.  **Config Switch**: Swap themes via TOML and verify the entire UI updates.
2.  **Multi-Instance Sync**: Run two instances with different sessions; verify text syncs instantly.
3.  **Keymap Override**: Map `Ctrl-S` to something else in TOML and verify the change.

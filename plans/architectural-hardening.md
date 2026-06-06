# Plan: Architectural Hardening & Phased Feature Rollout

This plan describes the phased re-introduction of advanced TelaRex features (Bento UI, Macros, LodgeNet) while hardening the underlying architecture and maintaining absolute rendering stability.

## 1. Objectives
- **Protocol Separation**: Move LodgeNet (libp2p/Gossipsub) logic from `telarex-tui` to `telarex-core`.
- **Bento UI Restoration**: Re-introduce the segmented "pill" design and ASCII Word Art with strict width-validation to prevent rendering artifacts.
- **System Polish**: Harden the Macro System (playback logic) and Focus System (safe state synchronization).
- **Robustness**: Implement a centralized Error Registry and ensure logs are strictly file-bound (never stdout/stderr).
- **Granular Configuration**: Fully expand the TUI configuration panel for all editor, network, and theme settings.

## 2. Implementation Phases

### Phase 1: Core Protocol Migration (`LodgeNet`)
- **Move**: Migrate `crates/telarex-tui/src/network/` to `crates/telarex-core/src/network/`.
- **Refactor**: Expose a clean, async-trait based `NetworkClient` for the TUI.
- **Isolate**: Ensure the TUI only deals with `NetworkEvent` and `NetworkCommand`, with zero knowledge of libp2p internals.

### Phase 2: Bento UI & Visual Polish
- **Pills**: Re-introduce `` and `` in `status_bar.rs` using a `pill(content, color)` helper that handles absolute width.
- **Word Art**: Re-introduce the ASCII banner in `welcome_view.rs` within a strictly bounded `Paragraph`.
- **Symbols**: Re-introduce Unicode Math symbols (e.g., `󰚩`, `󰔎`, `󰚩`) in the UI, ensuring they are correctly stripped of control characters by the global `sanitize` utility.

### Phase 3: Macro & Focus Hardening
- **Macro Playback**: Refactor `EditorView` to replay macros using the internal `UIAction` enum rather than string-matching debug representations.
- **Focus Sync**: Update `LayoutTree` to explicitly notify all editor instances of their focus state whenever the `active_pane` changes, preventing `RefCell` drift.

### Phase 4: Error Registry & Logging
- **Error Registry**: Create `crates/telarex-core/src/errors/` with a `TrexError` enum and unique codes (e.g., `TRX-N01` for Network Join Failure).
- **Log Enforcer**: Add a global panic hook that redirects crashes to `trex.log`, ensuring the terminal is never corrupted by unexpected output.

### Phase 5: Granular Configuration Expansion
- **Categories**: Complete the `ConfigModal` implementation for:
  - **Editor**: Tab size, Auto-save, Line wrapping, Relative numbers.
  - **Network**: Bootstrap nodes, Peer ID, Gossipsub topics.
  - **Theme**: Active theme, custom color overrides.
  - **Keymaps**: List of current keybindings (read-only first).

## 3. Verification Plan
- **Integration Tests**: Run the core networking tests after migration.
- **Visual Audit**: After Phase 2, verify that the Bento elements do not re-introduce the "ladder" effect on any screen.
- **Stability Test**: Rapidly switch between screens and split/close panes to ensure focus and layout integrity.

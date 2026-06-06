# Plan: Functional Integrity & Data Hardening

This plan fixes the critical regressions in input handling, network feedback, and data persistence while introducing a more professional "Lodge Metadata" protocol.

## 1. Input Stabilization (Shift & Control)
**Objective**: Fix the bug blocking special characters and clobbering shortcuts.

### A. Fix `KeyMapper` Parser (`telarex-tui/src/events/key_mapper.rs`)
- Remove ambiguous 1-character modifier aliases (`c`, `a`, `s`).
- Only allow `ctrl`, `alt`, `shift` (case-insensitive) as modifiers.
- Ensure that if a key string is just a single character (e.g., "c"), it is treated as a `KeyCode::Char('c')` with `NONE` modifiers.

### B. Harden Editor Input Loop (`telarex-tui/src/components/editor/mod.rs`)
- **Insert Mode**: Strictly allow only `KeyCode::Char` if `modifiers` does not contain `CONTROL` or `ALT`.
- **Normal Mode**: Gated command execution. If `CONTROL` is present, it *must* resolve to a global/custom shortcut; otherwise, it is ignored (prevents `Ctrl+C` from triggering the `c` command).
- **Copy/Paste**: Implement `UIAction::Copy` and `UIAction::Paste` and wire them to the OS clipboard.

## 2. Network Confirmation & Feedback
**Objective**: Ensure the UI accurately reflects the network state.

### A. Update Protocol (`telarex-core/src/network/mod.rs`)
- Add `NetworkEvent::LodgeLeft { lodge_id: Uuid }`.
- Add `NetworkEvent::NetworkShutdown`.

### B. Emitting Events (`telarex-core/src/network/lodgenet.rs`)
- Update `NetworkManager` to send `LodgeLeft` after processing `LeaveLodge`.
- Send `NetworkShutdown` when the network loop terminates.

### C. UI Reaction (`telarex-tui/src/app.rs`)
- Add status bar notifications or "toast" messages (via `ErrorModal` in Info mode) when leaving or disconnecting.

## 3. Data Integrity (Anti-Poisoning)
**Objective**: Prevent duplicate lodges and stale sessions.

### A. Database Hardening (`crates/telarex-core/src/database/mod.rs`)
- Add `UNIQUE(path)` constraint to the `lodges` table.
- Use `INSERT OR IGNORE` or `UPDATE` logic for `register_lodge`.
- Add `UNIQUE(lodge_id, peer_id)` to `sessions` and use `REPLACE` for `register_session`.

### B. Config Cleanup
- Refactor `add_recent_project` to perform a "de-duplication" pass before saving.

## 4. Lodge Metadata Protocol
**Objective**: Provide rich peer/lodge info to the TUI.

### A. Schema Update
```rust
pub struct LodgeMetadata {
    pub owner_username: String,
    pub capability_hash: String,
    pub member_count: u32,
    pub is_private: bool,
}
```
- Update `WireMessage::Discovery` to include this metadata.
- Update `WelcomeView` to display the "Owner" and "Private" status in the discovery list.

## 5. Verification Plan
- **Input Test**: Type `#`, `/`, `\`, and `@` in a Rust file. Verify they are inserted correctly.
- **Shortcut Test**: Press `Ctrl+C` and `Ctrl+V`. Verify text is copied/pasted and the `c` command is NOT triggered.
- **Duplication Test**: Open the same project 3 times. Verify `recent_projects` only contains 1 entry.
- **Network Test**: Disconnect from the network and verify the Status Bar updates to `Offline` with 0 peers.
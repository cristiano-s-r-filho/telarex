# Plan: Core Modularization & Comprehensive Testing

This plan aims to elevate TelaRex to Beta-grade stability by increasing modularity in `telarex-core` and implementing a robust, multi-layered testing strategy.

## 1. Objectives
- **Deep Modularization**: Break down large modules (Buffer, CRDT, Database) into smaller, testable units.
- **Integration Testing**: Implement end-to-end tests for LodgeNet synchronization and workspace sharing.
- **Pristine Build**: Resolve all remaining compiler warnings.
- **Test Shield**: Increase test count from 6 to 25+ covering critical paths.

## 2. Structural Changes (`telarex-core`)

### A. Buffer & History Separation
- Refactor `buffer/document.rs` to delegate history management to a dedicated `History` engine in `buffer/history.rs`.
- **Tests**: Add `test_undo_stack_limit` and `test_redo_consistency`.

### B. CRDT & Sync Logic isolation
- Refactor `crdt/sync_engine.rs` to isolate the `ManagedDocument` state.
- **Tests**: Implement `test_concurrent_merges` simulating Peer A and Peer B editing the same line.

### C. Database Accountability
- Implement dedicated tests for the v2 Schema in `database/mod.rs`.
- **Tests**: `test_lodge_registration`, `test_session_logging`, `test_access_control_persistence`.

## 3. Integration Testing Suite
Create a new file `crates/telarex-core/tests/sync_integration.rs`:
- **Scenario**: 
  1. Initialize two `SyncEngine` instances.
  2. Mock a `SyncMessage` exchange.
  3. Verify both documents reach identical states.

## 4. TUI Component Verification
Add unit tests to `telarex-tui` (specifically `LayoutTree` and `KeyMapper`):
- **Layout**: `test_split_recursion_depth` and `test_focus_synchronization`.
- **Keymaps**: `test_shifted_symbol_resolution`.

## 5. Implementation Phases

### Phase 1: Modularization Pass
- Split `buffer/document.rs`.
- Split `crdt/sync_engine.rs`.
- Clean up `unused_mut` warnings.

### Phase 2: Core Shield (Unit Tests)
- Implement tests for the new sub-modules.
- Target: 100% coverage on `motions.rs` and `history.rs`.

### Phase 3: Integration Shield
- Implement the `sync_integration.rs` suite.
- Test the `NetworkManager` behavior in a headless mode.

### Phase 4: Final Verification
- Run `cargo test --workspace`.
- Ensure zero regression on rendering or focus.

## 6. Verification
- All tests must pass in a single `cargo test` run.
- Zero warnings in `cargo check`.

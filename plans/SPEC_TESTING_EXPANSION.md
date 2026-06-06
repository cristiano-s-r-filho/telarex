# Spec: Testing Expansion ("The Shield Initiative")

## 1. Current State

| Metric | Value |
|--------|-------|
| Total test functions | 11 |
| Test files | 6 |
| Integration tests | 2 |
| Unit test modules | 4 |
| Code with `#[allow(dead_code)]` | 50 annotations |
| TUI crate tests | 0 |
| Property-based tests | 0 |
| Fuzz tests | 0 |

**Target: 50+ tests** in 10+ test files across both crates.

## 2. Test Structure

```
crates/telarex-core/tests/
├── actor_tests.rs              # 3 → 6 tests
├── sync_integration.rs         # 1 → 5 tests
├── config_tests.rs             # NEW: 4 tests
├── database_tests.rs           # NEW: 5 tests
├── network_tests.rs            # NEW: 4 tests
├── identity_tests.rs           # NEW: 6 tests (capability + multi-session)
└── buffer_integration.rs       # NEW: 3 tests

crates/telarex-tui/tests/
├── event_tests.rs              # NEW: 4 tests (key mapping, action dispatch)
└── layout_tests.rs             # NEW: 3 tests

In-module unit tests:
core/src/buffer/document.rs     # 2 → 4 tests
core/src/buffer/motions.rs      # 3 → 6 tests
core/src/buffer/history.rs      # 0 → 3 tests
core/src/crdt/sync_engine.rs    # 1 → 4 tests
core/src/network/auth/mod.rs    # 1 → 3 tests
core/src/config/schema.rs       # 0 → 3 tests
core/src/config/theme_engine.rs # 0 → 2 tests
core/src/errors/mod.rs          # 0 → 2 tests
```

## 3. Test Categories & Requirements

### 3.1 Unit Tests (25+ tests)

| Module | Tests | What To Cover |
|--------|-------|---------------|
| `buffer/document.rs` | 4 | Insert, delete at bounds/middle/end, undo/redo consistency |
| `buffer/motions.rs` | 6 | Word forward/back/end, line start/end, paragraph, edge cases (empty line, whitespace-only) |
| `buffer/history.rs` | 3 | Stack limit, undo past limit, redo consistency |
| `buffer/buffer_manager.rs` | 2 | Get/create, scratch buffer, remove/close |
| `buffer/macro_engine.rs` | 2 | Record, playback, empty recording |
| `config/schema.rs` | 3 | Default values, serialization round-trip, recent projects dedup |
| `config/theme_engine.rs` | 2 | Theme loading from files, fallback on missing theme |
| `errors/mod.rs` | 2 | Factory methods, Display formatting |
| `crdt/sync_engine.rs` | 4 | Local change propagation, remote merge, conflict resolution, empty doc |
| `network/auth/mod.rs` | 3 | Key generation, sign+verify round-trip, invalid signature rejection |
| `workspace/mod.rs` | 2 | Add/remove files, active file tracking |

### 3.2 Integration Tests (15+ tests)

| File | Tests | What To Cover |
|------|-------|---------------|
| `actor_tests.rs` | 6 | Buffer lifecycle: create, edit, save, close; concurrent edits; multiple buffers |
| `sync_integration.rs` | 5 | 2-peer sync, 3-peer sync, concurrent edits merge, offline-then-sync, partial sync |
| `config_tests.rs` | 4 | Full config save/load cycle, config migration, theme directory loading |
| `database_tests.rs` | 5 | CRUD operations: lodge, session, access_control; reset; concurrent connections |
| `network_tests.rs` | 4 | Lodge manifest validation, capability delegation chain, tombstone acceptance |
| `identity_tests.rs` | 6 | Key generation, device signing, multi-device collation, device revocation |
| `buffer_integration.rs` | 3 | Tree-sitter parse + edit cycle, incremental parse speed, large file handling |
| `event_tests.rs` | 4 | Key mapper resolution, action dispatch, mode switching, unknown key fallback |
| `layout_tests.rs` | 3 | Split ratio calculation, focus routing, max split depth |

### 3.3 Property-Based Tests (5+ tests)

Using `proptest` crate:

| Test | Property |
|------|----------|
| Document insert/delete | `∀ ops: apply(ops).len() == apply(reverse(ops)).len()` after certain sequences |
| CRDT merge | `∀ a,b: merge(a,b) == merge(b,a)` (commutativity) |
| CRDT associativity | `∀ a,b,c: merge(merge(a,b),c) == merge(a,merge(b,c))` |
| Capability attenuation | `∀ c: attenuate(c).permissions ⊆ c.permissions` |
| Motions round-trip | `∀ pos, text: apply_motion(apply_motion(pos, fwd), back) == pos` (within tolerance) |

### 3.4 Concurrency Tests (3+ tests)

| Test | What |
|------|------|
| Buffer actor concurrent sends | 10 parallel `GetOrCreate` → no deadlock, all return |
| Database concurrent reads | 5 threads reading same data → consistent |
| Network event ordering | Events from 3 peers → causal order maintained |

## 4. Test Infrastructure

### 4.1 Headless Mode

For integration tests involving the TUI:

```rust
// In telarex-tui tests:
struct MockTerminal {
    // Implements ratatui::backend::Backend
    // Buffers all render calls for assertion
}

impl MockTerminal {
    fn rendered_frames(&self) -> &[Frame];
    fn assert_last_frame_contains(&self, text: &str);
    fn assert_cursor_at(&self, x: u16, y: u16);
}
```

### 4.2 Network Test Harness

```rust
struct TestNetwork {
    peers: Vec<NetworkManager>,
    // In-memory only, no real sockets
    hub: InMemoryHub, // routes messages between peers
}
```

### 4.3 Test Utilities

- `create_test_document(text: &str)` → `Document`
- `create_test_buffer(text: &str)` → `ManagedBuffer`
- `create_test_config()` → `TelaRexConfig` (with test paths)
- `create_test_database()` → `Database` (in-memory SQLite)
- `generate_test_identity()` → `(QuantumAuth, Vec<u8>)`

## 5. CI Integration

```toml
# .github/workflows/ci.yml (future)
- name: Run tests
  run: cargo test --workspace --all-features

- name: Run tests with sanitizers (nightly)
  run: cargo test --workspace -Z sanitizer=address

- name: Run property tests
  run: cargo test --workspace --release proptest_  # longer timeout

- name: Check for dead code
  run: cargo clippy --workspace -- -D warnings -W clippy::dead_code
```

## 6. Configuration (Cargo.toml)

```toml
# workspace Cargo.toml
[workspace.dependencies]
proptest = { version = "1.5", optional = true }

# telarex-core Cargo.toml
[dev-dependencies]
proptest = { workspace = true }
tempfile = "3.10"

# telarex-tui Cargo.toml
[dev-dependencies]
tempfile = "3.10"
```

## 7. Implementation Order

1. **Phase 1** (immediate): Add unit tests for history, config, errors, workspace — these are simple, isolated, high-value
2. **Phase 2** (this sprint): Add integration tests for database, config, buffer — mockable with tempfile
3. **Phase 3** (next sprint): CRDT multi-peer integration tests, network tests
4. **Phase 4** (ongoing): property-based tests for CRDT invariants, fuzz tests
5. **Phase 5** (tooling): CI pipeline, test coverage reporting, dead-code linting

## 8. Coverage Target

| Module | Current | Target |
|--------|---------|--------|
| `buffer/` | 4 tests | 17 tests |
| `crdt/` | 1 test | 9 tests |
| `config/` | 0 tests | 9 tests |
| `database/` | 0 tests | 5 tests |
| `network/` | 1 test | 10 tests |
| `errors/` | 0 tests | 2 tests |
| `workspace/` | 0 tests | 2 tests |
| `events/` | 0 tests | 4 tests |
| **Total** | **11 tests** | **58+ tests** |

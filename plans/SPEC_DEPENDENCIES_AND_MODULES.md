# Spec: Dependency Audit & Modularization

## 1. Current Dependency State

### 1.1 Dead/Unnecessary Dependencies

| Crate | Why It's There | Actual Usage | Action |
|-------|---------------|--------------|--------|
| `syntect` | Regex-based highlighting | Superseded by tree-sitter | **Remove** from both Cargo.toml files |
| `syntect-tui` | TUI bridge for syntect | Unused | **Remove** |
| `tui-file-explorer` | Third-party file explorer | Unused (custom FileTree used instead) | **Remove** |
| `tabitha` | TUI app builder | Used in `main.rs` but 3-line wrapper. Can inline AppBuilder pattern. | **Remove** and inline |
| `num-bigint` | Big number math | Listed but no actual usage found | **Remove** from both Cargo.toml files |
| `num-traits` | Number trait abstractions | No usage found | **Remove** |
| `confy` | Config auto-load/save | Only used in `config/mod.rs`. Adds 2 deps. Can replace with manual file ops. | **Remove** and inline with serde + directories |
| `git2` | Git integration | Only imported, no actual UI integration | Keep but make optional; currently dead code |
| `sha2` | Hashing | Used? Audit needed | Keep (likely needed for identity) |
| `ignore` | Gitignore-aware file walking | Used in search palette | **Keep** |
| `clap` | CLI argument parsing | Used in main.rs | **Keep** |
| `env_logger` | Logging initialization | Used in main.rs | **Keep** |
| `walkdir` | File walking | Used in explorer and project search | **Keep** |
| `dirs` | System dirs | Duplicates `directories` crate | **Remove** — use `directories` only |
| `hex` | Hex encoding | Used? Audit needed | Keep (useful for key display) |
| `uuid` | UUID generation | Used in database and lodge | Keep, but move to workspace dep |

### 1.2 Target Post-Cleanup Dependency List

**telarex-core** dependencies:
```
serde, serde_json, toml           # serialization
directories                       # XDG paths
anyhow, thiserror                 # error handling
log                               # logging
ropey                             # text buffer
tree-sitter, tree-sitter-highlight # syntax parsing
tree-sitter-rust/json/md/toml    # grammar files
automerge                         # CRDT
tokio                             # async runtime
uuid                              # IDs
hex                               # hex encoding
sha2                              # hashing
rusqlite                          # persistence
ratatui                           # Style types in core
libp2p (features: gossipsub, mdns, noise, tcp, yamux, kad, identify, ping)
pqc_dilithium                     # post-quantum auth
futures                           # async primitives
walkdir                           # file traversal
```

**telarex-tui** dependencies:
```
telarex-core
ratatui, crossterm               # TUI framework
tokio                            # async
anyhow, log                      # utilities
clap                             # CLI
arboard                          # clipboard
ignore                           # gitignore file walk
unicode-width                    # width calculations
serde_json                       # (only if needed in TUI)
```

## 2. Dead Code Cleanup

### 2.1 Empty Files (Delete)

| File | Reason |
|------|--------|
| `crates/telarex-core/src/buffer/operation.rs` | Empty — declared in mod.rs but 0 content |
| `crates/telarex-core/src/utils/mod.rs` | Empty — remove module declaration from lib.rs |
| `crates/telarex-core/src/workspace/project.rs` | Empty — remove module declaration from mod.rs |

### 2.2 `#[allow(dead_code)]` Audit

**Target: 0 dead code annotations.** Each must be resolved by one of:
1. **Wire up** — if the code is needed, implement proper callers
2. **Remove** — if the code is truly unused, delete it
3. **Gate behind feature flag** — if planned for future, add feature gate

| Location | Count | Resolution |
|----------|-------|------------|
| `events/actions.rs` | 18 UIAction variants | Most are planned features. Keep but gate with `#[cfg(feature = "unstable")]` |
| `workspace/mod.rs` | 5 methods | Remove `unshare()`, `remove_file()`; keep `next_file()`, `prev_file()`, `active_file()` |
| `theme/mod.rs` | 5 fields | Keep — planned for theme expansion |
| `search.rs` | 4 items | Wire up render/show properly or remove |
| `command.rs` | 3 items | Wire up or remove |
| `network/mod.rs` | 3 NetworkCommand variants | Remove `SendAuthChallenge`, `SendAuthResponse` — handled in auth module |
| `explorer/mod.rs` | 2 methods | Wire up `input_modal_show()` |
| `motions.rs` | 2 motions | ParagraphForward/Backward — keep, they're planned |
| `tab_bar.rs` | 3 items | Fix imports and wire up |
| `app.rs` | 1 field | `identity_keys` — needed for identity system, keep |
| `editor_view.rs` | 1 method | `start_lsp()` — keep and wire up |
| `config_modal.rs` | 1 method | `get_config()` — keep and wire up |
| `tab_controller.rs` | 1 method | `remove_active_tab()` — wire up |
| `components/mod.rs` | 1 import | Fix re-export |

### 2.3 Unused Imports

Run `cargo clippy` to identify and remove all unused imports in every file.

## 3. Modularization

### 3.1 Modern Module Structure

Migrate from `mod.rs` style to `directoryname.rs` style (Rust 2018+).

**Before:**
```
actor/
  mod.rs
```

**After:**
```
actor.rs       # replaces actor/mod.rs
actor/
  (only if >2 submodules)
```

Migration plan:
1. Rename each `dir/mod.rs` to `dir.rs` at parent level
2. Remove the old `dir/` directory
3. Update `lib.rs` / parent `mod.rs` declarations
4. Exception: keep directories with 2+ submodules as directories

Directories to migrate:
- `actor/` → `actor.rs` (single file)
- `command/` → `command.rs` (single file)
- `crdt/` → `crdt.rs` (single file)
- `database/` → `database.rs` (single file)
- `errors/` → `errors.rs` (single file)
- `lsp/` → `lsp.rs` (single file)
- `syntax/` → `syntax.rs` (single file) — only if submodules can be inlined
- `workspace/` → `workspace.rs` (single file)

Keep as directories (2+ submodules):
- `buffer/` — multiple sub-files
- `config/` — multiple sub-files
- `network/` — has `auth/` subdirectory

For TUI crate:
- `bars/` → inline into single file
- `palettes/` → inline into single file
- `modals/` → inline into single file (or merged into components)
- `events/` → `events.rs` (single file)
- `render/` → `render.rs` or delete (it's a placeholder)
- `theme/` → `theme.rs` (single file)
- `network/` → `network.rs` (single file — just 1 line re-export)
- `utils/` → `utils.rs` (single file)

### 3.2 Crate Boundary Cleanup

**Current issue:** `telarex-tui` depends on many crates that should be in `telarex-core`:
- `arboard` (clipboard) — move to core, expose `ClipboardService`
- `automerge` — duplicated in both crates. Keep only in core, re-export needed types.
- `uuid`, `rand`, `hex` — duplicated. Keep in core, `pub use` in tui if needed.
- `serde_json`, `serde` — duplicated. Core owns serialization, tui re-exports.

**Target:** `telarex-tui` depends ONLY on:
- `telarex-core`
- `ratatui`, `crossterm`
- `tokio`
- `anyhow`, `log`
- `clap`
- `unicode-width`

Everything else lives in core and gets re-exported.

### 3.3 Feature Gates

```toml
[features]
default = ["lsp", "clipboard", "git"]
lsp = ["telarex-core/lsp"]
clipboard = ["arboard"]
git = ["git2"]
unstable = []  # for features in development (dead-coded UI actions)
```

## 4. Implementation Plan

### Phase 1: Dependency Cleanup (immediate)
- [ ] Remove `syntect`, `syntect-tui` from workspace + both crates
- [ ] Remove `tui-file-explorer` from tui crate
- [ ] Remove `num-bigint`, `num-traits` from both crates
- [ ] Remove `dirs` from core crate (use `directories` instead)
- [ ] Remove `tabitha` — inline `AppBuilder` logic into `main.rs`
- [ ] Move duplicate deps to workspace level (uuid, rand, hex, futures)
- [ ] Move `arboard` to workspace (core needs clipboard abstraction)

### Phase 2: Dead Code Removal
- [ ] Delete 3 empty files: `operation.rs`, `utils/mod.rs`, `project.rs`
- [ ] Remove unused UIAction variants (gate behind `unstable` feature)
- [ ] Remove `SendAuthChallenge`, `SendAuthResponse` from NetworkCommand
- [ ] Remove `unshare()`, `remove_file()` from workspace
- [ ] Fix all tab_bar unused imports
- [ ] Add `#![deny(dead_code)]` to lib.rs once clean

### Phase 3: Module Restructuring
- [ ] Migrate single-file directories to `name.rs` style
- [ ] Keep multi-file directories as-is (buffer, config, network, components)
- [ ] Inline single-file subdirectories (bars, palettes, modals)
- [ ] Clean up crate dependency boundaries

### Phase 4: Linting
- [ ] Add `clippy` configuration to workspace
- [ ] Run `cargo clippy --fix` for automatic fixes
- [ ] Add `#![deny(clippy::all, clippy::pedantic)]` after cleanup
- [ ] Add CI lint step

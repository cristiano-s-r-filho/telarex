# Roadmap: TelaRex Beta Release (v0.2.0-beta)

This roadmap outlines the path to a stable, user-ready Beta release of TelaRex. Our focus shifts from rapid feature development to "Zero-Bug" stability, performance hardening, and user documentation.

## 1. Feature Freeze & Hardening
- **LodgeNet 2.0**: Finalize the public-internet resilience (Phase 2 of previous plan).
- **Quantum Identity**: Finalize ML-DSA transition.
- **Git Sidecar**: Implement Git operations as a parallel "manual" workflow, ensuring it never collides with Automerge's automated sync.
- **Bento UI Completion**: Standardize the visual language across all screens.

## 2. Testing Expansion (The "Shield" Initiative)
✅ **All Shield items complete** — 90 tests across 15 test locations:
- **Document**: 8 tests (buffer ops, undo/redo, load/save)
- **History**: 6 tests (stack depth, dedup, undo/redo consistency)
- **Buffer Manager**: 4 tests (create, get, same instance, remove)
- **Schema**: 7 tests (defaults, dedup, cap, roundtrip, profiles, keymaps, network)
- **Errors**: 5 tests (levels, display, format, factory fns)
- **Motions**: 3 tests (word, line, paragraph navigation)
- **Sync Engine**: 8 unit tests (3-way merge, concurrent insert, causal chain, cursor sync, stress)
- **Database**: 8 tests (init, CRUD lodges, cascade delete, sessions, reset, recent projects, error log)
- **Actor**: 2 integration tests
- **Config**: 4 integration tests
- **Identity**: 5 integration tests (key gen, sign/verify, wrong key, wrong msg, deterministic)
- **E2E**: 5 tests (2-peer flow, 3-peer convergence, cursor sync, conflict resolution, multi-round)
- **KeyMapper**: 14 tests (parse key, parse action, resolve global/window/editor/explorer, passthrough, release events)
- **LayoutTree**: 10 tests (split, close, navigate, sync_focus, compute_rects, noop cases)

## 3. Performance Benchmarks
- **Input Latency**: Ensure 100% of keystrokes are processed in <5ms.
- **Memory Footprint**: Monitor heap growth during large project walks (Search).

## 4. Market Comparison (2026 Q2/Q3 Context)
- **TelaRex**: The only TUI editor providing **Native, Serverless P2P Collaboration**.
- **Neovim 0.12**: Focuses on Lua-based AI agents; collaboration remains a plugin-heavy manual process.
- **Helix 26.05**: Extremely fast and stable, but strictly single-user.
- **Zed (TUI Mode)**: Fast, but requires centralized accounts for collaboration.
- **Beta Goal**: Position TelaRex as the "Collaboration-First" alternative to Helix.

## 5. Release Checklist
- [ ] 0 Warnings, 0 Errors across all crates.
- [ ] 50%+ Test Coverage on `telarex-core`.
- [ ] User Guide: `docs/BETA_GUIDE.md` covering LodgeNet and Git workflows.
- [ ] Binary Packaging: Cross-platform release builds via GitHub Actions.

## 6. Implementation Stages
1. **Stage 1**: ✅ "Shield" pass — all 90 tests written.
2. **Stage 2**: "Clean House" (Resolve all remaining TUI logic quirks).
3. **Stage 3**: "Git Sidecar" (Implement manual Git commit/push actions).
4. **Stage 4**: Final Bento Polish & Documentation.

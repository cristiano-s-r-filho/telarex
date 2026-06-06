# Roadmap: TelaRex Beta Release (v0.2.0-beta)

This roadmap outlines the path to a stable, user-ready Beta release of TelaRex. Our focus shifts from rapid feature development to "Zero-Bug" stability, performance hardening, and user documentation.

## 1. Feature Freeze & Hardening
- **LodgeNet 2.0**: Finalize the public-internet resilience (Phase 2 of previous plan).
- **Quantum Identity**: Finalize ML-DSA transition.
- **Git Sidecar**: Implement Git operations as a parallel "manual" workflow, ensuring it never collides with Automerge's automated sync.
- **Bento UI Completion**: Standardize the visual language across all screens.

## 2. Testing Expansion (The "Shield" Initiative)
We currently have ~48 tests (up from 7). Expand to:
- **Core Coverage**: ✅ Done (document, history, buffer_manager, schema, errors, actor, sync_integration)
  - `CRDT Engine`: Multi-peer merge consistency tests.
  - `Database`: Migration and persistence integrity tests.
- **TUI Logic**: ⬜ TODO
  - `Event Routing`: Unit tests for `KeyMapper` resolution.
  - `Focus Sync`: Verification that `LayoutTree` focus pushes correctly.
- **Integration**:
  - `End-to-End`: Simulate a headless network join and document sync.

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
1. **Stage 1**: "Shield" pass (Add missing tests).
2. **Stage 2**: "Clean House" (Resolve all remaining TUI logic quirks).
3. **Stage 3**: "Git Sidecar" (Implement manual Git commit/push actions).
4. **Stage 4**: Final Bento Polish & Documentation.

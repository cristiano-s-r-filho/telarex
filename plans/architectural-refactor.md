# Implementation Plan: Major Architectural Refactor (Hardened)

## Objective
To fundamentally redesign TelaRex as a modular, production-ready technical workspace. This refactor resolves the chronic failures in input handling, buffer isolation, and LodgeNet connectivity by adopting best-in-class Rust patterns (Actor model, Pub/Sub, Transactional Core).

## 1. Core Architecture: The Transactional Core
- **`BufferManager`**: A new central registry in `telarex-core` that owns all `Document` instances. This ensures strict isolation and allows multiple tabs to view the same file without desync.
- **Operation-Based History**: Refactor `History` to store incremental edits (ops) rather than full `Rope` clones.
- **Grapheme-Aware Engine**: Standardize on `unicode-width` and `unicode-segmentation` for all coordinate math.

## 2. Networking 2.0: Pub/Sub & reliable Sync
- **Gossip-Heads Protocol**:
    - **Pub/Sub (Gossipsub)**: Use topics per Lodge to broadcast document "Heads" (hashes) only.
    - **Request-Response**: Use a reliable libp2p protocol for the actual Automerge binary sync.
- **Network Actor**: Decouple `NetworkManager` from Automerge logic. It becomes a pure message router.
- **Lodge Registry**: A central state in `App` to track all connected lodges, their members, and our local presence.

## 3. Interaction & Input Hardening
- **Universal Input Routing**:
    - Refactor `KeyMapper` to provide a "Raw Character" fallback for unmapped keys in all modes. This definitively unblocks `/`, `?`, and `Shift` sequences.
    - Standardize on a `Command` pattern: Input → Action → Transaction.
- **Lodge Identity UI**:
    - Add a "Copy ID" feature to the status bar.
    - Implement a robust Kademlia bootstrap for the "Join by ID" flow.

## 4. Async Highlighting & IO
- **Background Sync Engine**: Move Automerge merging to a dedicated task to prevent UI stutter.
- **Tokio-Native IO**: All file and network operations moved to non-blocking tasks.

## Key Files to Refactor
- `crates/telarex-core/src/buffer/document.rs` -> `BufferManager` logic.
- `crates/telarex-core/src/network/lodgenet.rs` -> Actor & Pub/Sub overhaul.
- `crates/telarex-tui/src/events/key_mapper.rs` -> Fallback logic.
- `crates/telarex-tui/src/app.rs` -> Central action routing.

## Phase 1: The Transactional Core (Immediate Focus)
1. Implement `BufferManager` and migrate `Editor` to use references.
2. Refactor `KeyMapper` for unblocked input.
3. Patch `TreeHighlighter` to match Rope lines exactly.

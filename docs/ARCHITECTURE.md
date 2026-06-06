# TelaRex: Architectural Sovereignty

This document outlines the core architectural principles and patterns of the TelaRex technical workspace.

## 1. Zero-Contention State (The Actor Model)
TelaRex rejects the shared mutable state anti-pattern (`Arc<Mutex<T>>`) for its core loop. Instead, it utilizes an **Actor Model**:

### `BufferActor`
- **Responsibility**: Sole owner of all text buffers (`Rope`) and syntax trees (`Tree-sitter`).
- **Communication**: Asynchronous `mpsc` channels.
- **Benefits**: Eliminates deadlocks, provides atomic transactional edits, and ensures single-threaded consistency for complex data structures.

### `LodgeActor`
- **Responsibility**: Manages P2P networking and synchronization.
- **Protocol**: libp2p + Gossipsub with ML-DSA (Post-Quantum) signatures.
- **Interaction**: Relays sync messages to the `BufferActor` via high-priority channels.

## 2. High-Performance Text Analysis
- **Incremental Parsing**: Tree-sitter pulls text directly from `Ropey` chunks using the `parse_with` API ($O(\log N)$ complexity).
- **Drained Highlighting**: Parser requests are automatically debounced/drained, ensuring that UI rendering only reflects the most current buffer state.

## 3. Bento-Box UI Architecture
The TUI is designed using a **modular grid system**:
- **Declarative Layouts**: Zero manual `Rect` arithmetic; all areas are calculated via `ratatui` Constraints.
- **Nuclear Draw**: Every frame clear and background fill ensures no visual artifacts or ghost characters from previous renders.
- **Data Sovereignty**: Project history and Lodge metadata are persisted in a hardened SQLite database.

## 4. Interaction Model
- **Modeless Sovereignty**: A unified interaction layer removes the complexity of Vim-modes, favoring immediate technical productivity.
- **Transaction Flow**: Input → `KeyMapper` → `UIAction` → `Actor Command` → `State Update`.

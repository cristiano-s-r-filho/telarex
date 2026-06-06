# telarex-core

The heart of the TelaRex collaborative technical workspace. This crate manages the high-performance data structures and decentralized protocols required for a modern terminal editor.

## Key Components

### 1. `ManagedBuffer` & `BufferActor`
Implements the **Actor Model** for state management. The `BufferActor` is the sole owner of the `Rope` (text data) and `Tree-sitter` tree. It processes edits incrementally, ensuring that UI updates and network synchronization are always performed on a consistent state without the need for manual mutex locking in the TUI thread.

### 2. `LodgeActor` (Network Layer)
Handles the **libp2p** gossip and synchronization. It utilizes **ML-DSA** (Dilithium) for quantum-resistant peer authentication. Lodges are discovered via a global gossip topic and verified using a challenge-response protocol.

### 3. Incremental Syntax Highlighting
Integrates Tree-sitter directly with Ropey chunks. By using the `parse_with` API, we avoid large string allocations and perform syntax analysis in $O(\log N)$ time, providing a smooth experience even in multi-megabyte source files.

### 4. Database (SQLite)
Provides a hardened persistent layer for:
- Recent Projects and workspace history.
- Lodge metadata and authorized peer registries.
- Security-critical identity seeds.

## Technical Patterns
- **Zero-Contention State**: Favoring message-passing over shared mutable state (`Arc<Mutex<T>>`).
- **Data Sovereignty**: Local-first design with absolute path canonicalization and isolated Lodge sync states.
- **Post-Quantum Security**: Native integration of NIST-standard quantum-resistant signatures for all collaborative operations.

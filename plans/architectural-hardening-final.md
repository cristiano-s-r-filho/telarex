# Implementation Plan: Architecture & UI Revamp (Phase 3+)

## Objective
To eliminate the `Arc<Mutex<T>>` anti-pattern by adopting an **Actor-based state management** system and to revamp the TUI into a modern, grid-based "technical dashboard."

## 1. Actor Model Transition (Shared State)
- **Problem**: `ManagedBuffer` and `SyncEngine` are currently wrapped in `Arc<Mutex<T>>`, leading to potential deadlocks and performance ceilings.
- **Solution**:
    - **`BufferActor`**: A dedicated background task that owns the `Rope` and `Tree`. Components request read-only snapshots or send `BufferCommand` messages to apply edits.
    - **`LodgeActor`**: A background task that owns the networking and sync state, communicating with the `BufferActor` via high-priority channels.
    - **`AppController`**: The UI thread remains the sole owner of the UI state, receiving `StateUpdate` events from actors.

## 2. UI Revamp: Bento-Box Dashboard
- **`WelcomeView`**:
    - Replace the vertical list with a **grid layout**.
    - Left Pane (2/3): Hero banner + Recent Projects (2-column grid).
    - Right Pane (1/3): Discovered Lodges + Active Session Info.
    - Bottom Row: Quick shortcuts hints.
- **`EditorView`**:
    - **Omni-Layout**: Move split management to the central `AppState`.
    - **Adaptive UI**: Render floating "Bento Pills" for info instead of a solid status bar.
- **`ConfigView`**:
    - Modularize into categories (General, Theme, Keys).

## 3. Interaction & Input (Definitive Unblock)
- **Command-Transaction Pattern**: 
    - `KeyMapper` → `UIAction` → `Command` → `LodgeActor` / `BufferActor`.
    - Absolute separation between UI navigation and data mutation.

## 4. Quality & Verification
- **Documentation**:
    - Detailed `docs/ARCHITECTURE.md` explaining the actor flow.
    - Unified doc comments in `telarex-core`.
- **Testing**:
    - **Integration**: Multi-actor concurrency tests in `crates/telarex-core/tests`.
    - **Unit**: KeyMapper exhaustive resolution tests.

## Key Changes by Phase
### Phase 1: The Actor Core
1. Implement `BufferActor` and `mpsc` command loop.
2. Refactor `App` to be the primary event consumer.

### Phase 2: Bento Dashboard
1. Implement `Grid` layout helpers.
2. Redesign `WelcomeView` and `StatusBar`.

### Phase 3: Documentation & Test Shield
1. Final doc sweep and coverage increase.

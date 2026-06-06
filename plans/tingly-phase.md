# Plan: Tingly Phase (Collaboration & Security)

This plan implements the "Tingly" core of TelaRex: full P2P document synchronization, authenticated user profiles, a workspace sharing protocol, and ZK-based access control.

## Objectives
1.  **User Profiles**: Define `Profile` (PeerID, Name, PubKey) and integrate into the UI.
2.  **Lodge Protocol**: Define a system where folders can be marked as "Shareable Workspaces" with unique IDs.
3.  **Full P2P Sync**: Bridge `automerge` CRDTs with `libp2p` Gossipsub for real-time, conflict-free editing.
4.  **ZK-Based Access**: Implement a cryptographic challenge-response system to ensure only authorized peers can join a "Lodge" workspace without revealing long-term secrets.

## Key Files & Context
- `telarex-core/src/config/schema.rs`: Add User Profile fields.
- `telarex-core/src/workspace/mod.rs`: Add "Shareable" state and WorkspaceID.
- `telarex-tui/src/network/mod.rs`: Expand P2P message types for state sync and auth.
- `telarex-core/src/crdt/sync_engine.rs`: Implement patch-based sync (instead of full text replacement).

## Phased Implementation Plan

### Phase 1: Identity & Profiles
1.  Update `TelaRexConfig` to include `username` and a generated `ed25519` keypair.
2.  Display the user's name/ID in the `StatusBar`.

### Phase 2: The "Lodge" Protocol
1.  Extend `Workspace` to include a `uuid` and `is_shared` flag.
2.  Implement a "Share Workspace" command in the Command Palette.
3.  Broadcast "Lodge Discovery" messages over Gossipsub when a workspace is shared.

### Phase 3: P2P Sync Overhaul
1.  Refactor `SyncEngine` to emit/absorb `automerge` sync messages (binary changesets).
2.  Wire `NetworkManager` to forward these changesets between peers.
3.  Implement a "Remote Cursor" state to track collaborator positions.

### Phase 4: ZK-Access Security
1.  Implement a simple Proof-of-Possession or Challenge-Response using the workspace key.
2.  Peers must solve a cryptographic puzzle (ZK-lite) to prove they are members of the Lodge before being accepted as sync targets.

## Verification
1.  **Multi-Instance Test**: Run two copies of `trex` on the same machine.
2.  **Discovery**: Peer A shares a folder; Peer B sees it in their "Network" list.
3.  **Sync**: Peer A types; Peer B sees characters appear in real-time.
4.  **Security**: Peer C (unauthorized) attempts to join but fails the ZK-challenge.

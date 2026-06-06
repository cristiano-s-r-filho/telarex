# Plan: LodgeNet Global Connectivity (DHT & Presence)

This phase expands TelaRex's P2P network from local LAN (mDNS) to global LodgeNet using Kademlia DHT and Identify protocols, and implements visible peer presence.

## Objectives
1.  **Global Discovery**: Implement Kademlia DHT to allow peers to find Lodges across the internet (beyond LAN).
2.  **Address Exchange**: Use the Identify protocol to automatically share listen addresses with peers.
3.  **Presence Visibility**: Track and display a list of "Lodge Members" currently active in the workspace.
4.  **Network Stability**: Add the Ping protocol to monitor peer health and prune dead connections.

## Key Files & Context
- `telarex-tui/Cargo.toml`: Add `kad`, `identify`, `ping` features to libp2p.
- `telarex-tui/src/network/mod.rs`: Update `MyBehaviour` and `NetworkManager` for DHT operations.
- `telarex-tui/src/app.rs`: Manage the list of active Lodge members.
- `telarex-tui/src/screens/editor_view.rs`: Add a UI element for the "Lodge Members" list.

## Implementation Steps

### Phase 1: Dependency Upgrade
1.  Enable `kad`, `identify`, and `ping` features in `libp2p` dependency.

### Phase 2: Global Behaviour (DHT & Identify)
1.  Update `MyBehaviour` in `network/mod.rs` to include `Kademlia`, `Identify`, and `Ping`.
2.  Refactor `NetworkManager::start` to initialize these behaviors.
3.  Implement `Identify` event handling: Automatically add identified peer addresses to the Kademlia DHT.
4.  Implement `Kademlia` event handling: Track `PutRecord` and `GetRecord` for Lodge IDs.

### Phase 3: Lodge Aggregation (The "LodgeNet" Protocol)
1.  When a workspace is shared, "Put" its Lodge UUID as a key in the DHT with the local PeerID as the value.
2.  When joining a Lodge, "Get" the providers for the Lodge UUID from the DHT to find the host.

### Phase 4: Presence & UI
1.  Extend `NetworkEvent` with `PeerStatus(String, bool)` to track active members.
2.  Update the **Editor Status Bar** to show a "Peers" count pill.
3.  Implement a floating "Members" list in the Editor (via TachyonFX overlay).

## Verification
1.  **PeerID Consistency**: Verify that PeerID remains stable across restarts (Account persistence).
2.  **Global Search**: Mock a bootstrap node and verify Peer A can find Peer B's Lodge ID in the DHT.
3.  **Member List**: Verify that when Peer B joins Peer A's Lodge, both see each other in their "Members" list.

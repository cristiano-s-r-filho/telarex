//! Networking — libp2p-based peer-to-peer lodge discovery, sync, and authentication.
//!
//! [`NetworkEvent`] and [`NetworkCommand`] define the protocol between the UI and
//! the network layer. [`NetworkManager`] implements the full p2p stack (gossipsub,
//! mDNS, Kademlia, identify, ping). The [`auth`] sub-module provides
//! quantum-resistant identity using ML-DSA (Dilithium).

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::path::PathBuf;

/// Events produced by the network layer and consumed by the UI/application.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum NetworkEvent {
    PeerConnected(String),
    PeerDisconnected(String),
    LodgeDiscovery { id: Uuid, name: String, peer_id: String },
    JoinRequest { lodge_id: Uuid, peer_id: String, username: String, public_key: Vec<u8> },
    SyncMessage { lodge_id: Uuid, path: PathBuf, data: Vec<u8> },
    AuthChallenge { lodge_id: Uuid, challenge: Vec<u8> },
    AuthVerify { lodge_id: Uuid, challenge: Vec<u8>, proof: Vec<u8>, public_key: Vec<u8> },
    NetworkError(String),
    LodgeLeft { lodge_id: Uuid },
    NetworkShutdown,
    LodgeMembersUpdated { lodge_id: Uuid, members: Vec<String> },
    LodgeJoined { lodge_id: Uuid },
    JoinRejected { lodge_id: Uuid },
    Tick,
}

/// Commands sent to the network layer from the UI/application.
#[derive(Debug, Clone)]
pub enum NetworkCommand {
    ShareLodge { id: Uuid, name: String },
    SendSync { lodge_id: Uuid, path: PathBuf, data: Vec<u8> },
    JoinLodge { lodge_id: Uuid, public_key: Vec<u8>, username: String },
    LeaveLodge { lodge_id: Uuid },
    Disconnect,
    AnnouncePresence { lodge_id: Uuid, username: String },
    SendAuthChallenge { lodge_id: Uuid, challenge: Vec<u8> },
    SendAuthResponse { lodge_id: Uuid, proof: Vec<u8> },
    ApproveJoin { lodge_id: Uuid, peer_id: String },
    RejectJoin { lodge_id: Uuid, peer_id: String },
}

pub mod lodgenet;
pub mod auth;
pub use lodgenet::NetworkManager;

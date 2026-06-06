use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum NetworkEvent {
    PeerConnected(String),
    PeerDisconnected(String),
    LodgeDiscovery { id: Uuid, name: String, peer_id: String },
    SyncMessage { lodge_id: Uuid, path: PathBuf, data: Vec<u8> },
    AuthChallenge { lodge_id: Uuid, challenge: Vec<u8> },
    AuthVerify { lodge_id: Uuid, challenge: Vec<u8>, proof: Vec<u8> },
    NetworkError(String),
    LodgeLeft { lodge_id: Uuid },
    NetworkShutdown,
    LodgeMembersUpdated { lodge_id: Uuid, members: Vec<String> },
    Tick,
}

#[derive(Debug, Clone)]
pub enum NetworkCommand {
    ShareLodge { id: Uuid, name: String },
    SendSync { lodge_id: Uuid, path: PathBuf, data: Vec<u8> },
    JoinLodge { lodge_id: Uuid, public_key: Vec<u8> },
    LeaveLodge { lodge_id: Uuid },
    Disconnect,
    AnnouncePresence { lodge_id: Uuid, username: String },
    SendAuthChallenge { lodge_id: Uuid, challenge: Vec<u8> },
    SendAuthResponse { lodge_id: Uuid, proof: Vec<u8> },
}

pub mod lodgenet;
pub mod auth;
pub use lodgenet::NetworkManager;

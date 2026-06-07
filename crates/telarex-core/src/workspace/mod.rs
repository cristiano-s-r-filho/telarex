use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum MemberRole {
    Admin,
    Member,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkspaceMember {
    pub peer_id: String,
    pub username: String,
    pub public_key: Vec<u8>,
    pub role: MemberRole,
    pub joined_at: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PendingJoin {
    pub peer_id: String,
    pub username: String,
    pub public_key: Vec<u8>,
    pub requested_at: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum JoinPolicy {
    Open,
    Approval,
    Invite,
}

impl Default for JoinPolicy {
    fn default() -> Self {
        Self::Approval
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Workspace {
    pub id: Uuid,
    pub root: PathBuf,
    pub is_shared: bool,
    pub members: Vec<WorkspaceMember>,
    pub pending_joins: Vec<PendingJoin>,
    pub join_policy: JoinPolicy,
    pub name: String,
}

impl Workspace {
    pub fn new(root: PathBuf) -> Self {
        Self {
            id: Uuid::new_v4(),
            root,
            is_shared: false,
            members: Vec::new(),
            pending_joins: Vec::new(),
            join_policy: JoinPolicy::default(),
            name: String::new(),
        }
    }

    pub fn share(&mut self, name: String) {
        self.is_shared = true;
        self.name = name;
    }

    pub fn add_member(&mut self, peer_id: String, username: String, public_key: Vec<u8>, role: MemberRole) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.members.push(WorkspaceMember {
            peer_id,
            username,
            public_key,
            role,
            joined_at: now,
        });
    }

    pub fn add_pending_join(&mut self, peer_id: String, username: String, public_key: Vec<u8>) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.pending_joins.push(PendingJoin {
            peer_id,
            username,
            public_key,
            requested_at: now,
        });
    }

    pub fn approve_join(&mut self, peer_id: &str) -> Option<PendingJoin> {
        if let Some(pos) = self.pending_joins.iter().position(|j| j.peer_id == peer_id) {
            let join_req = self.pending_joins.remove(pos);
            self.add_member(join_req.peer_id.clone(), join_req.username.clone(), join_req.public_key.clone(), MemberRole::Member);
            Some(join_req)
        } else {
            None
        }
    }

    pub fn reject_join(&mut self, peer_id: &str) {
        self.pending_joins.retain(|j| j.peer_id != peer_id);
    }
}

//! Workspace — shared workspace (lodge) model for collaborative editing sessions.
//!
//! [`Workspace`] represents a lodge: a directory shared with a group of peers.
//! It tracks members, pending join requests, and the join policy.
//! [`WorkspaceMember`], [`PendingJoin`], [`MemberRole`], and [`JoinPolicy`]
//! provide the data model for access control and membership management.

use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Role a member can have within a workspace.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum MemberRole {
    Admin,
    /// Standard member with no administrative privileges.
    Member,
}

/// A member of a shared workspace (lodge).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkspaceMember {
    pub peer_id: String,
    pub username: String,
    pub public_key: Vec<u8>,
    pub role: MemberRole,
    pub joined_at: u64,
}

/// A pending request from a peer to join a workspace.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PendingJoin {
    pub peer_id: String,
    pub username: String,
    pub public_key: Vec<u8>,
    pub requested_at: u64,
}

/// Policy controlling how new peers can join a workspace.
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

/// A shared workspace (lodge) with members, pending joins, and access policy.
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
    /// Create a new workspace rooted at the given directory.
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

    /// Mark this workspace as shared and give it a display name.
    pub fn share(&mut self, name: String) {
        self.is_shared = true;
        self.name = name;
    }

    /// Add a member to this workspace with the given role.
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

    /// Record a pending join request from a peer.
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

    /// Approve a pending join request, promoting it to a full member.
    pub fn approve_join(&mut self, peer_id: &str) -> Option<PendingJoin> {
        if let Some(pos) = self.pending_joins.iter().position(|j| j.peer_id == peer_id) {
            let join_req = self.pending_joins.remove(pos);
            self.add_member(join_req.peer_id.clone(), join_req.username.clone(), join_req.public_key.clone(), MemberRole::Member);
            Some(join_req)
        } else {
            None
        }
    }

    /// Reject a pending join request, removing it from the queue.
    pub fn reject_join(&mut self, peer_id: &str) {
        self.pending_joins.retain(|j| j.peer_id != peer_id);
    }
}

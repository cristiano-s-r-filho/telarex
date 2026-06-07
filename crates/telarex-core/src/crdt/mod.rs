//! CRDT — Automerge-based conflict-free replicated data types for collaborative editing.
//!
//! CRDT-based document synchronisation.
//!
//! [`SyncEngine`](sync_engine::SyncEngine) manages synchronised documents with text and cursor objects,
//! enabling multiple peers to concurrently edit the same file and converge
//! to a consistent state without a central server.

pub mod sync_engine; 

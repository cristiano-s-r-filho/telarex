//! TelaRex Core — shared logic, state management, and all subsystems.
//!
//! This crate is the engine of TelaRex. It provides:
//! - **Buffer management** — document editing, undo/redo, macros, and motions
//! - **Syntax highlighting** — Tree-sitter-based highlighting with themeable stylesheets
//! - **Configuration** — TOML-based settings with schema validation and theme engine
//! - **Networking** — P2P lodge discovery and sync via libp2p (gossipsub, mDNS, Kademlia)
//! - **CRDT** — Automerge-based document synchronization for collaborative editing
//! - **Actor system** — async buffer actor for thread-safe buffer access
//! - **Workspace** — shared workspace (lodge) model with member/join management
//! - **LSP** — Language Server Protocol client for completions and diagnostics
//! - **Database** — SQLite persistence for lodges, sessions, and recent projects
//! - **Errors** — typed error framework with codes, levels, and suggested solutions
#![allow(dead_code)]
pub mod buffer;
pub mod clipboard;
pub mod command;
#[cfg(feature = "git")]
pub mod git_sidecar;
pub mod config;
pub mod crdt;
pub mod syntax;
pub mod workspace;
pub mod actor;
pub mod lsp;
pub mod database;
pub mod network;
pub mod errors;

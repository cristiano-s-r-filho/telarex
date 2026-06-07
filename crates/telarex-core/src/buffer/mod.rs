//! Buffer management — document editing, undo/redo history, macros, cursor motions,
//! and a managed buffer system with Tree-sitter integration.
//!
//! The central types are:
//! - [`Document`] — a text document backed by a rope with history-aware editing
//! - [`ManagedBuffer`] — a buffer pairing text content with its syntax tree
//! - [`BufferManager`] — a registry that caches and deduplicates buffers by path

pub mod document;
pub mod history;
pub mod macro_engine;
pub mod motions;
pub mod buffer_manager;
pub mod managed;

pub use document::Document;
pub use buffer_manager::BufferManager;
pub use managed::{ManagedBuffer, BufferCommand};

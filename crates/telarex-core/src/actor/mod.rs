//! Actor system — async buffer actor for thread-safe buffer lifecycle management.
//!
//! [`BufferActor`] runs in a Tokio task and owns the authoritative copies of all
//! open buffers. Callers send [`BufferActorCommand`] messages via an `mpsc` channel
//! to get or create buffers, apply edits, or close them.

use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::mpsc;
use crate::buffer::{ManagedBuffer, BufferCommand};
use tree_sitter::Parser;
use std::sync::{Arc, Mutex};

/// Messages sent to the BufferActor
pub enum BufferActorCommand {
    /// Load or create a buffer for a given path. Returns a handle to the SHARED buffer.
    GetOrCreate { 
        path: PathBuf, 
        reply: std::sync::mpsc::Sender<Arc<Mutex<ManagedBuffer>>> 
    },
    /// Direct command to a buffer
    BufferCmd {
        path: PathBuf,
        cmd: BufferCommand,
    },
    /// Close a buffer
    Close { path: PathBuf },
}

/// Manages the authoritative copies of all open buffers in a Tokio task.
pub struct BufferActor {
    buffers: HashMap<PathBuf, Arc<Mutex<ManagedBuffer>>>,
    receiver: mpsc::Receiver<BufferActorCommand>,
}

impl BufferActor {
    /// Spawn the actor in a Tokio task and return a channel sender for commands.
    pub fn start() -> mpsc::Sender<BufferActorCommand> {
        let (tx, rx) = mpsc::channel(100);
        tokio::spawn(async move {
            let mut actor = Self {
                buffers: HashMap::new(),
                receiver: rx,
            };
            actor.run().await;
        });
        tx
    }

    async fn run(&mut self) {
        let mut parser = Parser::new();
        // Default to Rust
        let _ = parser.set_language(&tree_sitter_rust::LANGUAGE.into());

        while let Some(cmd) = self.receiver.recv().await {
            match cmd {
                BufferActorCommand::GetOrCreate { path, reply } => {
                    let buffer = self.buffers.entry(path.clone()).or_insert_with(|| {
                        let rope = if path.exists() {
                            std::fs::read_to_string(&path)
                                .map(|s| ropey::Rope::from_str(&s))
                                .unwrap_or_else(|_| ropey::Rope::new())
                        } else {
                            ropey::Rope::new()
                        };
                        let mut b = ManagedBuffer::from_rope(rope);
                        b.path = Some(path.clone());
                        b.parse(&mut parser);
                        Arc::new(Mutex::new(b))
                    });
                    let _ = reply.send(buffer.clone());
                }
                BufferActorCommand::BufferCmd { path, cmd } => {
                    if let Some(shared) = self.buffers.get(&path) {
                        let mut b = shared.lock().unwrap();
                        b.handle_command(cmd, &mut parser);
                    }
                }
                BufferActorCommand::Close { path } => {
                    self.buffers.remove(&path);
                }
            }
        }
    }
}

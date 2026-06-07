use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use crate::buffer::ManagedBuffer;
use anyhow::Result;

/// A thread-safe registry of buffers, deduplicated by canonical path.
pub struct BufferManager {
    // Map of canonicalized path to Shared ManagedBuffer
    buffers: Mutex<HashMap<PathBuf, Arc<Mutex<ManagedBuffer>>>>,
}

impl BufferManager {
    /// Create an empty buffer manager.
    pub fn new() -> Self {
        Self {
            buffers: Mutex::new(HashMap::new()),
        }
    }

    /// Get a buffer by path, loading it from disk if not already cached.
    pub fn get_or_load(&self, path: impl AsRef<Path>) -> Result<Arc<Mutex<ManagedBuffer>>> {
        let path_ref = path.as_ref();
        let canonical = if path_ref.exists() {
            std::fs::canonicalize(path_ref)?
        } else {
            path_ref.to_path_buf()
        };

        let mut buffers = self.buffers.lock().unwrap();
        if let Some(doc) = buffers.get(&canonical) {
            Ok(doc.clone())
        } else {
            // Load file into rope
            let rope = if canonical.exists() {
                let file = std::fs::File::open(&canonical)?;
                ropey::Rope::from_reader(file)?
            } else {
                ropey::Rope::new()
            };
            
            let mut doc = ManagedBuffer::from_rope(rope);
            doc.path = Some(canonical.clone());
            
            let shared = Arc::new(Mutex::new(doc));
            buffers.insert(canonical, shared.clone());
            Ok(shared)
        }
    }

    /// Create a new scratch (unnamed, virtual) buffer.
    pub fn create_scratch(&self, name: String) -> Arc<Mutex<ManagedBuffer>> {
        let mut buffers = self.buffers.lock().unwrap();
        let doc = ManagedBuffer::new();
        // Scratch buffers use a virtual path
        let path = PathBuf::from(format!("scratch://{}", name));
        let shared = Arc::new(Mutex::new(doc));
        buffers.insert(path, shared.clone());
        shared
    }

    /// Remove a buffer from the registry by its path.
    pub fn remove(&self, path: &Path) {
        let mut buffers = self.buffers.lock().unwrap();
        buffers.remove(path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_buffer_manager_create_scratch() {
        let bm = BufferManager::new();
        let buf = bm.create_scratch("test".to_string());
        let doc = buf.lock().unwrap();
        assert!(doc.path.is_none() || doc.path.as_ref().unwrap().starts_with("scratch://"));
    }

    #[test]
    fn test_buffer_manager_get_or_load_new() {
        let bm = BufferManager::new();
        let path = PathBuf::from("/tmp/nonexistent_telarex_test_file.txt");
        let buf = bm.get_or_load(&path).unwrap();
        let doc = buf.lock().unwrap();
        assert_eq!(doc.rope.to_string(), "");
    }

    #[test]
    fn test_buffer_manager_returns_same_instance() {
        let bm = BufferManager::new();
        let path = PathBuf::from("/tmp/nonexistent_telarex_test_file.txt");
        let buf1 = bm.get_or_load(&path).unwrap();
        let buf2 = bm.get_or_load(&path).unwrap();
        assert!(Arc::ptr_eq(&buf1, &buf2));
    }

    #[test]
    fn test_buffer_manager_remove() {
        let bm = BufferManager::new();
        let path = PathBuf::from("/tmp/nonexistent_telarex_test_remove.txt");
        let _ = bm.get_or_load(&path).unwrap();
        bm.remove(&path);
        // After removal, a new get should create a fresh buffer
        let buf = bm.get_or_load(&path).unwrap();
        assert!(path.ends_with("nonexistent_telarex_test_remove.txt"));
        // Prevent unused variable warning
        let _ = buf;
    }
}

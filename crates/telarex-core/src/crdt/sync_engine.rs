use std::path::PathBuf;
use automerge::{AutoCommit, ReadDoc, transaction::Transactable, sync::SyncDoc, ObjType};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

pub struct SyncEngine {
    pub documents: Arc<Mutex<Vec<ManagedDocument>>>,
}

pub struct ManagedDocument {
    pub path: PathBuf,
    pub doc: AutoCommit,
    pub text_obj: automerge::ObjId,
    pub cursor_obj: automerge::ObjId,
}

impl SyncEngine {
    pub fn new() -> Self {
        Self {
            documents: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn register_document(&self, path: PathBuf) {
        let mut docs = self.documents.lock().unwrap();
        if !docs.iter().any(|d| d.path == path) {
            let mut doc = AutoCommit::new();
            let text_obj = doc.put_object(automerge::ROOT, "content", ObjType::Text).unwrap();
            let cursor_obj = doc.put_object(automerge::ROOT, "cursors", ObjType::Map).unwrap();
            docs.push(ManagedDocument {
                path,
                doc,
                text_obj,
                cursor_obj,
            });
        }
    }

    pub fn apply_local_splice(&self, path: &PathBuf, pos: usize, del: isize, text: &str) {
        let mut docs = self.documents.lock().unwrap();
        if let Some(managed) = docs.iter_mut().find(|d| d.path == *path) {
            let obj = managed.text_obj.clone();
            let _ = managed.doc.splice_text(obj, pos, del, text);
        }
    }

    pub fn apply_local_full(&self, path: &PathBuf, text: &str) {
        let mut docs = self.documents.lock().unwrap();
        if let Some(managed) = docs.iter_mut().find(|d| d.path == *path) {
            let obj = managed.text_obj.clone();
            let len = managed.doc.length(&obj);
            let _ = managed.doc.splice_text(obj, 0, len as isize, text);
        }
    }

    pub fn update_cursor(&self, path: &PathBuf, peer_id: &str, pos: usize) {
        let mut docs = self.documents.lock().unwrap();
        if let Some(managed) = docs.iter_mut().find(|d| d.path == *path) {
            let obj = managed.cursor_obj.clone();
            let _ = managed.doc.put(obj, peer_id, pos as u64);
        }
    }

    pub fn generate_sync_message(&self, path: &PathBuf, state: &mut automerge::sync::State) -> Option<automerge::sync::Message> {
        let mut docs = self.documents.lock().unwrap();
        let managed = docs.iter_mut().find(|d| d.path == *path)?;
        let msg = managed.doc.sync().generate_sync_message(state);
        msg
    }

    pub fn receive_sync_message(&self, path: &PathBuf, state: &mut automerge::sync::State, msg: automerge::sync::Message) {
        let mut docs = self.documents.lock().unwrap();
        if let Some(managed) = docs.iter_mut().find(|d| d.path == *path) {
            let _ = managed.doc.sync().receive_sync_message(state, msg);
        }
    }

    pub fn get_content(&self, path: &PathBuf) -> Option<String> {
        let docs = self.documents.lock().unwrap();
        let managed = docs.iter().find(|d| d.path == *path)?;
        managed.doc.text(&managed.text_obj).ok()
    }

    pub fn get_cursors(&self, path: &PathBuf) -> HashMap<String, usize> {
        let mut cursors = HashMap::new();
        let docs = self.documents.lock().unwrap();
        if let Some(managed) = docs.iter().find(|d| d.path == *path) {
            let keys = managed.doc.keys(&managed.cursor_obj);
            for key in keys {
                if let Ok(Some((automerge::Value::Scalar(v), _))) = managed.doc.get(&managed.cursor_obj, &key) {
                    if let automerge::ScalarValue::Uint(pos) = *v {
                        cursors.insert(key, pos as usize);
                    }
                }
            }
        }
        cursors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_engine_consistency() {
        let engine_a = SyncEngine::new();
        let engine_b = SyncEngine::new();
        let path = PathBuf::from("test.rs");

        engine_a.register_document(path.clone());
        engine_a.apply_local_splice(&path, 0, 0, "Hello");

        // Simulate Peer B joining by loading A's current state
        {
            let mut docs_a = engine_a.documents.lock().unwrap();
            let bytes = docs_a[0].doc.save();
            
            let mut docs_b = engine_b.documents.lock().unwrap();
            let doc_b = AutoCommit::load(&bytes).unwrap();
            let (_, text_obj) = doc_b.get(automerge::ROOT, "content").unwrap().unwrap();
            let (_, cursor_obj) = doc_b.get(automerge::ROOT, "cursors").unwrap().unwrap();
            
            docs_b.push(ManagedDocument {
                path: path.clone(),
                doc: doc_b,
                text_obj,
                cursor_obj,
            });
        }

        assert_eq!(engine_b.get_content(&path), Some("Hello".to_string()));

        // Test incremental sync
        engine_a.apply_local_splice(&path, 5, 0, " World");
        
        let mut state_a = automerge::sync::State::new();
        let mut state_b = automerge::sync::State::new();
        
        loop {
            let mut changed = false;
            if let Some(msg) = engine_a.generate_sync_message(&path, &mut state_a) {
                engine_b.receive_sync_message(&path, &mut state_b, msg);
                changed = true;
            }
            if let Some(msg) = engine_b.generate_sync_message(&path, &mut state_b) {
                engine_a.receive_sync_message(&path, &mut state_a, msg);
                changed = true;
            }
            if !changed { break; }
        }

        assert_eq!(engine_b.get_content(&path), Some("Hello World".to_string()));
    }
}

impl Default for SyncEngine {
    fn default() -> Self {
        Self::new()
    }
}

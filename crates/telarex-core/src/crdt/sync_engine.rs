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

    fn peer_from(engine: &SyncEngine, path: &PathBuf, idx: usize) -> ManagedDocument {
        let mut docs = engine.documents.lock().unwrap();
        let bytes = docs[idx].doc.save();
        let doc = AutoCommit::load(&bytes).unwrap();
        let (_, text_obj) = doc.get(automerge::ROOT, "content").unwrap().unwrap();
        let (_, cursor_obj) = doc.get(automerge::ROOT, "cursors").unwrap().unwrap();
        ManagedDocument {
            path: path.clone(),
            doc,
            text_obj,
            cursor_obj,
        }
    }

    fn push_doc(engine: &SyncEngine, doc: ManagedDocument) {
        engine.documents.lock().unwrap().push(doc);
    }

    fn sync_all(engines: &[&SyncEngine], path: &PathBuf) {
        let n = engines.len();
        let mut states: Vec<Vec<automerge::sync::State>> = (0..n).map(|_| (0..n).map(|_| automerge::sync::State::new()).collect()).collect();

        for _round in 0..10 {
            let mut any = false;
            for i in 0..n {
                for j in 0..n {
                    if i == j { continue; }
                    if let Some(msg) = engines[i].generate_sync_message(path, &mut states[i][j]) {
                        engines[j].receive_sync_message(path, &mut states[j][i], msg);
                        any = true;
                    }
                }
            }
            if !any { break; }
        }
    }

    fn converged(engines: &[&SyncEngine], path: &PathBuf) -> Vec<String> {
        engines.iter().map(|e| e.get_content(path).unwrap_or_default()).collect()
    }

    #[test]
    fn test_sync_one_way_initial() {
        let a = SyncEngine::new();
        let b = SyncEngine::new();
        let path = PathBuf::from("a.rs");

        a.register_document(path.clone());
        a.apply_local_splice(&path, 0, 0, "Hello");
        push_doc(&b, peer_from(&a, &path, 0));

        assert_eq!(b.get_content(&path), Some("Hello".to_string()));
    }

    #[test]
    fn test_sync_incremental() {
        let a = SyncEngine::new();
        let b = SyncEngine::new();
        let path = PathBuf::from("b.rs");

        a.register_document(path.clone());
        a.apply_local_splice(&path, 0, 0, "Hello");
        push_doc(&b, peer_from(&a, &path, 0));
        a.apply_local_splice(&path, 5, 0, " World");

        sync_all(&[&a, &b], &path);
        let contents = converged(&[&a, &b], &path);
        assert_eq!(contents[0], "Hello World");
        assert_eq!(contents[0], contents[1]);
    }

    #[test]
    fn test_three_way_concurrent_merge() {
        let a = SyncEngine::new();
        let b = SyncEngine::new();
        let c = SyncEngine::new();
        let path = PathBuf::from("three.rs");

        a.register_document(path.clone());
        a.apply_local_splice(&path, 0, 0, "base");
        push_doc(&b, peer_from(&a, &path, 0));
        push_doc(&c, peer_from(&a, &path, 0));

        a.apply_local_splice(&path, 4, 0, " AAA");
        b.apply_local_splice(&path, 4, 0, " BBB");
        c.apply_local_splice(&path, 4, 0, " CCC");

        sync_all(&[&a, &b, &c], &path);
        let contents = converged(&[&a, &b, &c], &path);
        assert_eq!(contents[0], contents[1]);
        assert_eq!(contents[1], contents[2]);
        assert!(contents[0].contains("AAA"));
        assert!(contents[0].contains("BBB"));
        assert!(contents[0].contains("CCC"));
    }

    #[test]
    fn test_concurrent_insert_same_position() {
        let a = SyncEngine::new();
        let b = SyncEngine::new();
        let path = PathBuf::from("conflict.rs");

        a.register_document(path.clone());
        a.apply_local_splice(&path, 0, 0, "[]");
        push_doc(&b, peer_from(&a, &path, 0));

        a.apply_local_splice(&path, 1, 0, "AAA");
        b.apply_local_splice(&path, 1, 0, "BBB");

        sync_all(&[&a, &b], &path);
        let contents = converged(&[&a, &b], &path);
        assert_eq!(contents[0], contents[1]);
        assert!(contents[0].contains("AAA"));
        assert!(contents[0].contains("BBB"));
    }

    #[test]
    fn test_concurrent_delete_and_insert() {
        let a = SyncEngine::new();
        let b = SyncEngine::new();
        let path = PathBuf::from("delins.rs");

        a.register_document(path.clone());
        a.apply_local_splice(&path, 0, 0, "Hello World");
        push_doc(&b, peer_from(&a, &path, 0));

        a.apply_local_splice(&path, 6, 5, "");
        b.apply_local_splice(&path, 11, 0, "!!!");

        sync_all(&[&a, &b], &path);
        let contents = converged(&[&a, &b], &path);
        assert_eq!(contents[0], contents[1]);
    }

    #[test]
    fn test_causal_delivery_chain() {
        let a = SyncEngine::new();
        let b = SyncEngine::new();
        let c = SyncEngine::new();
        let path = PathBuf::from("causal.rs");

        a.register_document(path.clone());
        a.apply_local_splice(&path, 0, 0, "");
        push_doc(&b, peer_from(&a, &path, 0));

        a.apply_local_splice(&path, 0, 0, "alpha-");
        sync_all(&[&a, &b], &path);

        b.apply_local_splice(&path, 6, 0, "beta-");
        push_doc(&c, peer_from(&b, &path, 0));

        sync_all(&[&a, &b, &c], &path);
        let contents = converged(&[&a, &b, &c], &path);
        assert_eq!(contents[0], contents[1]);
        assert_eq!(contents[1], contents[2]);
        assert!(contents[0].contains("alpha"));
    }

    #[test]
    fn test_cursor_sync() {
        let a = SyncEngine::new();
        let b = SyncEngine::new();
        let path = PathBuf::from("cursor.rs");

        a.register_document(path.clone());
        a.apply_local_splice(&path, 0, 0, "Hello World");
        push_doc(&b, peer_from(&a, &path, 0));

        a.update_cursor(&path, "peer_a", 3);
        sync_all(&[&a, &b], &path);

        let cursors = b.get_cursors(&path);
        assert_eq!(cursors.get("peer_a"), Some(&3));
    }

    #[test]
    fn test_many_rounds_convergence() {
        let a = SyncEngine::new();
        let b = SyncEngine::new();
        let path = PathBuf::from("stress.rs");

        a.register_document(path.clone());
        a.apply_local_splice(&path, 0, 0, "");
        push_doc(&b, peer_from(&a, &path, 0));

        for i in 0..20 {
            a.apply_local_splice(&path, i as usize, 0, &char::from(b'a' + (i % 26) as u8).to_string());
        }
        for i in 0..15 {
            b.apply_local_splice(&path, i as usize, 0, &char::from(b'A' + (i % 26) as u8).to_string());
        }

        sync_all(&[&a, &b], &path);
        let contents = converged(&[&a, &b], &path);
        assert_eq!(contents[0], contents[1]);
        assert!(contents[0].len() >= 35);
    }
}

impl Default for SyncEngine {
    fn default() -> Self {
        Self::new()
    }
}

use telarex_core::crdt::sync_engine::{SyncEngine, ManagedDocument};
use std::path::PathBuf;
use automerge::sync::State;
use automerge::{AutoCommit, ReadDoc};

#[test]
fn test_multipeer_sync_integration() {
    let engine_a = SyncEngine::new();
    let engine_b = SyncEngine::new();
    let path = PathBuf::from("shared.txt");

    // Peer A starts the document
    engine_a.register_document(path.clone());
    engine_a.apply_local_splice(&path, 0, 0, "Initial");

    // Peer B joins (Simulated by state transfer)
    {
        let mut docs_a = engine_a.documents.lock().unwrap();
        let bytes = docs_a[0].doc.save();
        
        let mut docs_b = engine_b.documents.lock().unwrap();
        let doc_b = AutoCommit::load(&bytes).expect("Failed to load Peer A state");
        let (_, text_obj) = doc_b.get(automerge::ROOT, "content").unwrap().unwrap();
        let (_, cursor_obj) = doc_b.get(automerge::ROOT, "cursors").unwrap().unwrap();
        
        docs_b.push(ManagedDocument {
            path: path.clone(),
            doc: doc_b,
            text_obj,
            cursor_obj,
        });
    }

    // Now both have "Initial"
    assert_eq!(engine_b.get_content(&path), Some("Initial".to_string()));

    // 1. Peer A types "Hello"
    engine_a.apply_local_splice(&path, 7, 0, " World");

    // 2. Mock network exchange
    let mut state_a = State::new();
    let mut state_b = State::new();

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

    // Verify convergence
    assert_eq!(engine_b.get_content(&path), Some("Initial World".to_string()));

    // 3. Concurrent edits
    engine_a.apply_local_splice(&path, 13, 0, " from A");
    engine_b.apply_local_splice(&path, 0, 0, "B says ");

    // Final exchange
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

    let final_a = engine_a.get_content(&path).unwrap();
    let final_b = engine_b.get_content(&path).unwrap();

    assert_eq!(final_a, final_b);
}

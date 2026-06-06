use telarex_core::crdt::sync_engine::{SyncEngine, ManagedDocument};
use std::path::PathBuf;
use automerge::{AutoCommit, ReadDoc};

fn peer_from(engine: &SyncEngine, path: &PathBuf, idx: usize) -> ManagedDocument {
    let docs = engine.documents.lock().unwrap();
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
    let mut states: Vec<automerge::sync::State> = (0..n).map(|_| automerge::sync::State::new()).collect();
    for _round in 0..10 {
        let mut any = false;
        for i in 0..n {
            for j in 0..n {
                if i == j { continue; }
                if let Some(msg) = engines[i].generate_sync_message(path, &mut states[i]) {
                    engines[j].receive_sync_message(path, &mut states[j], msg);
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
fn test_headless_two_peer_full_flow() {
    let alice = SyncEngine::new();
    let bob = SyncEngine::new();
    let path = PathBuf::from("e2e.txt");

    // Alice creates document
    alice.register_document(path.clone());
    alice.apply_local_splice(&path, 0, 0, "Hello World");

    // Bob joins via state transfer
    push_doc(&bob, peer_from(&alice, &path, 0));
    assert_eq!(bob.get_content(&path), Some("Hello World".to_string()));

    // Alice edits, Bob edits concurrently
    alice.apply_local_splice(&path, 6, 5, "CRDT");
    bob.apply_local_splice(&path, 0, 0, ">> ");

    // Full sync exchange
    sync_all(&[&alice, &bob], &path);
    let result = converged(&[&alice, &bob], &path);
    assert_eq!(result[0], result[1], "Alice and Bob must converge");
    assert!(result[0].contains("CRDT"), "Alice's edit must survive");
}

#[test]
fn test_headless_three_peer_convergence() {
    let a = SyncEngine::new();
    let b = SyncEngine::new();
    let c = SyncEngine::new();
    let path = PathBuf::from("three_e2e.txt");

    a.register_document(path.clone());
    a.apply_local_splice(&path, 0, 0, "Start");
    push_doc(&b, peer_from(&a, &path, 0));
    push_doc(&c, peer_from(&a, &path, 0));

    // Each peer makes non-overlapping edits
    a.apply_local_splice(&path, 5, 0, " [Alice]");
    b.apply_local_splice(&path, 5, 0, " [Bob]");
    c.apply_local_splice(&path, 5, 0, " [Carol]");

    sync_all(&[&a, &b, &c], &path);
    let contents = converged(&[&a, &b, &c], &path);
    assert_eq!(contents[0], contents[1]);
    assert_eq!(contents[1], contents[2]);
    assert!(contents[0].contains("Alice"));
    assert!(contents[0].contains("Bob"));
    assert!(contents[0].contains("Carol"));
}

#[test]
fn test_headless_cursor_sync_between_peers() {
    let alice = SyncEngine::new();
    let bob = SyncEngine::new();
    let path = PathBuf::from("cursor_e2e.txt");

    alice.register_document(path.clone());
    alice.apply_local_splice(&path, 0, 0, "Collaborative Editing");
    push_doc(&bob, peer_from(&alice, &path, 0));

    alice.update_cursor(&path, "alice", 5);
    bob.update_cursor(&path, "bob", 12);

    sync_all(&[&alice, &bob], &path);

    let alice_cursors = alice.get_cursors(&path);
    let bob_cursors = bob.get_cursors(&path);

    assert_eq!(alice_cursors.get("alice"), Some(&5));
    assert_eq!(alice_cursors.get("bob"), Some(&12));
    assert_eq!(bob_cursors.get("alice"), Some(&5));
    assert_eq!(bob_cursors.get("bob"), Some(&12));
}

#[test]
fn test_headless_conflict_resolution() {
    let a = SyncEngine::new();
    let b = SyncEngine::new();
    let path = PathBuf::from("conflict_e2e.txt");

    a.register_document(path.clone());
    a.apply_local_splice(&path, 0, 0, "....");
    push_doc(&b, peer_from(&a, &path, 0));

    // Both insert different text at the exact same position
    a.apply_local_splice(&path, 2, 0, "LEFT");
    b.apply_local_splice(&path, 2, 0, "RGHT");

    sync_all(&[&a, &b], &path);
    let contents = converged(&[&a, &b], &path);
    assert_eq!(contents[0], contents[1]);
    assert!(contents[0].contains("LEFT"));
    assert!(contents[0].contains("RGHT"));
}

#[test]
fn test_headless_multi_round_sync() {
    let alice = SyncEngine::new();
    let bob = SyncEngine::new();
    let path = PathBuf::from("multi_e2e.txt");

    alice.register_document(path.clone());
    alice.apply_local_splice(&path, 0, 0, "a");
    push_doc(&bob, peer_from(&alice, &path, 0));

    for i in 2..=10 {
        let ch = char::from(i + 96);
        alice.apply_local_splice(&path, i as usize - 1, 0, &ch.to_string());
        sync_all(&[&alice, &bob], &path);
        let contents = converged(&[&alice, &bob], &path);
        assert_eq!(contents[0], contents[1], "round {} should converge", i);
    }

    let final_content = alice.get_content(&path).unwrap();
    assert_eq!(final_content.len(), 10);
}

use telarex_core::buffer::{ManagedBuffer, BufferCommand};
use telarex_core::actor::{BufferActor, BufferActorCommand};
use tree_sitter::Parser;
use tokio::sync::oneshot;

#[test]
fn test_managed_buffer_incremental_edit() {
    let mut parser = Parser::new();
    let _ = parser.set_language(&tree_sitter_rust::LANGUAGE.into());
    
    let mut buffer = ManagedBuffer::from_rope(ropey::Rope::from_str("fn main() {}"));
    buffer.parse(&mut parser);
    
    assert!(buffer.tree.is_some());
    let old_tree = buffer.tree.clone().unwrap();
    
    // Apply edit: insert "let x = 1;" inside the block
    buffer.handle_command(BufferCommand::ApplyEdit { 
        start_char: 11, 
        del_chars: 0, 
        new_text: "let x = 1;".to_string() 
    }, &mut parser);
    
    assert_eq!(buffer.rope.to_string(), "fn main() {let x = 1;}");
    assert!(buffer.tree.is_some());
    
    // Verify that the tree actually changed
    let new_tree = buffer.tree.unwrap();
    assert_ne!(old_tree.root_node().to_sexp(), new_tree.root_node().to_sexp());
}

#[test]
fn test_managed_buffer_full_reset() {
    let mut parser = Parser::new();
    let _ = parser.set_language(&tree_sitter_rust::LANGUAGE.into());
    
    let mut buffer = ManagedBuffer::new();
    buffer.handle_command(BufferCommand::SetText("pub struct Hello;".to_string()), &mut parser);
    
    assert_eq!(buffer.rope.to_string(), "pub struct Hello;");
    assert!(buffer.tree.is_some());
}

#[tokio::test]
async fn test_buffer_actor_lifecycle() {
    let buffer_tx = BufferActor::start();
    let path = std::path::PathBuf::from("test.rs");
    
    let (reply_tx, reply_rx) = oneshot::channel();
    let _ = buffer_tx.send(BufferActorCommand::GetOrCreate { 
        path: path.clone(), 
        reply: reply_tx 
    }).await;
    
    let doc_tx = reply_rx.await.unwrap();
    
    // Send an edit
    let _ = doc_tx.send(BufferCommand::SetText("fn test() {}".to_string())).await;
    
    // Give some time for the actor to process (in a real test we'd use a feedback channel)
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    
    // Get snapshot
    let (snap_tx, snap_rx) = oneshot::channel();
    let _ = buffer_tx.send(BufferActorCommand::GetSnapshot { 
        path: path.clone(), 
        reply: snap_tx 
    }).await;
    
    let rope = snap_rx.await.unwrap().unwrap();
    assert_eq!(rope.to_string(), "fn test() {}");
}

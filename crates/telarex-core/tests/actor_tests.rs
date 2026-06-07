use telarex_core::buffer::{ManagedBuffer, BufferCommand};
use telarex_core::actor::BufferActorCommand;
use tree_sitter::Parser;
use std::sync::{Arc, Mutex};

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

#[tokio::test(flavor = "multi_thread")]
async fn test_buffer_actor_lifecycle() {
    let buffer_tx = telarex_core::actor::BufferActor::start();
    let path = std::path::PathBuf::from("test.rs");
    
    let (reply_tx, reply_rx) = std::sync::mpsc::channel();
    let _ = buffer_tx.send(BufferActorCommand::GetOrCreate { 
        path: path.clone(), 
        reply: reply_tx 
    }).await;
    
    let doc_arc: Arc<Mutex<ManagedBuffer>> = reply_rx.recv().unwrap();
    
    // Send an edit
    {
        let mut b = doc_arc.lock().unwrap();
        let mut parser = Parser::new();
        let _ = parser.set_language(&tree_sitter_rust::LANGUAGE.into());
        b.handle_command(BufferCommand::SetText("fn test() {}".to_string()), &mut parser);
    }
    
    // Verify via direct buffer access
    let b = doc_arc.lock().unwrap();
    assert_eq!(b.rope.to_string(), "fn test() {}");
}

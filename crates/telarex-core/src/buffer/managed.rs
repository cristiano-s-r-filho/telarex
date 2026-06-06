use ropey::Rope;
use tree_sitter::{InputEdit, Parser, Point, Tree};
use std::path::PathBuf;

#[derive(Debug)]
pub enum BufferCommand {
    ApplyEdit { start_char: usize, del_chars: usize, new_text: String },
    SetText(String),
    Save,
}

/// A buffer that manages both the raw text (Rope) and its syntax tree (Tree-sitter).
/// This is the 'Source of Truth' for all editing operations.
pub struct ManagedBuffer {
    pub rope: Rope,
    pub tree: Option<Tree>,
    pub path: Option<PathBuf>,
    pub modified: bool,
    /// MONOTONIC VERSION: Tracks every edit to synchronize with async consumers (like highlighters)
    pub version: u64,
}

impl ManagedBuffer {
    pub fn new() -> Self {
        Self {
            rope: Rope::new(),
            tree: None,
            path: None,
            modified: false,
            version: 0,
        }
    }

    pub fn from_rope(rope: Rope) -> Self {
        Self {
            rope,
            tree: None,
            path: None,
            modified: false,
            version: 0,
        }
    }

    pub fn handle_command(&mut self, cmd: BufferCommand, parser: &mut Parser) {
        match cmd {
            BufferCommand::ApplyEdit { start_char, del_chars, new_text } => {
                self.apply_edit(start_char, del_chars, &new_text);
                self.parse(parser);
            }
            BufferCommand::SetText(text) => {
                self.rope = Rope::from_str(&text);
                self.tree = None; // Reset tree for full re-parse
                self.parse(parser);
                self.modified = true;
                self.version += 1;
            }
            BufferCommand::Save => {
                if let Some(path) = &self.path {
                    if let Ok(mut file) = std::fs::File::create(path) {
                        let _ = self.rope.write_to(&mut file);
                        self.modified = false;
                    }
                }
            }
        }
    }

    /// Performs an incremental parse of the buffer.
    pub fn parse(&mut self, parser: &mut Parser) {
        let mut callback = |byte_offset: usize, _position: Point| -> &[u8] {
            if byte_offset >= self.rope.len_bytes() {
                return &[];
            }
            if let Some((chunk, chunk_byte_idx, _, _)) = self.rope.get_chunk_at_byte(byte_offset) {
                let offset_in_chunk = byte_offset - chunk_byte_idx;
                &chunk.as_bytes()[offset_in_chunk..]
            } else {
                &[]
            }
        };

        self.tree = parser.parse_with(&mut callback, self.tree.as_ref());
    }

    /// Appies an edit to both the Rope and the syntax Tree.
    pub fn apply_edit(&mut self, start_char: usize, del_chars: usize, new_text: &str) {
        // 1. Convert char offsets to byte offsets for Tree-sitter
        let start_byte = self.rope.char_to_byte(start_char);
        let old_end_byte = self.rope.char_to_byte(start_char + del_chars);
        let new_end_byte = start_byte + new_text.len();

        // 2. Capture positions (Points) for Tree-sitter
        let start_position = char_to_point(&self.rope, start_char);
        let old_end_position = char_to_point(&self.rope, start_char + del_chars);

        // 3. Update the Rope
        if del_chars > 0 {
            self.rope.remove(start_char..start_char + del_chars);
        }
        if !new_text.is_empty() {
            self.rope.insert(start_char, new_text);
        }

        // 4. Calculate new end position and update the Tree
        let new_end_position = char_to_point(&self.rope, start_char + new_text.chars().count());

        if let Some(tree) = &mut self.tree {
            tree.edit(&InputEdit {
                start_byte,
                old_end_byte,
                new_end_byte,
                start_position,
                old_end_position,
                new_end_position,
            });
        }
        
        self.modified = true;
        self.version += 1;
    }

    pub fn line_count(&self) -> usize {
        self.rope.len_lines()
    }

    pub fn len_chars(&self) -> usize {
        self.rope.len_chars()
    }
}

/// Helper to convert character offsets to Tree-sitter Points (row, byte_col)
fn char_to_point(rope: &Rope, char_idx: usize) -> Point {
    let char_idx = char_idx.min(rope.len_chars());
    let line = rope.char_to_line(char_idx);
    let line_start_char = rope.line_to_char(line);
    
    // IMPORTANT: Point column must be BYTES from line start
    let mut byte_col = 0;
    if line < rope.len_lines() {
        let line_slice = rope.line(line);
        let chars_to_count = char_idx - line_start_char;
        for (i, ch) in line_slice.chars().enumerate() {
            if i >= chars_to_count { break; }
            byte_col += ch.len_utf8();
        }
    }
    
    Point::new(line, byte_col)
}

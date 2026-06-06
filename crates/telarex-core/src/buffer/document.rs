use ropey::Rope;
use std::path::Path;
use crate::buffer::history::History;
// This was written by User_17838!  

pub struct Document {
    pub rope: Rope,
    path: Option<std::path::PathBuf>,
    pub modified: bool,
    history: History,
}


impl Document {
    pub fn new() -> Self {
        Self {
            rope: Rope::new(),
            path: None,
            modified: false,
            history: History::new(),
        }
    }

    pub fn load(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let path_ref = path.as_ref();
        if !path_ref.exists() {
            return Ok(Self {
                rope: Rope::new(),
                path: Some(path_ref.to_path_buf()),
                modified: false,
                history: History::new(),
            });
        }
        let text = std::fs::read_to_string(path_ref)?;
        Ok(Self {
            rope: Rope::from_str(&text),
            path: Some(path_ref.to_path_buf()),
            modified: false,
            history: History::new(),
        })
    }

    pub fn save(&mut self) -> std::io::Result<()> {
        if let Some(path) = &self.path {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(path, self.rope.to_string())?;
            self.modified = false;
        }
        Ok(())
    }

    pub fn insert_char(&mut self, pos: usize, ch: char) {
        self.history.push(self.rope.clone());
        self.rope.insert_char(pos, ch);
        self.modified = true;
    }

    pub fn insert(&mut self, pos: usize, text: &str) {
        self.history.push(self.rope.clone());
        self.rope.insert(pos, text);
        self.modified = true;
    }

    pub fn delete_char(&mut self, pos: usize) {
        if pos < self.rope.len_chars() {
            self.history.push(self.rope.clone());
            self.rope.remove(pos..pos + 1);
            self.modified = true;
        }
    }

    pub fn delete_range(&mut self, range: std::ops::Range<usize>) {
        if range.start < self.rope.len_chars() {
            self.history.push(self.rope.clone());
            let end = range.end.min(self.rope.len_chars());
            self.rope.remove(range.start..end);
            self.modified = true;
        }
    }

    pub fn undo(&mut self) {
        if let Some(prev) = self.history.undo(self.rope.clone()) {
            self.rope = prev;
            self.modified = true;
        }
    }

    pub fn redo(&mut self) {
        if let Some(next) = self.history.redo(self.rope.clone()) {
            self.rope = next;
            self.modified = true;
        }
    }

    pub fn len_chars(&self) -> usize {
        self.rope.len_chars()
    }

    pub fn line_count(&self) -> usize {
        self.rope.len_lines()
    }

    pub fn line(&self, index: usize) -> Option<String> {
        self.rope.get_line(index).map(|cow| cow.to_string())
    }

    pub fn is_modified(&self) -> bool {
        self.modified
    }

    pub fn path(&self) -> Option<&std::path::Path> {
        self.path.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_insert() {
        let mut doc = Document::new();
        doc.insert_char(0, 'a');
        doc.insert_char(1, 'b');
        assert_eq!(doc.rope.to_string(), "ab");
        assert!(doc.is_modified());
    }

    #[test]
    fn test_document_delete() {
        let mut doc = Document::new();
        doc.insert_char(0, 'a');
        doc.insert_char(1, 'b');
        doc.delete_char(1);
        assert_eq!(doc.rope.to_string(), "a");
    }

    #[test]
    fn test_document_insert_at_bounds() {
        let mut doc = Document::new();
        doc.insert(0, "hello");
        doc.insert(5, " world");
        assert_eq!(doc.rope.to_string(), "hello world");
    }

    #[test]
    fn test_document_delete_range() {
        let mut doc = Document::new();
        doc.insert(0, "hello world");
        doc.delete_range(5..11);
        assert_eq!(doc.rope.to_string(), "hello");
    }

    #[test]
    fn test_document_undo_redo_consistency() {
        let mut doc = Document::new();
        doc.insert(0, "abc");
        doc.insert(3, "def");
        assert_eq!(doc.rope.to_string(), "abcdef");
        doc.undo();
        assert_eq!(doc.rope.to_string(), "abc");
        doc.undo();
        assert_eq!(doc.rope.to_string(), "");
        doc.redo();
        assert_eq!(doc.rope.to_string(), "abc");
        doc.redo();
        assert_eq!(doc.rope.to_string(), "abcdef");
    }

    #[test]
    fn test_document_delete_at_end() {
        let mut doc = Document::new();
        doc.insert(0, "ab");
        doc.delete_char(1);
        assert_eq!(doc.rope.to_string(), "a");
        doc.delete_char(0);
        assert_eq!(doc.rope.to_string(), "");
    }

    #[test]
    fn test_document_line_count() {
        let mut doc = Document::new();
        assert_eq!(doc.line_count(), 1);
        doc.insert(0, "line1\nline2\nline3");
        assert_eq!(doc.line_count(), 3);
    }

    #[test]
    fn test_document_load_save_roundtrip() {
        let dir = std::env::temp_dir().join("telarex_test_doc");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("test_roundtrip.txt");
        std::fs::write(&path, "test content").unwrap();
        let mut doc = Document::load(&path).unwrap();
        assert_eq!(doc.rope.to_string(), "test content");
        assert!(!doc.is_modified());
        doc.insert(12, " extended");
        doc.save().unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert_eq!(content, "test content extended");
        let _ = std::fs::remove_dir_all(&dir);
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

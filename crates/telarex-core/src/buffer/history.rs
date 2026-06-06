use ropey::Rope;

pub struct History {
    undo_stack: Vec<Rope>,
    redo_stack: Vec<Rope>,
    max_depth: usize,
}

impl History {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_depth: 100,
        }
    }

    pub fn push(&mut self, state: Rope) {
        // If the new state is same as the last one, don't push
        if let Some(last) = self.undo_stack.last() {
            if last.to_string() == state.to_string() {
                return;
            }
        }
        
        self.undo_stack.push(state);
        if self.undo_stack.len() > self.max_depth {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    pub fn undo(&mut self, current: Rope) -> Option<Rope> {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(current);
            Some(prev)
        } else {
            None
        }
    }

    pub fn redo(&mut self, current: Rope) -> Option<Rope> {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(current);
            Some(next)
        } else {
            None
        }
    }
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rope(s: &str) -> Rope {
        Rope::from_str(s)
    }

    #[test]
    fn test_history_push_and_undo() {
        let mut h = History::new();
        h.push(rope("a"));
        h.push(rope("ab"));
        h.push(rope("abc"));
        assert_eq!(h.undo(rope("abcd")).unwrap().to_string(), "abc");
        assert_eq!(h.undo(rope("abcd")).unwrap().to_string(), "ab");
    }

    #[test]
    fn test_history_redo_after_undo() {
        let mut h = History::new();
        h.push(rope("a"));
        h.push(rope("ab"));
        let _ = h.undo(rope("abc"));
        assert_eq!(h.redo(rope("ab")).unwrap().to_string(), "abc");
    }

    #[test]
    fn test_history_clear_redo_on_new_push() {
        let mut h = History::new();
        h.push(rope("a"));
        h.push(rope("ab"));
        let _ = h.undo(rope("abc"));
        h.push(rope("abd"));
        assert!(h.redo(rope("abd")).is_none());
    }

    #[test]
    fn test_history_undo_past_empty() {
        let mut h = History::new();
        assert!(h.undo(rope("a")).is_none());
    }

    #[test]
    fn test_history_max_depth() {
        let mut h = History::new();
        h.max_depth = 3;
        h.push(rope("1"));
        h.push(rope("2"));
        h.push(rope("3"));
        h.push(rope("4"));
        assert_eq!(h.undo_stack.len(), 3);
        assert_eq!(h.undo_stack[0].to_string(), "2");
    }

    #[test]
    fn test_history_dedup() {
        let mut h = History::new();
        h.push(rope("same"));
        h.push(rope("same"));
        assert_eq!(h.undo_stack.len(), 1);
    }
}

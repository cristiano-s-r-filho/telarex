use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Workspace {
    pub id: Uuid,
    pub root: PathBuf,
    pub open_files: Vec<PathBuf>,
    pub active_file_index: Option<usize>,
    pub is_shared: bool,
}

impl Workspace {
    pub fn new(root: PathBuf) -> Self {
        Self {
            id: Uuid::new_v4(),
            root,
            open_files: Vec::new(),
            active_file_index: None,
            is_shared: false,
        }
    }

    pub fn share(&mut self) {
        self.is_shared = true;
    }

    pub fn add_file(&mut self, path: PathBuf) {
        if !self.open_files.contains(&path) {
            self.open_files.push(path);
        }
        self.active_file_index = Some(self.open_files.len() - 1);
    }

    pub fn next_file(&mut self) {
        if self.open_files.is_empty() {
            return;
        }
        let current = self.active_file_index.unwrap_or(0);
        self.active_file_index = Some((current + 1) % self.open_files.len());
    }

    pub fn prev_file(&mut self) {
        if self.open_files.is_empty() {
            return;
        }
        let current = self.active_file_index.unwrap_or(0);
        self.active_file_index = Some(if current == 0 {
            self.open_files.len() - 1
        } else {
            current - 1
        });
    }

    pub fn active_file(&self) -> Option<&PathBuf> {
        self.active_file_index.and_then(|i| self.open_files.get(i))
    }
}

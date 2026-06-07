use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// A named macro consisting of a sequence of recorded key steps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Macro {
    pub name: String,
    pub steps: Vec<MacroStep>,
}

/// A single step in a recorded macro — a key press with optional modifiers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MacroStep {
    Key {
        code: String,
        modifiers: Vec<String>,
    },
}

/// Persistent store of named macros, backed by a JSON file.
pub struct MacroStore {
    pub macros: HashMap<String, Macro>,
    path: PathBuf,
}

impl MacroStore {
    /// Create the store, loading macros from disk if they exist.
    pub fn new() -> Self {
        let mut path = directories::ProjectDirs::from("com", "telarex", "trex")
            .map(|d| d.config_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        
        path.push("macros.json");
        
        let mut store = Self {
            macros: HashMap::new(),
            path,
        };
        let _ = store.load();
        store
    }

    /// Load macros from the JSON file on disk.
    pub fn load(&mut self) -> anyhow::Result<()> {
        if self.path.exists() {
            let data = std::fs::read_to_string(&self.path)?;
            self.macros = serde_json::from_str(&data)?;
        }
        Ok(())
    }

    /// Persist all macros to the JSON file on disk.
    pub fn save(&self) -> anyhow::Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(&self.macros)?;
        std::fs::write(&self.path, data)?;
        Ok(())
    }

    /// Add or overwrite a macro by name and persist.
    pub fn add(&mut self, m: Macro) {
        self.macros.insert(m.name.clone(), m);
        let _ = self.save();
    }

    /// Remove a macro by name and persist.
    pub fn remove(&mut self, name: &str) {
        self.macros.remove(name);
        let _ = self.save();
    }
}

impl Default for MacroStore {
    fn default() -> Self {
        Self::new()
    }
}

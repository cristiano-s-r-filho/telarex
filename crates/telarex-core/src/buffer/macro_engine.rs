use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Macro {
    pub name: String,
    pub steps: Vec<MacroStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MacroStep {
    Key {
        code: String,
        modifiers: Vec<String>,
    },
}

pub struct MacroStore {
    pub macros: HashMap<String, Macro>,
    path: PathBuf,
}

impl MacroStore {
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

    pub fn load(&mut self) -> anyhow::Result<()> {
        if self.path.exists() {
            let data = std::fs::read_to_string(&self.path)?;
            self.macros = serde_json::from_str(&data)?;
        }
        Ok(())
    }

    pub fn save(&self) -> anyhow::Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(&self.macros)?;
        std::fs::write(&self.path, data)?;
        Ok(())
    }

    pub fn add(&mut self, m: Macro) {
        self.macros.insert(m.name.clone(), m);
        let _ = self.save();
    }

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

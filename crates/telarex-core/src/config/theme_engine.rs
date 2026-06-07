use crate::syntax::StyleSheet;
use std::path::Path;
use anyhow::Result;
use std::collections::HashMap;

/// Manages loaded themes and provides access to the current theme's stylesheet.
pub struct ThemeEngine {
    pub themes: HashMap<String, StyleSheet>,
    pub current_theme: String,
}

impl ThemeEngine {
    /// Create a theme engine with the built-in default theme (Tokyo Night).
    pub fn new() -> Self {
        let mut themes = HashMap::new();
        let default = StyleSheet::default_dark();
        themes.insert(default.name.clone(), default);
        
        Self {
            themes,
            current_theme: "Tokyo Night".to_string(),
        }
    }

    /// Load all `.toml` theme files from the given directory.
    pub fn load_themes(&mut self, directory: impl AsRef<Path>) -> Result<()> {
        let dir = directory.as_ref();
        if !dir.exists() {
            std::fs::create_dir_all(dir)?;
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "toml") {
                let content = std::fs::read_to_string(&path)?;
                if let Ok(ss) = toml::from_str::<StyleSheet>(&content) {
                    self.themes.insert(ss.name.clone(), ss);
                }
            }
        }
        Ok(())
    }

    /// Return the stylesheet for the currently active theme.
    pub fn get_current(&self) -> &StyleSheet {
        self.themes.get(&self.current_theme).unwrap_or_else(|| {
            self.themes.values().next().expect("At least one theme must exist")
        })
    }

    /// Switch to a named theme; returns an error if it is not loaded.
    pub fn set_theme(&mut self, name: &str) -> Result<(), String> {
        if self.themes.contains_key(name) {
            self.current_theme = name.to_string();
            Ok(())
        } else {
            Err(format!("Theme '{}' not found", name))
        }
    }

    /// Return all loaded theme names, sorted alphabetically.
    pub fn list_themes(&self) -> Vec<String> {
        let mut list: Vec<String> = self.themes.keys().cloned().collect();
        list.sort();
        list
    }
}

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

pub const CURRENT_CONFIG_VERSION: u8 = 1;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TelaRexConfig {
    pub version: u8,
    pub editor: EditorConfig,
    pub profile: UserProfile,
    pub network: NetworkConfig,
    pub recent_projects: Vec<String>,
    pub keymaps: KeymapConfig,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct NetworkConfig {
    pub bootstrap_node: String,
    pub listen_addr: String,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct KeymapConfig {
    pub global: HashMap<String, String>,
    pub editor_normal: HashMap<String, String>,
    pub editor_insert: HashMap<String, String>,
    pub explorer: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserProfile {
    pub username: String,
    pub identity_seed: String,
    pub display_name: String,
    pub bio: String,
}

impl Default for UserProfile {
    fn default() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let seed: [u8; 32] = rng.gen();
        let username = format!("User_{}", rng.gen::<u16>());
        Self {
            display_name: username.clone(),
            username,
            identity_seed: hex::encode(seed),
            bio: String::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EditorConfig {
    pub tab_size: usize,
    pub theme: String,
    pub vim_mode: bool,
    pub line_numbers: bool,
    pub auto_save: bool,
    pub wrap_text: bool,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            tab_size: 4,
            theme: "Tokyo Night".to_string(),
            vim_mode: false,
            line_numbers: true,
            auto_save: false,
            wrap_text: false,
        }
    }
}

impl Default for TelaRexConfig {
    fn default() -> Self {
        Self {
            version: CURRENT_CONFIG_VERSION,
            editor: EditorConfig::default(),
            profile: UserProfile::default(),
            network: NetworkConfig::default(),
            recent_projects: Vec::new(),
            keymaps: KeymapConfig::default_mappings(),
        }
    }
}

impl KeymapConfig {
    pub fn default_mappings() -> Self {
        let mut global = HashMap::new();
        global.insert("ctrl-q".to_string(), "Quit".to_string());
        global.insert("ctrl-p".to_string(), "EnterCommandMode".to_string());
        global.insert("ctrl-f".to_string(), "EnterSearchMode".to_string());
        global.insert("ctrl-b".to_string(), "ToggleExplorer".to_string());
        global.insert("ctrl-e".to_string(), "SwitchFocus".to_string());
        global.insert("ctrl-s".to_string(), "Save".to_string());
        global.insert("ctrl-g".to_string(), "GitStatus".to_string());

        Self {
            global,
            ..Default::default()
        }
    }
}

impl TelaRexConfig {
    pub fn add_recent_project(&mut self, path: String) {
        // De-duplicate: Remove any existing entry for this path
        self.recent_projects.retain(|p| p != &path);
        // Insert at the front
        self.recent_projects.insert(0, path);
        // Cap the list size
        self.recent_projects.truncate(10);
    }

    pub fn save(&self, session: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
        super::save(self, session)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = TelaRexConfig::default();
        assert_eq!(config.version, CURRENT_CONFIG_VERSION);
        assert_eq!(config.editor.tab_size, 4);
        assert!(config.editor.line_numbers);
        assert!(!config.editor.vim_mode);
        assert!(!config.editor.auto_save);
        assert!(!config.editor.wrap_text);
    }

    #[test]
    fn test_add_recent_project_dedup() {
        let mut config = TelaRexConfig::default();
        config.add_recent_project("/a".to_string());
        config.add_recent_project("/b".to_string());
        config.add_recent_project("/a".to_string());
        assert_eq!(config.recent_projects.len(), 2);
        assert_eq!(config.recent_projects[0], "/a");
    }

    #[test]
    fn test_add_recent_project_cap() {
        let mut config = TelaRexConfig::default();
        for i in 0..15 {
            config.add_recent_project(format!("/project_{}", i));
        }
        assert_eq!(config.recent_projects.len(), 10);
        assert_eq!(config.recent_projects[0], "/project_14");
    }

    #[test]
    fn test_serialization_roundtrip() {
        let config = TelaRexConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: TelaRexConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.version, config.version);
        assert_eq!(deserialized.editor.tab_size, config.editor.tab_size);
        assert_eq!(deserialized.recent_projects.len(), config.recent_projects.len());
        assert_eq!(deserialized.keymaps.global.len(), config.keymaps.global.len());
    }

    #[test]
    fn test_user_profile_generates_identity() {
        let profile = UserProfile::default();
        assert!(!profile.username.is_empty());
        assert!(!profile.identity_seed.is_empty());
        assert_eq!(profile.identity_seed.len(), 64); // 32 bytes = 64 hex chars
    }

    #[test]
    fn test_keymap_default_mappings_exist() {
        let km = KeymapConfig::default_mappings();
        assert!(km.global.contains_key("ctrl-q"));
        assert!(km.global.contains_key("ctrl-p"));
        assert!(km.global.contains_key("ctrl-f"));
        assert!(km.global.contains_key("ctrl-b"));
        assert!(km.global.contains_key("ctrl-e"));
    }

    #[test]
    fn test_network_config_defaults() {
        let nc = NetworkConfig::default();
        assert_eq!(nc.bootstrap_node, "");
        assert_eq!(nc.listen_addr, "");
    }
}

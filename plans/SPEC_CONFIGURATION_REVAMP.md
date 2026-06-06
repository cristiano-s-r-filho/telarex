# Spec: Configuration System Revamp

## 1. Current Problems

1. **`confy` dependency** — auto-loads/saves config on every access, mixes config path logic with config types
2. **Flat schema** — `TelaRexConfig` has all fields at one level, no grouping
3. **Theme as HashMap** — `theme: HashMap<String, String>` is untyped and fragile
4. **No config version migration** — `CURRENT_CONFIG_VERSION = 1` exists but no migration logic
5. **No validation** — invalid values silently use defaults
6. **No XDG compliance** — `directories` + `dirs` both used, conflicting

## 2. New Schema

### 2.1 File Location

OS data dir (`~/.local/share/telarex/` on Linux, `~/AppData/Roaming/telarex/` on Windows):
```
~/.config/telarex/config.toml       # user configuration
~/.local/share/telarex/themes/       # custom themes
~/.local/share/telarex/identity/     # identity keys (encrypted)
~/.local/share/telarex/data.db       # SQLite database
```

### 2.2 Config Structure

```toml
[meta]
version = 2  # for migration support

[editor]
tab_size = 4
line_numbers = true
wrap_text = false
auto_save = false
vim_mode = false
font_family = "JetBrains Mono"
font_size = 13

[appearance]
theme = "tokyo-night"
show_status_bar = true
show_tab_bar = true
compact_mode = false

[network]
bootstrap_nodes = [
  "/ip4/.../tcp/.../p2p/...",
]
listen_addr = "/ip4/0.0.0.0/tcp/0"
enable_mdns = true
enable_dht = false  # only needed for public internet

[keys.global]
"ctrl-q" = "Quit"
"ctrl-p" = "CommandMode"
"ctrl-s" = "Save"
"ctrl-f" = "SearchMode"
"ctrl-b" = "ToggleExplorer"
"ctrl-e" = "SwitchFocus"

[keys.editor.normal]
"j" = "MoveDown"
"k" = "MoveUp"
"h" = "MoveLeft"
"l" = "MoveRight"
"w" = "MoveWordForward"
"b" = "MoveWordBackward"
"e" = "MoveWordEnd"

[keys.editor.insert]
"esc" = "EnterNormalMode"

[profile]
username = ""
# identity_seed auto-generated if empty
```

### 2.3 Rust Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelaRexConfig {
    pub meta: ConfigMeta,
    pub editor: EditorConfig,
    pub appearance: AppearanceConfig,
    pub network: NetworkConfig,
    pub keys: KeymapCollection,
    pub profile: UserProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMeta {
    pub version: u8,  // default: 2
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorConfig {
    pub tab_size: u8,        // 2-8, default 4
    pub line_numbers: bool,   // default true
    pub wrap_text: bool,      // default false
    pub auto_save: bool,      // default false
    pub vim_mode: bool,       // default false
    pub font_family: String,  // default "JetBrains Mono"
    pub font_size: u8,        // 8-32, default 13
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceConfig {
    pub theme: String,          // default "tokyo-night"
    pub show_status_bar: bool,  // default true
    pub show_tab_bar: bool,     // default true
    pub compact_mode: bool,     // default false
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub bootstrap_nodes: Vec<String>,  // default empty
    pub listen_addr: String,           // default "/ip4/0.0.0.0/tcp/0"
    pub enable_mdns: bool,             // default true
    pub enable_dht: bool,              // default false
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeymapCollection {
    pub global: HashMap<String, String>,
    pub editor: KeymapContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeymapContext {
    pub normal: HashMap<String, String>,
    pub insert: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub username: String,
    // identity_seed is NOT stored in config — kept in encrypted identity file
}
```

### 2.4 Validation

Validation is applied on load, not at edit time:

```rust
impl TelaRexConfig {
    pub fn validate(&mut self) {
        self.editor.tab_size = self.editor.tab_size.clamp(2, 8);
        self.editor.font_size = self.editor.font_size.clamp(8, 32);
        if self.appearance.theme.is_empty() {
            self.appearance.theme = "tokyo-night".to_string();
        }
        // Ensure all keymap entries have valid actions
        self.keys.global.retain(|_, v| {
            !v.is_empty() && UIAction::from_str(v).is_ok()
        });
    }
}
```

## 3. Config Loading/ Saving (Replace confy)

Replace `confy` with manual file operations:

```rust
pub fn load_config() -> Result<TelaRexConfig, ConfigError> {
    let path = get_config_path();
    if !path.exists() {
        let config = TelaRexConfig::default();
        save_config(&config)?;
        return Ok(config);
    }
    let content = std::fs::read_to_string(&path)?;
    let mut config: TelaRexConfig = toml::from_str(&content)?;
    migrate_config(&mut config);  // handle version upgrades
    config.validate();
    Ok(config)
}

pub fn save_config(config: &TelaRexConfig) -> Result<(), ConfigError> {
    let path = get_config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = toml::to_string_pretty(config)?;
    std::fs::write(&path, content)?;
    Ok(())
}

fn get_config_path() -> PathBuf {
    let dir = directories::ProjectDirs::from("com", "telarex", "TelaRex")
        .expect("valid project dirs");
    dir.config_dir().join("config.toml")
}
```

## 4. Config Migration

```rust
fn migrate_config(config: &mut TelaRexConfig) {
    match config.meta.version {
        0 | 1 => {
            // Migrate from flat config to grouped config
            config.meta.version = 2;
            // Migration logic here
        }
        _ => {}  // current version, no migration needed
    }
}
```

## 5. UI Integration

The config modal (`config_modal.rs`) already has UI for editing settings. Update it to:
- Group settings by section (Editor, Appearance, Network, Keymaps)
- Add theme preview (show colors)
- Add font family selection (list available monospace fonts)
- Add validation feedback (show error on invalid values)

## 6. Implementation Plan

- [ ] Create new schema types with grouped config
- [ ] Implement manual config load/save (replace confy)
- [ ] Remove confy dependency from workspace
- [ ] Implement config migration (v1 → v2)
- [ ] Implement config validation
- [ ] Update config modal UI to match new schema
- [ ] Write unit tests: save/load cycle, migration, validation
- [ ] Update all config call sites across codebase

pub mod schema;
pub mod style_registry;
pub mod theme_engine;
pub use schema::{EditorConfig, TelaRexConfig};
pub use style_registry::{StyleRegistry, StyleToken};
pub use theme_engine::ThemeEngine;

pub const APP_NAME: &str = "telarex";

fn config_path(session: Option<&str>) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let data_dir = directories::ProjectDirs::from("", "", APP_NAME)
        .ok_or("could not determine config directory")?
        .config_dir()
        .to_path_buf();
    std::fs::create_dir_all(&data_dir)?;
    let name = if let Some(s) = session {
        format!("{}-{}.toml", APP_NAME, s)
    } else {
        format!("{}.toml", APP_NAME)
    };
    Ok(data_dir.join(name))
}

/// Load configuration, creating default if missing
pub fn load(session: Option<&str>) -> Result<TelaRexConfig, Box<dyn std::error::Error>> {
    let path = config_path(session)?;
    if !path.exists() {
        let default = TelaRexConfig::default();
        let toml_str = toml::to_string_pretty(&default)?;
        std::fs::write(&path, toml_str)?;
        return Ok(default);
    }
    let content = std::fs::read_to_string(&path)?;
    let config: TelaRexConfig = toml::from_str(&content)?;
    Ok(config)
}

/// Save configuration to disk
pub fn save(config: &TelaRexConfig, session: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let path = config_path(session)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let toml_str = toml::to_string_pretty(config)?;
    std::fs::write(&path, toml_str)?;
    Ok(())
}
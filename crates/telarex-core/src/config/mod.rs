pub mod schema;
pub mod style_registry;
pub mod theme_engine;
pub use schema::{EditorConfig, TelaRexConfig};
pub use style_registry::{StyleRegistry, StyleToken};
pub use theme_engine::ThemeEngine;

pub const APP_NAME: &str = "telarex";

/// Load configuration, creating default if missing
pub fn load(session: Option<&str>) -> Result<TelaRexConfig, Box<dyn std::error::Error>> {
    let name = if let Some(s) = session {
        format!("{}-{}", APP_NAME, s)
    } else {
        APP_NAME.to_string()
    };
    confy::load(&name, None).map_err(Into::into)
}

/// Save configuration to disk
pub fn save(config: &TelaRexConfig, session: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let name = if let Some(s) = session {
        format!("{}-{}", APP_NAME, s)
    } else {
        APP_NAME.to_string()
    };
    confy::store(&name, None, config).map_err(Into::into)
}
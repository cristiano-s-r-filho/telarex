use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleToken {
    pub fg: Option<String>,
    pub bg: Option<String>,
    pub bold: bool,
    pub italic: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleRegistry {
    pub syntax: HashMap<String, StyleToken>,
    pub ui: HashMap<String, StyleToken>,
}

impl Default for StyleRegistry {
    fn default() -> Self {
        let mut ui = HashMap::new();
        ui.insert("border.active".to_string(), StyleToken { fg: Some("#98C379".into()), bg: None, bold: true, italic: false });
        ui.insert("border.inactive".to_string(), StyleToken { fg: Some("#5C6370".into()), bg: None, bold: false, italic: false });
        ui.insert("selection.bg".to_string(), StyleToken { fg: None, bg: Some("#3E4451".into()), bold: false, italic: false });
        ui.insert("gutter.fg".to_string(), StyleToken { fg: Some("#4B5263".into()), bg: None, bold: false, italic: false });
        
        Self { syntax: HashMap::new(), ui }
    }
}

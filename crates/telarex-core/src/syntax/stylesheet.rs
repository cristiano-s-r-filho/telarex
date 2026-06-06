use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleSheet {
    pub name: String,
    pub metadata: ThemeMetadata,
    pub palette: HashMap<String, String>,
    pub ui: UIStyles,
    pub syntax: HashMap<String, StyleToken>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeMetadata {
    pub author: String,
    pub variant: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ColorRef {
    Simple(String),
    WithAlpha { color: String, alpha: f32 },
}

impl ColorRef {
    pub fn color(&self) -> &str {
        match self {
            ColorRef::Simple(s) => s,
            ColorRef::WithAlpha { color, .. } => color,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIStyles {
    pub bg: String,
    pub fg: String,
    #[serde(default)]
    pub bg_alt: Option<String>,
    #[serde(default)]
    pub surface: Option<String>,
    #[serde(default)]
    pub surface_alt: Option<String>,
    #[serde(default)]
    pub fg_muted: Option<String>,
    #[serde(default)]
    pub fg_dim: Option<String>,
    pub border_active: String,
    pub border_inactive: String,
    #[serde(default = "default_selection_bg")]
    pub selection_bg: ColorRef,
    #[serde(default)]
    pub selection_fg: Option<String>,
    pub gutter_bg: String,
    pub gutter_fg: String,
    pub status_bar_bg: String,
    pub status_bar_fg: String,
    #[serde(default)]
    pub status_bar_mode_normal: Option<String>,
    #[serde(default)]
    pub status_bar_mode_insert: Option<String>,
    #[serde(default)]
    pub status_bar_mode_visual: Option<String>,
    pub error: String,
    pub warning: String,
    #[serde(default)]
    pub success: Option<String>,
    pub info: String,
    pub hint: String,
    #[serde(default)]
    pub cursor: Option<String>,
    #[serde(default)]
    pub tab_active_underline: Option<String>,
    #[serde(default)]
    pub tab_inactive_fg: Option<String>,
    pub git_added: String,
    pub git_modified: String,
    pub git_removed: String,
}

fn default_selection_bg() -> ColorRef {
    ColorRef::Simple("#3e4452".to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleToken {
    pub color: String,
    #[serde(default)]
    pub bold: bool,
    #[serde(default)]
    pub italic: bool,
}

impl StyleSheet {
    pub fn resolve_color(&self, name: &str) -> String {
        self.palette.get(name).cloned().unwrap_or_else(|| name.to_string())
    }

    pub fn default_dark() -> Self {
        let mut palette = HashMap::new();
        palette.insert("bg".to_string(), "#1a1b26".to_string());
        palette.insert("bg_alt".to_string(), "#16161e".to_string());
        palette.insert("surface".to_string(), "#24283b".to_string());
        palette.insert("surface_alt".to_string(), "#1f2335".to_string());
        palette.insert("fg".to_string(), "#a9b1d6".to_string());
        palette.insert("fg_muted".to_string(), "#565f89".to_string());
        palette.insert("fg_dim".to_string(), "#3b4261".to_string());
        palette.insert("accent".to_string(), "#7aa2f7".to_string());
        palette.insert("accent_secondary".to_string(), "#bb9af7".to_string());
        palette.insert("green".to_string(), "#9ece6a".to_string());
        palette.insert("red".to_string(), "#f7768e".to_string());
        palette.insert("orange".to_string(), "#ff9e64".to_string());
        palette.insert("yellow".to_string(), "#e0af68".to_string());
        palette.insert("cyan".to_string(), "#7dcfff".to_string());
        palette.insert("comment".to_string(), "#565f89".to_string());

        let mut syntax = HashMap::new();
        let tokens: [(&str, &str, bool, bool); 13] = [
            ("keyword", "accent_secondary", true, false),
            ("function", "accent", false, false),
            ("string", "green", false, false),
            ("comment", "comment", false, true),
            ("variable", "fg", false, false),
            ("type", "yellow", false, false),
            ("constant", "orange", false, false),
            ("operator", "cyan", false, false),
            ("attribute", "yellow", false, false),
            ("punctuation", "fg", false, false),
            ("namespace", "yellow", false, false),
            ("macro", "accent", false, false),
            ("label", "accent", false, false),
        ];

        for (name, color_name, bold, italic) in &tokens {
            let color = palette.get(*color_name).cloned().unwrap_or_else(|| color_name.to_string());
            syntax.insert(name.to_string(), StyleToken {
                color,
                bold: *bold,
                italic: *italic,
            });
        }

        Self {
            name: "Tokyo Night".to_string(),
            metadata: ThemeMetadata {
                author: "TelaRex Team".to_string(),
                variant: "dark".to_string(),
            },
            palette,
            ui: UIStyles {
                bg: "bg".to_string(),
                fg: "fg".to_string(),
                bg_alt: Some("bg_alt".to_string()),
                surface: Some("surface".to_string()),
                surface_alt: Some("surface_alt".to_string()),
                fg_muted: Some("fg_muted".to_string()),
                fg_dim: Some("fg_dim".to_string()),
                border_active: "accent".to_string(),
                border_inactive: "fg_dim".to_string(),
                selection_bg: ColorRef::WithAlpha { color: "accent".to_string(), alpha: 0.3 },
                selection_fg: Some("fg".to_string()),
                gutter_bg: "bg".to_string(),
                gutter_fg: "fg_muted".to_string(),
                status_bar_bg: "surface".to_string(),
                status_bar_fg: "fg_muted".to_string(),
                status_bar_mode_normal: Some("accent".to_string()),
                status_bar_mode_insert: Some("green".to_string()),
                status_bar_mode_visual: Some("accent_secondary".to_string()),
                error: "red".to_string(),
                warning: "yellow".to_string(),
                success: Some("green".to_string()),
                info: "accent".to_string(),
                hint: "cyan".to_string(),
                cursor: Some("accent".to_string()),
                tab_active_underline: Some("accent".to_string()),
                tab_inactive_fg: Some("fg_muted".to_string()),
                git_added: "green".to_string(),
                git_modified: "yellow".to_string(),
                git_removed: "red".to_string(),
            },
            syntax,
        }
    }
}

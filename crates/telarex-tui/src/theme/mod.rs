use ratatui::style::{Color, Style, Modifier};
use telarex_core::syntax::StyleSheet;
use telarex_core::syntax::stylesheet::StyleToken;

#[derive(Clone)]
pub struct Theme {
    pub bg: Color,
    pub fg: Color,
    pub border_active: Color,
    pub border_inactive: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,
    pub gutter_bg: Color,
    pub gutter_fg: Color,
    pub cursor_bg: Color,
    pub cursor_fg: Color,
    pub status_bar_bg: Color,
    pub status_bar_fg: Color,
    
    // Semantic Styles
    pub header: Style,
    pub subtitle: Style,
    pub mission: Style,
    pub list_selected: Style,
    pub list_inactive: Style,
    pub status_label: Style,
    pub status_value: Style,
    pub footer_hint: Style,
    
    // Syntax Tokens
    pub syntax: std::collections::HashMap<String, Style>,
}

impl Theme {
    pub fn from_stylesheet(ss: &StyleSheet) -> Self {
        let bg = parse_hex(&ss.resolve_color(&ss.ui.bg));
        let fg = parse_hex(&ss.resolve_color(&ss.ui.fg));
        let accent = parse_hex(&ss.resolve_color(&ss.ui.border_active));
        let border_inactive = parse_hex(&ss.resolve_color(&ss.ui.border_inactive));
        let selection_bg = parse_hex(&ss.resolve_color(&ss.ui.selection_bg));
        
        let mut syntax = std::collections::HashMap::new();
        for (name, token) in &ss.syntax {
            syntax.insert(name.clone(), token_to_style(token, ss));
        }

        Self {
            bg,
            fg,
            border_active: accent,
            border_inactive,
            selection_bg,
            selection_fg: bg,
            gutter_bg: parse_hex(&ss.resolve_color(&ss.ui.gutter_bg)),
            gutter_fg: parse_hex(&ss.resolve_color(&ss.ui.gutter_fg)),
            cursor_bg: accent,
            cursor_fg: bg,
            status_bar_bg: parse_hex(&ss.resolve_color(&ss.ui.status_bar_bg)),
            status_bar_fg: fg,
            
            header: Style::default().fg(accent).add_modifier(Modifier::BOLD),
            subtitle: Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
            mission: Style::default().fg(Color::Cyan),
            list_selected: Style::default().bg(selection_bg).fg(bg).add_modifier(Modifier::BOLD),
            list_inactive: Style::default().fg(Color::DarkGray),
            status_label: Style::default().fg(Color::DarkGray),
            status_value: Style::default().fg(fg),
            footer_hint: Style::default().fg(Color::DarkGray),
            
            syntax,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::from_stylesheet(&StyleSheet::default_dark())
    }
}

fn parse_hex(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return match hex.to_lowercase().as_str() {
            "black" => Color::Black,
            "red" => Color::Red,
            "green" => Color::Green,
            "yellow" => Color::Yellow,
            "blue" => Color::Blue,
            "magenta" => Color::Magenta,
            "cyan" => Color::Cyan,
            "white" => Color::White,
            "gray" | "grey" => Color::Gray,
            _ => Color::Reset,
        };
    }
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    Color::Rgb(r, g, b)
}

fn token_to_style(token: &StyleToken, ss: &StyleSheet) -> Style {
    let mut style = Style::default().fg(parse_hex(&ss.resolve_color(&token.color)));
    if token.bold { style = style.add_modifier(Modifier::BOLD); }
    if token.italic { style = style.add_modifier(Modifier::ITALIC); }
    style
}

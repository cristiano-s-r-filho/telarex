//! Theme — color palette, semantic styles, and syntax tokens derived from a stylesheet.
use ratatui::style::{Color, Style, Modifier};
use telarex_core::syntax::StyleSheet;
use telarex_core::syntax::stylesheet::StyleToken;

/// Complete theme definition — all visual colours, semantic styles, and syntax tokens.
#[derive(Clone)]
pub struct Theme {
    /// Primary background colour.
    pub bg: Color,
    /// Alternative background colour.
    pub bg_alt: Color,
    /// Primary foreground (text) colour.
    pub fg: Color,
    /// Muted foreground for secondary text.
    pub fg_muted: Color,
    /// Dimmed foreground for hints and less-important text.
    pub fg_dim: Color,
    /// Surface colour for panels and containers.
    pub surface: Color,
    /// Alternative surface colour for modal backgrounds and elevated panels.
    pub surface_alt: Color,
    /// Border colour for focused/active widgets.
    pub border_active: Color,
    /// Border colour for inactive widgets.
    pub border_inactive: Color,
    /// Background colour for list/item selection.
    pub selection_bg: Color,
    /// Foreground colour for selected text.
    pub selection_fg: Color,
    /// Background of the editor gutter.
    pub gutter_bg: Color,
    /// Foreground of the editor gutter (line numbers).
    pub gutter_fg: Color,
    /// Cursor background colour.
    pub cursor_bg: Color,
    /// Cursor foreground colour.
    pub cursor_fg: Color,
    /// Status bar background.
    pub status_bar_bg: Color,
    /// Status bar foreground.
    pub status_bar_fg: Color,
    /// Underline colour for the active tab.
    pub tab_active_underline: Color,
    /// Foreground colour for inactive tabs.
    pub tab_inactive_fg: Color,
    
    /// Primary accent colour.
    pub accent: Color,
    /// Secondary accent colour.
    pub accent_secondary: Color,
    
    /// Colour for success states.
    pub success: Color,
    /// Colour for warning states.
    pub warning: Color,
    /// Colour for error states.
    pub error: Color,
    /// Colour for informational states.
    pub info: Color,
    
    /// Colour for normal editing mode.
    pub mode_normal: Color,
    /// Colour for insert editing mode.
    pub mode_insert: Color,
    /// Colour for visual editing mode.
    pub mode_visual: Color,
    
    /// Style for header text (banners, titles).
    pub header: Style,
    /// Style for subtitle text.
    pub subtitle: Style,
    /// Style for mission/description text.
    pub mission: Style,
    /// Style for selected list items.
    pub list_selected: Style,
    /// Style for inactive list items.
    pub list_inactive: Style,
    /// Style for status bar labels.
    pub status_label: Style,
    /// Style for status bar values.
    pub status_value: Style,
    /// Style for footer hint text.
    pub footer_hint: Style,
    /// Style for active tab text.
    pub tab_active: Style,
    /// Style for inactive tab text.
    pub tab_inactive: Style,
    
    /// Syntax-highlighting token styles (token name -> ratatui Style).
    pub syntax: std::collections::HashMap<String, Style>,
}

impl Theme {
    /// Constructs a [`Theme`] from a [`StyleSheet`], resolving all colours and styles.
    pub fn from_stylesheet(ss: &StyleSheet) -> Self {
        let bg = parse_hex(&ss.resolve_color(&ss.ui.bg));
        let fg = parse_hex(&ss.resolve_color(&ss.ui.fg));
        let accent = parse_hex(&ss.resolve_color(&ss.ui.border_active));
        let border_inactive = parse_hex(&ss.resolve_color(&ss.ui.border_inactive));
        let selection_bg = parse_hex(&ss.resolve_color(ss.ui.selection_bg.color()));
        let bg_alt = ss.ui.bg_alt.as_ref().map(|s| parse_hex(&ss.resolve_color(s))).unwrap_or(accent);
        let surface = ss.ui.surface.as_ref().map(|s| parse_hex(&ss.resolve_color(s))).unwrap_or(darken(bg));
        let surface_alt = ss.ui.surface_alt.as_ref().map(|s| parse_hex(&ss.resolve_color(s))).unwrap_or(darken(surface));
        let fg_muted = ss.ui.fg_muted.as_ref().map(|s| parse_hex(&ss.resolve_color(s))).unwrap_or(Color::DarkGray);
        let fg_dim = ss.ui.fg_dim.as_ref().map(|s| parse_hex(&ss.resolve_color(s))).unwrap_or(Color::Gray);
        
        let mut syntax = std::collections::HashMap::new();
        for (name, token) in &ss.syntax {
            syntax.insert(name.clone(), token_to_style(token, ss));
        }

        // Derive accent_secondary from palette or fall back to accent
        let accent_secondary = ss.palette.get("accent_secondary")
            .and_then(|s| Some(parse_hex(s)))
            .unwrap_or_else(|| {
                parse_hex(&ss.resolve_color(&ss.ui.info))
            });

        // Mode colors: use theme-specific or fallback
        let mode_normal = ss.ui.status_bar_mode_normal.as_ref()
            .map(|s| parse_hex(&ss.resolve_color(s)))
            .unwrap_or(accent);
        let mode_insert = ss.ui.status_bar_mode_insert.as_ref()
            .map(|s| parse_hex(&ss.resolve_color(s)))
            .unwrap_or_else(|| ss.ui.success.as_ref().map(|s| parse_hex(&ss.resolve_color(s))).unwrap_or(Color::Green));
        let mode_visual = ss.ui.status_bar_mode_visual.as_ref()
            .map(|s| parse_hex(&ss.resolve_color(s)))
            .unwrap_or(accent_secondary);

        let tab_active_underline = ss.ui.tab_active_underline.as_ref()
            .map(|s| parse_hex(&ss.resolve_color(s)))
            .unwrap_or(accent);
        let tab_inactive_fg = ss.ui.tab_inactive_fg.as_ref()
            .map(|s| parse_hex(&ss.resolve_color(s)))
            .unwrap_or(fg_muted);

        Self {
            bg,
            bg_alt,
            fg,
            fg_muted,
            fg_dim,
            surface,
            surface_alt,
            border_active: accent,
            border_inactive,
            selection_bg,
            selection_fg: bg,
            gutter_bg: parse_hex(&ss.resolve_color(&ss.ui.gutter_bg)),
            gutter_fg: parse_hex(&ss.resolve_color(&ss.ui.gutter_fg)),
            cursor_bg: accent,
            cursor_fg: bg,
            status_bar_bg: parse_hex(&ss.resolve_color(&ss.ui.status_bar_bg)),
            status_bar_fg: parse_hex(&ss.resolve_color(&ss.ui.status_bar_fg)),
            tab_active_underline,
            tab_inactive_fg,
            
            // Accents
            accent,
            accent_secondary,
            
            // Semantic Colors
            success: ss.ui.success.as_ref().map(|s| parse_hex(&ss.resolve_color(s))).unwrap_or(Color::Green),
            warning: parse_hex(&ss.resolve_color(&ss.ui.warning)),
            error: parse_hex(&ss.resolve_color(&ss.ui.error)),
            info: parse_hex(&ss.resolve_color(&ss.ui.info)),
            
            // Mode-specific colors
            mode_normal,
            mode_insert,
            mode_visual,
            
            header: Style::default().fg(accent).add_modifier(Modifier::BOLD),
            subtitle: Style::default().fg(fg_dim).add_modifier(Modifier::ITALIC),
            mission: Style::default().fg(Color::Cyan),
            list_selected: Style::default().bg(selection_bg).fg(bg).add_modifier(Modifier::BOLD),
            list_inactive: Style::default().fg(fg_muted),
            status_label: Style::default().fg(fg_muted),
            status_value: Style::default().fg(fg),
            footer_hint: Style::default().fg(fg_dim),
            tab_active: Style::default().fg(fg).add_modifier(Modifier::BOLD),
            tab_inactive: Style::default().fg(tab_inactive_fg),
            
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

fn darken(c: Color) -> Color {
    if let Color::Rgb(r, g, b) = c {
        Color::Rgb(r.saturating_sub(15), g.saturating_sub(15), b.saturating_sub(15))
    } else {
        c
    }
}

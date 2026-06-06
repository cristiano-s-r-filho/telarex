use tree_sitter_highlight::{HighlightConfiguration, HighlightEvent, Highlighter as TSHighlighter};
use std::collections::HashMap;
use ratatui::text::{Line, Span};
use ratatui::style::{Style, Modifier, Color};
use crate::syntax::stylesheet::{StyleSheet, StyleToken};
use ropey::Rope;

pub struct TreeHighlighter {
    highlighter: TSHighlighter,
    configs: HashMap<String, HighlightConfiguration>,
    scope_map: HashMap<usize, String>,
}

impl TreeHighlighter {
    pub fn new() -> Self {
        let mut configs = HashMap::new();
        
        let highlights = [
            "keyword", "function", "string", "comment", "variable", 
            "type", "constant", "operator", "attribute", "punctuation"
        ];

        if let Ok(mut config) = HighlightConfiguration::new(
            tree_sitter_rust::LANGUAGE.into(),
            "rust",
            tree_sitter_rust::HIGHLIGHTS_QUERY,
            "",
            ""
        ) {
            config.configure(&highlights);
            configs.insert("rust".to_string(), config);
        }

        if let Ok(mut config) = HighlightConfiguration::new(
            tree_sitter_json::LANGUAGE.into(),
            "json",
            tree_sitter_json::HIGHLIGHTS_QUERY,
            "",
            ""
        ) {
            config.configure(&highlights);
            configs.insert("json".to_string(), config);
        }

        Self {
            highlighter: TSHighlighter::new(),
            configs,
            scope_map: highlights.iter().enumerate().map(|(i, s)| (i, s.to_string())).collect(),
        }
    }

    /// HIGH-PERFORMANCE HIGHLIGHTING: Processes only the visible range.
    pub fn highlight_visible_range(
        &mut self,
        rope: &Rope,
        language: &str,
        stylesheet: &StyleSheet,
        start_line: usize,
        end_line: usize,
    ) -> Vec<Line<'static>> {
        let config = match self.configs.get(language) {
            Some(c) => c,
            None => return self.plain_text_lines(rope, start_line, end_line),
        };

        let start_char = rope.line_to_char(start_line);
        let end_char = rope.line_to_char(end_line.min(rope.len_lines()));
        
        // O(visible) extraction
        let source = rope.slice(start_char..end_char).to_string();
        
        let highlights_res = self.highlighter.highlight(config, source.as_bytes(), None, |_| None);
        
        let highlights = match highlights_res {
            Ok(h) => h,
            Err(_) => return self.plain_text_lines(rope, start_line, end_line),
        };

        let mut lines = Vec::new();
        let mut current_line_spans = Vec::new();
        
        let initial_fg = stylesheet.resolve_color(&stylesheet.ui.fg);
        let active_style = StyleToken { color: initial_fg, bold: false, italic: false };
        let mut style_stack = vec![active_style];

        for event in highlights {
            let event = match event {
                Ok(e) => e,
                Err(_) => break,
            };

            match event {
                HighlightEvent::Source { start, end } => {
                    let content = &source[start..end];
                    let parts: Vec<&str> = content.split(|c| c == '\n' || c == '\r').collect();
                    
                    for (i, part) in parts.iter().enumerate() {
                        if !part.is_empty() {
                            let token = style_stack.last().unwrap();
                            let mut style = Style::default().fg(parse_hex(&stylesheet.resolve_color(&token.color)));
                            if token.bold { style = style.add_modifier(Modifier::BOLD); }
                            if token.italic { style = style.add_modifier(Modifier::ITALIC); }
                            
                            let clean_part = part.chars().filter(|c| !c.is_control()).collect::<String>();
                            current_line_spans.push(Span::styled(clean_part, style));
                        }
                        
                        if i < parts.len() - 1 {
                            lines.push(Line::from(current_line_spans));
                            current_line_spans = Vec::new();
                        }
                    }
                }
                HighlightEvent::HighlightStart(s) => {
                    let scope = &self.scope_map[&s.0];
                    let token = stylesheet.syntax.get(scope).cloned().unwrap_or_else(|| style_stack.last().unwrap().clone());
                    style_stack.push(token);
                }
                HighlightEvent::HighlightEnd => {
                    if style_stack.len() > 1 { style_stack.pop(); }
                }
            }
        }

        lines.push(Line::from(current_line_spans));

        let requested_count = end_line.saturating_sub(start_line);
        while lines.len() < requested_count {
            lines.push(Line::raw(""));
        }
        lines.truncate(requested_count);
        lines
    }

    fn plain_text_lines(&self, rope: &Rope, start: usize, end: usize) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        for i in start..end.min(rope.len_lines()) {
            let s = rope.line(i).to_string();
            lines.push(Line::raw(s.trim_matches(|c| c == '\r' || c == '\n').to_string()));
        }
        lines
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
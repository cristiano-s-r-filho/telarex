use crossterm::event::{KeyCode, KeyModifiers, KeyEvent};
use ratatui::
{
    layout::{Rect, Position, Layout, Constraint},
    style::{Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
    prelude::Stylize,
};
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use crate::tui_compat::{AppContext, Component, DrawContext, Event, EventResult};
use telarex_core::buffer::ManagedBuffer as Document;
use telarex_core::syntax::{TreeHighlighter, StyleSheet};

use crate::theme::Theme;
use crate::utils::sanitize;

#[derive(Debug, Clone)]
pub struct TextChange {
    pub pos: usize,
    pub del: usize,
    pub text: String,
}

pub struct Editor {
    document: Option<Arc<Mutex<Document>>>,
    pub focused: RefCell<bool>,
    pub cursor_offset: usize,
    pub scroll_line: usize,
    pub scroll_col: usize,
    last_area: RefCell<Rect>,
    stylesheet: StyleSheet,
    language: Option<String>,
    pub gutter_width: usize,
    pub theme: Theme,
    pub selection: Option<std::ops::Range<usize>>,
    pub last_change: Option<TextChange>,
    preferred_visual_col: Option<usize>,
    
    // HIGH-PERFORMANCE HIGHLIGHTER
    highlighter: RefCell<TreeHighlighter>,
    
    file_to_open: Option<std::path::PathBuf>,
}

impl std::fmt::Debug for Editor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Editor")
            .field("focused", &self.focused)
            .field("cursor_offset", &self.cursor_offset)
            .finish()
    }
}

impl Editor {
    pub fn new() -> Self {
        let stylesheet = StyleSheet::default_dark();
        let theme = Theme::from_stylesheet(&stylesheet);

        Self {
            document: None,
            focused: RefCell::new(false),
            cursor_offset: 0,
            scroll_line: 0,
            scroll_col: 0,
            last_area: RefCell::new(Rect::default()),
            stylesheet,
            language: None,
            gutter_width: 4,
            theme,
            selection: None,
            last_change: None,
            preferred_visual_col: None,
            highlighter: RefCell::new(TreeHighlighter::new()),
            file_to_open: None,
        }
    }

    pub fn apply_theme(&mut self, ss: &StyleSheet) {
        self.stylesheet = ss.clone();
        self.theme = Theme::from_stylesheet(ss);
    }

    pub fn load_document(&mut self, path: std::path::PathBuf, doc: Arc<Mutex<Document>>) {
        self.document = Some(doc);
        self.cursor_offset = 0;
        self.scroll_line = 0;
        self.scroll_col = 0;
        self.selection = None;
        self.preferred_visual_col = None;
        
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            self.language = match ext {
                "rs" => Some("rust".to_string()),
                "json" => Some("json".to_string()),
                "md" => Some("markdown".to_string()),
                "toml" => Some("toml".to_string()),
                _ => None,
            };
        } else {
            self.language = None;
        }
        
        self.update_gutter_width();
    }

    pub fn take_file_to_open(&mut self) -> Option<std::path::PathBuf> {
        self.file_to_open.take()
    }

    pub fn _document(&self) -> Option<Arc<Mutex<Document>>> {
        self.document.clone()
    }

    fn update_gutter_width(&mut self) {
        if let Some(doc_arc) = &self.document {
            let line_count = doc_arc.lock().unwrap().line_count();
            self.gutter_width = line_count.to_string().len() + 1;
        }
    }

    pub fn set_text(&mut self, text: &str) {
        if let Some(doc_arc) = &self.document {
            {
                let mut d = doc_arc.lock().unwrap();
                d.rope = ropey::Rope::from_str(text);
                let new_len = d.len_chars();
                self.cursor_offset = self.cursor_offset.min(new_len);
                if let Some(ref mut sel) = self.selection {
                    sel.start = sel.start.min(new_len);
                    sel.end = sel.end.min(new_len);
                }
                d.version += 1;
            }
            self.update_gutter_width();
            self.ensure_cursor_visible();
        }
    }

    pub fn save(&mut self) -> std::io::Result<()> {
        if let Some(doc_arc) = &self.document {
            let doc = doc_arc.lock().unwrap();
            if let Some(path) = &doc.path {
                let mut file = std::fs::File::create(path)?;
                doc.rope.write_to(&mut file)?;
            }
        }
        Ok(())
    }

    pub fn is_modified(&self) -> bool {
        self.document.as_ref().map_or(false, |d| d.lock().unwrap().modified)
    }

    pub fn file_path(&self) -> Option<std::path::PathBuf> {
        self.document.as_ref().and_then(|d| d.lock().unwrap().path.clone())
    }

    pub fn language(&self) -> Option<&str> {
        self.language.as_deref()
    }

    pub fn cursor_position(&self) -> (usize, usize) {
        if let Some(doc_arc) = &self.document {
            let doc = doc_arc.lock().unwrap();
            let (line, visual_col) = self.char_to_visual_internal(&doc.rope, self.cursor_offset);
            (line + 1, visual_col + 1)
        } else {
            (1, 1)
        }
    }

    fn char_to_visual_internal(&self, rope: &ropey::Rope, offset: usize) -> (usize, usize) {
        if offset == 0 { return (0, 0); }
        let char_idx = offset.min(rope.len_chars());
        let line = rope.char_to_line(char_idx);
        let line_start_char = rope.line_to_char(line);
        let char_col = char_idx - line_start_char;
        
        let mut visual_col = 0;
        if line < rope.len_lines() {
            let line_slice = rope.line(line);
            for (i, ch) in line_slice.chars().enumerate() {
                if i >= char_col { break; }
                if ch == '\r' || ch == '\n' { break; }
                if ch.is_control() { continue; }
                visual_col += unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
            }
        }
        (line, visual_col)
    }

    fn visual_to_char_internal(&self, rope: &ropey::Rope, line: usize, visual_col: usize) -> usize {
        if line >= rope.len_lines() { return rope.len_chars(); }
        
        let line_start = rope.line_to_char(line);
        let line_slice = rope.line(line);
        
        let mut current_visual_col = 0;
        let mut char_idx = 0;
        
        for ch in line_slice.chars() {
            if ch == '\n' || ch == '\r' { break; }
            let w = if ch.is_control() { 0 } else { unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0) };
            if current_visual_col + w > visual_col { break; }
            current_visual_col += w;
            char_idx += 1;
        }
        
        line_start + char_idx
    }

    fn visible_area(&self) -> Rect {
        let area = *self.last_area.borrow();
        let border_style = if *self.focused.borrow() { Style::default().fg(self.theme.border_active) } else { Style::default().fg(self.theme.border_inactive) };
        let block = Block::default().borders(Borders::ALL).border_style(border_style);
        let inner = block.inner(area);
        
        let gutter_width = self.gutter_width as u16;
        let layout = Layout::horizontal([
            Constraint::Length(gutter_width),
            Constraint::Min(0),
        ]).split(inner);

        layout[1]
    }

    fn ensure_cursor_visible(&mut self) {
        let area = self.visible_area();
        let visible_height = area.height as usize;
        let visible_width = area.width as usize;
        if visible_height == 0 || visible_width == 0 { return; }

        let (cursor_line, cursor_visual_col) = if let Some(doc_arc) = &self.document {
            let doc = doc_arc.lock().unwrap();
            self.char_to_visual_internal(&doc.rope, self.cursor_offset)
        } else {
            (0, 0)
        };

        if cursor_line >= self.scroll_line + visible_height {
            self.scroll_line = cursor_line - visible_height + 1;
        } else if cursor_line < self.scroll_line {
            self.scroll_line = cursor_line;
        }

        if cursor_visual_col >= self.scroll_col + visible_width {
            self.scroll_col = cursor_visual_col - visible_width + 1;
        } else if cursor_visual_col < self.scroll_col {
            self.scroll_col = cursor_visual_col;
        }
    }

    fn move_cursor_left(&mut self) {
        if self.cursor_offset > 0 {
            self.cursor_offset -= 1;
            self.preferred_visual_col = None;
            self.ensure_cursor_visible();
        }
    }

    fn move_cursor_right(&mut self) {
        if let Some(doc_arc) = &self.document {
            let d = doc_arc.lock().unwrap();
            if self.cursor_offset < d.len_chars() {
                let ch = d.rope.char(self.cursor_offset);
                if ch == '\r' && self.cursor_offset + 1 < d.len_chars() && d.rope.char(self.cursor_offset + 1) == '\n' {
                    self.cursor_offset += 2;
                } else {
                    self.cursor_offset += 1;
                }
            }
        }
        self.preferred_visual_col = None;
        self.ensure_cursor_visible();
    }

    fn move_cursor_up(&mut self) {
        if let Some(doc_arc) = &self.document {
            let doc = doc_arc.lock().unwrap();
            let (line, visual_col) = self.char_to_visual_internal(&doc.rope, self.cursor_offset);
            let target_col = self.preferred_visual_col.unwrap_or(visual_col);
            if line > 0 {
                self.cursor_offset = self.visual_to_char_internal(&doc.rope, line - 1, target_col);
                self.preferred_visual_col = Some(target_col);
                drop(doc);
                self.ensure_cursor_visible();
            }
        }
    }

    fn move_cursor_down(&mut self) {
        if let Some(doc_arc) = &self.document {
            let doc = doc_arc.lock().unwrap();
            let (line, visual_col) = self.char_to_visual_internal(&doc.rope, self.cursor_offset);
            let target_col = self.preferred_visual_col.unwrap_or(visual_col);
            
            let line_count = doc.line_count();
            if line + 1 < line_count {
                self.cursor_offset = self.visual_to_char_internal(&doc.rope, line + 1, target_col);
                self.preferred_visual_col = Some(target_col);
                drop(doc);
                self.ensure_cursor_visible();
            }
        }
    }

    pub fn insert_char(&mut self, ch: char) -> (usize, usize, String) {
        if let Some(doc_arc) = &self.document {
            let pos = self.cursor_offset;
            doc_arc.lock().unwrap().apply_edit(pos, 0, &ch.to_string());
            self.cursor_offset += 1;
            self.preferred_visual_col = None;
            self.update_gutter_width();
            self.ensure_cursor_visible();
            (pos, 0, ch.to_string())
        } else {
            (0, 0, String::new())
        }
    }

    pub fn backspace(&mut self) -> Option<(usize, usize, String)> {
        if self.cursor_offset > 0 {
            let mut pos = self.cursor_offset - 1;
            let mut del_len = 1;
            
            if let Some(doc_arc) = &self.document {
                {
                    let mut d = doc_arc.lock().unwrap();
                    if pos > 0 && d.rope.char(pos) == '\n' && d.rope.char(pos - 1) == '\r' {
                        pos -= 1;
                        del_len = 2;
                    }
                    d.apply_edit(pos, del_len, "");
                }
                self.cursor_offset = pos;
                self.preferred_visual_col = None;
                self.update_gutter_width();
                self.ensure_cursor_visible();
                return Some((pos, del_len, String::new()));
            }
        }
        None
    }

    pub fn delete_under_cursor(&mut self) -> Option<(usize, usize, String)> {
        if let Some(doc_arc) = &self.document {
            let pos = self.cursor_offset;
            let mut del_len = 0;
            {
                let mut d = doc_arc.lock().unwrap();
                if pos < d.len_chars() {
                    del_len = 1;
                    if d.rope.char(pos) == '\r' && pos + 1 < d.len_chars() && d.rope.char(pos + 1) == '\n' {
                        del_len = 2;
                    }
                    d.apply_edit(pos, del_len, "");
                }
            }
            if del_len > 0 {
                self.preferred_visual_col = None;
                self.update_gutter_width();
                self.ensure_cursor_visible();
                return Some((pos, del_len, String::new()));
            }
        }
        None
    }

    pub fn copy(&mut self) -> Result<(), String> {
        if let Some(range) = &self.selection {
            if let Some(doc_arc) = &self.document {
                let text = doc_arc.lock().unwrap().rope.slice(range.start..range.end).to_string();
                telarex_core::clipboard::copy(&text)?;
            }
        }
        Ok(())
    }

    pub fn paste(&mut self) -> Result<TextChange, String> {
        let text = telarex_core::clipboard::paste()?;
        let (pos, del, t) = self.insert_text(&text);
        Ok(TextChange { pos, del, text: t })
    }

    pub fn insert_text(&mut self, text: &str) -> (usize, usize, String) {
        if let Some(doc_arc) = &self.document {
            let pos = self.cursor_offset;
            doc_arc.lock().unwrap().apply_edit(pos, 0, text);
            self.cursor_offset += text.chars().count();
            self.preferred_visual_col = None;
            self.update_gutter_width();
            self.ensure_cursor_visible();
            (pos, 0, text.to_string())
        } else {
            (0, 0, String::new())
        }
    }

    pub fn handle_key_event(&mut self, key_event: &KeyEvent) -> (EventResult, Option<TextChange>) {
        let code = key_event.code;
        let mods = key_event.modifiers;

        if key_event.kind != crossterm::event::KeyEventKind::Press && key_event.kind != crossterm::event::KeyEventKind::Repeat {
            return (EventResult::Handled, None);
        }

        match code {
            KeyCode::Char(ch) => {
                let is_strictly_control = mods.contains(KeyModifiers::CONTROL) && !mods.contains(KeyModifiers::ALT);
                let is_strictly_alt = mods.contains(KeyModifiers::ALT) && !mods.contains(KeyModifiers::CONTROL);
                
                if !is_strictly_control && !is_strictly_alt {
                    let (pos, del, text) = self.insert_char(ch);
                    return (EventResult::Handled, Some(TextChange { pos, del, text }));
                }
            }
            KeyCode::Backspace => {
                let res = self.backspace().map(|(pos, del, text)| TextChange { pos, del, text });
                return (EventResult::Handled, res);
            }
            KeyCode::Enter => {
                let (pos, del, text) = self.insert_char('\n');
                return (EventResult::Handled, Some(TextChange { pos, del, text }));
            }
            KeyCode::Tab => {
                let pos = self.cursor_offset;
                if let Some(doc_arc) = &self.document {
                    doc_arc.lock().unwrap().apply_edit(pos, 0, "    ");
                    self.cursor_offset += 4;
                    self.ensure_cursor_visible();
                }
                return (EventResult::Handled, Some(TextChange { pos, del: 0, text: "    ".to_string() }));
            }
            KeyCode::Delete => {
                let res = self.delete_under_cursor().map(|(pos, del, text)| TextChange { pos, del, text });
                return (EventResult::Handled, res);
            }
            KeyCode::Left => { self.move_cursor_left(); return (EventResult::Handled, None); }
            KeyCode::Right => { self.move_cursor_right(); return (EventResult::Handled, None); }
            KeyCode::Up => { self.move_cursor_up(); return (EventResult::Handled, None); }
            KeyCode::Down => { self.move_cursor_down(); return (EventResult::Handled, None); }
            _ => {}
        }

        (EventResult::Unhandled, None)
    }

    fn handle_mouse(&mut self, mouse: &crossterm::event::MouseEvent) -> EventResult {
        if mouse.kind == crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left) {
            let area = self.visible_area();
            if area.contains(Position::new(mouse.column, mouse.row)) {
                if let Some(doc_arc) = &self.document {
                    let doc = doc_arc.lock().unwrap();
                    let visual_x = (mouse.column as usize).saturating_sub(area.x as usize) + self.scroll_col;
                    let line_idx = (mouse.row as usize).saturating_sub(area.y as usize) + self.scroll_line;
                    self.cursor_offset = self.visual_to_char_internal(&doc.rope, line_idx, visual_x);
                    self.preferred_visual_col = None;
                }
                return EventResult::Handled;
            }
        } else if mouse.kind == crossterm::event::MouseEventKind::ScrollUp {
            if self.scroll_line > 0 { self.scroll_line -= 1; }
            return EventResult::Handled;
        } else if mouse.kind == crossterm::event::MouseEventKind::ScrollDown {
            if let Some(doc_arc) = &self.document {
                let line_count = doc_arc.lock().unwrap().line_count();
                if self.scroll_line + 1 < line_count { self.scroll_line += 1; }
            }
            return EventResult::Handled;
        }
        EventResult::Unhandled
    }
}

impl Component for Editor {
    fn handle_event(&mut self, event: &Event, _ctx: &mut AppContext) -> EventResult {
        self.last_change = None;
        match event {
            Event::Key(key_event) if *self.focused.borrow() => {
                let (res, change) = self.handle_key_event(key_event);
                self.last_change = change;
                res
            }
            Event::Mouse(mouse_event) => self.handle_mouse(mouse_event),
            _ => EventResult::Unhandled,
        }
    }

    fn draw(&self, frame: &mut Frame, area: Rect, _ctx: &DrawContext) {
        *self.last_area.borrow_mut() = area;
        let main_layout = Layout::vertical([
            Constraint::Length(1), 
            Constraint::Min(0),    
        ]).split(area);

        let path_str = self.file_path().map(|p| p.display().to_string()).unwrap_or_else(|| "Untitled".to_string());
        frame.render_widget(Paragraph::new(sanitize(&path_str)).bg(self.theme.bg).fg(self.theme.gutter_fg), main_layout[0]);

        let border_style = if *self.focused.borrow() { Style::default().fg(self.theme.border_active) } else { Style::default().fg(self.theme.border_inactive) };
        let block = Block::default().borders(Borders::ALL).border_style(border_style).bg(self.theme.bg);
        let inner_area = block.inner(main_layout[1]);
        frame.render_widget(block, main_layout[1]);

        let [gutter_area, content_area] = Layout::horizontal([
            Constraint::Length(self.gutter_width as u16),
            Constraint::Min(0),
        ]).areas(inner_area);

        let doc_arc = self.document.clone();
        if doc_arc.is_none() { return; }
        let doc_binding = doc_arc.unwrap();
        let doc = doc_binding.lock().unwrap();
        
        let line_count = doc.line_count().max(1);
        let start_line = self.scroll_line;
        let visible_lines = content_area.height as usize;
        let end_line = (start_line + visible_lines).min(line_count);

        let mut gutter_spans = Vec::new();
        let cursor_line_idx = doc.rope.char_to_line(self.cursor_offset.min(doc.rope.len_chars()));
        for i in start_line..end_line {
            let style = if i == cursor_line_idx && *self.focused.borrow() { Style::default().fg(self.theme.fg).add_modifier(Modifier::BOLD) } else { Style::default().fg(self.theme.gutter_fg) };
            gutter_spans.push(Line::from(Span::styled(format!("{:>width$} ", i + 1, width = self.gutter_width - 1), style)));
        }
        frame.render_widget(Paragraph::new(gutter_spans).bg(self.theme.gutter_bg), gutter_area);

        // LIVE ZERO-COPY HIGHLIGHTING
        let mut highlighter = self.highlighter.borrow_mut();
        let lang = self.language().unwrap_or("plain");
        let highlighted_lines = if let Some(ref tree) = doc.tree {
            highlighter.highlight_visible_range(
                &doc.rope,
                tree,
                lang,
                &self.stylesheet,
                start_line,
                end_line,
            )
        } else {
            // Fallback if tree is not yet parsed
            doc.rope.lines()
                .skip(start_line)
                .take(end_line - start_line)
                .map(|l| Line::raw(l.to_string().trim_matches(|c| c == '\r' || c == '\n').to_string()))
                .collect()
        };

        let (cursor_line, cursor_visual_col) = self.char_to_visual_internal(&doc.rope, self.cursor_offset);
        
        let mut lines = Vec::new();
        for (i, h_line) in highlighted_lines.into_iter().enumerate() {
            let line_idx = start_line + i;
            
            let mut processed_spans = Vec::new();
            let mut current_visual_col_in_line = 0;

            for span in h_line.spans {
                let chars: Vec<char> = span.content.chars().collect();
                for ch in chars {
                    let char_width = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
                    let is_cursor = line_idx == cursor_line && *self.focused.borrow() && current_visual_col_in_line == cursor_visual_col;
                    
                    let mut style = span.style;
                    if is_cursor { style = style.bg(self.theme.cursor_bg).fg(self.theme.cursor_fg); }
                    
                    processed_spans.push(Span::styled(ch.to_string(), style));
                    current_visual_col_in_line += char_width;
                }
            }

            // Handle end-of-line cursor
            if line_idx == cursor_line && *self.focused.borrow() && cursor_visual_col >= current_visual_col_in_line {
                processed_spans.push(Span::styled(" ", Style::default().bg(self.theme.cursor_bg)));
            }

            // Apply horizontal scroll
            let mut scrolled_spans = Vec::new();
            let mut current_v_col = 0;
            for span in processed_spans {
                for ch in span.content.chars() {
                    let w = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
                    if current_v_col >= self.scroll_col {
                        scrolled_spans.push(Span::styled(ch.to_string(), span.style));
                    }
                    current_v_col += w;
                }
            }
            
            lines.push(Line::from(scrolled_spans));
        }
        
        frame.render_widget(Paragraph::new(lines).bg(self.theme.bg), content_area);
    }
}

impl Default for Editor { fn default() -> Self { Self::new() } }
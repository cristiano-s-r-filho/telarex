use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style, Modifier};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders};
use ratatui::Frame;
use ratatui::prelude::Stylize;

/// Purge all control characters, newlines, and carriage returns to prevent terminal staircase effects.
pub fn sanitize(s: &str) -> String {
    s.chars().filter(|c| !c.is_control() && *c != '\n' && *c != '\r').collect()
}

/// Helper to draw a "Bento Box" (themed segment)
pub fn draw_bento_box(frame: &mut Frame, area: Rect, title: &str, border_color: Color, bg_color: Color) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", sanitize(title)))
        .border_style(Style::default().fg(border_color))
        .bg(bg_color);
    frame.render_widget(block, area);
}

/// Helper to create spans for a "Pill" with safe Unicode delimiters
pub fn pill_spans<'a>(content: String, color: Color, bg_color: Color) -> Vec<Span<'a>> {
    // We use standard-width math symbols that resemble Powerline but are more stable
    vec![
        Span::styled(" (", Style::default().fg(color).bg(bg_color)),
        Span::styled(sanitize(&content), Style::default().bg(color).fg(Color::Black).add_modifier(Modifier::BOLD)),
        Span::styled(") ", Style::default().fg(color).bg(bg_color)),
    ]
}

/// Helper function to create a centered rect using up to certain % of the available rect `r`
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let vertical_layout = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Percentage(percent_y),
        Constraint::Fill(1),
    ])
    .split(r);

    let horizontal_layout = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Percentage(percent_x),
        Constraint::Fill(1),
    ])
    .split(vertical_layout[1]);

    horizontal_layout[1]
}

pub fn centered_rect_fixed(width: u16, height: u16, r: Rect) -> Rect {
    let vertical_layout = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(height),
        Constraint::Fill(1),
    ])
    .split(r);
    
    let horizontal_layout = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(width),
        Constraint::Fill(1),
    ])
    .split(vertical_layout[1]);

    horizontal_layout[1]
}

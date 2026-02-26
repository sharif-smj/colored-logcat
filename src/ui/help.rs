use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

const HELP_LINES: &[(&str, &str)] = &[
    ("q / Ctrl+C", "Quit"),
    ("?", "Toggle this help"),
    ("", ""),
    ("--- Filtering ---", ""),
    ("/", "Filter by regex pattern"),
    ("t", "Filter by tag"),
    ("p", "Filter by package name"),
    ("1-6", "Toggle V/D/I/W/E/F levels"),
    ("Esc", "Clear filter / cancel input"),
    ("", ""),
    ("--- Navigation ---", ""),
    ("Space", "Pause / Resume tailing"),
    ("↑/↓ or j/k", "Scroll (when paused)"),
    ("PgUp / PgDn", "Page scroll"),
    ("Mouse wheel", "Scroll logs (up pauses tailing)"),
    ("Right click", "Jump to bottom / resume tail"),
    ("Home", "Jump to top"),
    ("End / G", "Jump to bottom / resume tail"),
    ("", ""),
    ("--- Actions ---", ""),
    ("c", "Clear logcat buffer"),
    ("s", "Save visible logs to file"),
    ("d", "Toggle device panel"),
    ("x", "Toggle crash panel"),
];

pub fn render(f: &mut Frame, area: Rect) {
    let width = 50u16.min(area.width.saturating_sub(4));
    let height = (HELP_LINES.len() as u16 + 2).min(area.height.saturating_sub(4));

    let popup_area = centered_rect(width, height, area);

    let lines: Vec<Line> = HELP_LINES
        .iter()
        .map(|(key, desc)| {
            if key.starts_with("---") {
                Line::from(Span::styled(
                    key.to_string(),
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ))
            } else if key.is_empty() {
                Line::raw("")
            } else {
                Line::from(vec![
                    Span::styled(
                        format!("{:<16}", key),
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(desc.to_string(), Style::default().fg(Color::White)),
                ])
            }
        })
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Help ")
        .border_style(Style::default().fg(Color::Yellow));

    f.render_widget(Clear, popup_area);
    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, popup_area);
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .split(area);
    Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .split(vertical[0])[0]
}

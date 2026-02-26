use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

const HELP_LINES: &[(&str, &str)] = &[
    ("", "Press h (or ?) to toggle this panel."),
    ("", "Esc closes help, or clears filters when help is hidden."),
    ("", ""),
    ("--- Core ---", ""),
    ("q / Ctrl+C", "Quit"),
    ("h / ?", "Toggle help"),
    ("", ""),
    ("--- Filtering ---", ""),
    ("/", "Regex filter"),
    ("t", "Tag filter"),
    ("p", "Package filter (auto PID lookup)"),
    ("1-6", "Toggle V/D/I/W/E/F levels"),
    ("Esc", "Clear filter / cancel input"),
    ("", ""),
    ("--- Navigation ---", ""),
    ("Space", "Pause / resume tailing"),
    ("↑/↓ or j/k", "Scroll line-by-line"),
    ("PgUp / PgDn", "Page scroll"),
    ("Mouse wheel", "Scroll logs (up pauses tailing)"),
    ("Right click", "Jump to bottom / resume tail"),
    ("Home", "Jump to top"),
    ("End / G", "Jump to bottom / tail"),
    ("", ""),
    ("--- Panels/Actions ---", ""),
    ("x", "Toggle crash/ANR panel"),
    ("d", "Toggle device panel"),
    ("s", "Save visible logs"),
    ("c", "Clear logcat buffer"),
];

pub fn render_sidebar(f: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Command Help (h) ")
        .border_style(Style::default().fg(Color::Yellow));

    let paragraph = Paragraph::new(build_lines())
        .block(block)
        .wrap(Wrap { trim: false });
    f.render_widget(paragraph, area);
}

pub fn render_overlay(f: &mut Frame, area: Rect) {
    let width = 56u16.min(area.width.saturating_sub(4));
    let height = (HELP_LINES.len() as u16 + 4).min(area.height.saturating_sub(4));
    let popup_area = centered_rect(width, height, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Command Help (h) ")
        .border_style(Style::default().fg(Color::Yellow));

    f.render_widget(Clear, popup_area);
    let paragraph = Paragraph::new(build_lines())
        .block(block)
        .wrap(Wrap { trim: false });
    f.render_widget(paragraph, popup_area);
}

fn build_lines() -> Vec<Line<'static>> {
    HELP_LINES
        .iter()
        .map(|(key, desc)| {
            if key.starts_with("---") {
                Line::from(Span::styled(
                    key.to_string(),
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ))
            } else if key.is_empty() && desc.is_empty() {
                Line::raw("")
            } else if key.is_empty() {
                Line::from(Span::styled(
                    desc.to_string(),
                    Style::default().fg(Color::Gray),
                ))
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
        .collect()
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .split(area);
    Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .split(vertical[0])[0]
}

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;
use crate::json;
use crate::parser::LogLevel;

fn level_color(level: LogLevel) -> Color {
    match level {
        LogLevel::Verbose => Color::DarkGray,
        LogLevel::Debug => Color::Cyan,
        LogLevel::Info => Color::Gray,
        LogLevel::Warn => Color::Yellow,
        LogLevel::Error => Color::Red,
        LogLevel::Fatal => Color::Magenta,
    }
}

fn level_style(level: LogLevel) -> Style {
    let color = level_color(level);
    let style = Style::default().fg(color);
    if level >= LogLevel::Error {
        style.add_modifier(Modifier::BOLD)
    } else {
        style
    }
}

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let height = area.height.saturating_sub(2) as usize; // block borders
    let total = app.filtered_indices.len();

    let (start, end) = if app.scroll_offset == 0 {
        // Tailing: show last `height` entries
        let start = total.saturating_sub(height);
        (start, total)
    } else {
        let end = total.saturating_sub(app.scroll_offset);
        let start = end.saturating_sub(height);
        (start, end)
    };

    let lines: Vec<Line> = app.filtered_indices[start..end]
        .iter()
        .map(|&idx| {
            let entry = &app.logs[idx];
            render_entry(entry)
        })
        .collect();

    let title = if app.scroll_offset > 0 {
        format!(" Logs [{}/{} | PAUSED] ", end, total)
    } else {
        format!(" Logs [{} | TAILING] ", total)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(Color::DarkGray));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}

fn render_entry(entry: &crate::parser::LogEntry) -> Line<'static> {
    let color = level_color(entry.level);
    let lstyle = level_style(entry.level);

    let mut spans = vec![
        Span::styled(
            format!("{} ", entry.timestamp),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(
            format!("{:>5} {:>5} ", entry.pid, entry.tid),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(format!("{} ", entry.level), lstyle),
        Span::styled(
            format!("{}: ", entry.tag),
            Style::default().fg(Color::DarkGray),
        ),
    ];

    // Check for JSON in message
    if json::detect_json(&entry.message) {
        if let Some(json_spans) = json::json_spans(&entry.message) {
            spans.extend(json_spans);
        } else {
            spans.push(Span::styled(entry.message.clone(), Style::default().fg(color)));
        }
    } else {
        spans.push(Span::styled(entry.message.clone(), Style::default().fg(color)));
    }

    Line::from(spans)
}

pub fn render_crash_panel(f: &mut Frame, area: Rect, app: &App) {
    let height = area.height.saturating_sub(2) as usize;
    let total = app.crash_indices.len();
    let start = total.saturating_sub(height);

    let lines: Vec<Line> = app.crash_indices[start..]
        .iter()
        .map(|&idx| {
            let entry = &app.logs[idx];
            render_entry(entry)
        })
        .collect();

    let title = format!(" Crashes/ANRs [{}] ", total);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(Color::Red));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}

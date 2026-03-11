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
    let (start, end) = app.visible_bounds(height);

    let lines: Vec<Line> = app.filtered_indices[start..end]
        .iter()
        .filter_map(|&idx| {
            app.entry_at(idx)
                .map(|entry| render_entry(entry, app.selection_contains(idx)))
        })
        .collect();

    let title = if app.tailing {
        format!(" Logs [{} | TAILING] ", total)
    } else {
        format!(" Logs [{}/{} | PAUSED] ", end, total)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(Color::DarkGray));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}

fn render_entry(entry: &crate::parser::LogEntry, selected: bool) -> Line<'static> {
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
    if let Some(pretty_json) = entry.pretty_json.as_deref() {
        spans.extend(json::colorize_json(pretty_json));
    } else {
        spans.push(Span::styled(entry.message.clone(), Style::default().fg(color)));
    }

    let mut line = Line::from(spans);
    if selected {
        line = line.patch_style(
            Style::default()
                .bg(Color::Rgb(42, 76, 132))
                .add_modifier(Modifier::BOLD),
        );
    }
    line
}

pub fn render_crash_panel(f: &mut Frame, area: Rect, app: &App) {
    let height = area.height.saturating_sub(2) as usize;
    let total = app.crash_indices.len();
    let start = total.saturating_sub(height);

    let lines: Vec<Line> = app.crash_indices[start..]
        .iter()
        .filter_map(|&idx| app.entry_at(idx).map(|entry| render_entry(entry, false)))
        .collect();

    let title = format!(" Crashes/ANRs [{}] ", total);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(Color::Red));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}

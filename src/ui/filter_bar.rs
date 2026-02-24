use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{App, InputMode};
use crate::parser::LogLevel;

const LEVELS: [LogLevel; 6] = [
    LogLevel::Verbose,
    LogLevel::Debug,
    LogLevel::Info,
    LogLevel::Warn,
    LogLevel::Error,
    LogLevel::Fatal,
];

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

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let mut spans: Vec<Span> = Vec::new();

    // Filter/search input
    match app.input_mode {
        InputMode::Filter => {
            spans.push(Span::styled(" /", Style::default().fg(Color::Yellow)));
            spans.push(Span::styled(
                app.filter_input.clone(),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled("█", Style::default().fg(Color::Yellow)));
        }
        InputMode::Tag => {
            spans.push(Span::styled(" tag:", Style::default().fg(Color::Cyan)));
            spans.push(Span::styled(
                app.filter_input.clone(),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled("█", Style::default().fg(Color::Cyan)));
        }
        InputMode::Package => {
            spans.push(Span::styled(" pkg:", Style::default().fg(Color::Green)));
            spans.push(Span::styled(
                app.filter_input.clone(),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled("█", Style::default().fg(Color::Green)));
        }
        InputMode::Normal => {
            // Show active filter if any
            if let Some(ref re) = app.filters.regex_filter {
                spans.push(Span::styled(
                    format!(" /{}/", re.as_str()),
                    Style::default().fg(Color::Yellow),
                ));
            }
            if let Some(ref tag) = app.filters.tag_filter {
                spans.push(Span::styled(
                    format!(" tag:{}", tag),
                    Style::default().fg(Color::Cyan),
                ));
            }
            if let Some(ref pkg) = app.package_filter {
                let pid_str = app.filters.pid_filter
                    .map(|p| format!(" (pid:{})", p))
                    .unwrap_or_else(|| " (resolving...)".to_string());
                spans.push(Span::styled(
                    format!(" pkg:{}{}", pkg, pid_str),
                    Style::default().fg(Color::Green),
                ));
            }
            if spans.is_empty() {
                spans.push(Span::styled(
                    " No active filters",
                    Style::default().fg(Color::DarkGray),
                ));
            }
        }
    }

    // Separator
    spans.push(Span::styled("  │ ", Style::default().fg(Color::DarkGray)));

    // Level toggles
    for level in &LEVELS {
        let idx = level.index();
        let on = app.filters.level_toggles[idx];
        let color = level_color(*level);
        let key = (idx + 1).to_string();

        if on {
            spans.push(Span::styled(
                format!(" {}{} ", key, level.as_char()),
                Style::default().fg(Color::Black).bg(color),
            ));
        } else {
            spans.push(Span::styled(
                format!(" {}{} ", key, level.as_char()),
                Style::default().fg(Color::DarkGray),
            ));
        }
    }

    let line = Line::from(spans);
    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(Color::DarkGray));
    let paragraph = Paragraph::new(line).block(block);
    f.render_widget(paragraph, area);
}

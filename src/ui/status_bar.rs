use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::App;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let mut spans: Vec<Span> = Vec::new();

    // Log counts
    let total = app.logs.len();
    let filtered = app.filtered_indices.len();
    let errors = app.stats.errors;
    let crashes = app.crash_indices.len();

    spans.push(Span::styled(
        format!(" {} logs", total),
        Style::default().fg(Color::White),
    ));

    if filtered != total {
        spans.push(Span::styled(
            format!(" ({} shown)", filtered),
            Style::default().fg(Color::Yellow),
        ));
    }

    if errors > 0 {
        spans.push(Span::styled(
            format!(" │ {} errors", errors),
            Style::default().fg(Color::Red),
        ));
    }

    if crashes > 0 {
        spans.push(Span::styled(
            format!(" │ {} crashes", crashes),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ));
    }

    // Tailing/Paused state
    if app.scroll_offset > 0 {
        spans.push(Span::styled(
            " │ PAUSED",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ));
    } else {
        spans.push(Span::styled(
            " │ TAILING",
            Style::default().fg(Color::Green),
        ));
    }

    // Connection status
    if let Some(ref status) = app.status_message {
        spans.push(Span::styled(
            format!(" │ {}", status),
            Style::default().fg(Color::Yellow),
        ));
    }

    // Keybind hints (right-aligned via padding)
    let hints = " ?:help /:filter t:tag p:pkg Space:pause ";
    let hints_len = hints.len() as u16;
    let spans_text_len: u16 = spans.iter().map(|s| s.content.len() as u16).sum();
    let padding = area.width.saturating_sub(spans_text_len + hints_len);

    if padding > 0 {
        spans.push(Span::raw(" ".repeat(padding as usize)));
    }
    spans.push(Span::styled(hints, Style::default().fg(Color::DarkGray)));

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));
    f.render_widget(paragraph, area);
}

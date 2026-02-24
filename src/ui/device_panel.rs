use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let lines: Vec<Line> = if app.device_list.is_empty() {
        vec![Line::styled(
            "No devices found",
            Style::default().fg(Color::Yellow),
        )]
    } else {
        app.device_list
            .iter()
            .map(|d| Line::styled(d.clone(), Style::default().fg(Color::White)))
            .collect()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Devices [{}] ", app.device_list.len()))
        .border_style(Style::default().fg(Color::Blue));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}

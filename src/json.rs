use ratatui::style::{Color, Style};
use ratatui::text::Span;
use serde_json::Value;

pub fn detect_json(msg: &str) -> bool {
    let trimmed = msg.trim_start();
    trimmed.starts_with('{') || trimmed.starts_with('[')
}

pub fn json_spans(msg: &str) -> Option<Vec<Span<'static>>> {
    let trimmed = msg.trim_start();
    let value: Value = serde_json::from_str(trimmed).ok()?;
    let pretty = serde_json::to_string_pretty(&value).ok()?;
    Some(colorize_json(&pretty))
}

fn colorize_json(text: &str) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let ch = chars[i];

        match ch {
            '{' | '}' | '[' | ']' => {
                spans.push(Span::styled(
                    ch.to_string(),
                    Style::default().fg(Color::White),
                ));
                i += 1;
            }
            ':' | ',' => {
                spans.push(Span::styled(
                    ch.to_string(),
                    Style::default().fg(Color::DarkGray),
                ));
                i += 1;
            }
            '"' => {
                let start = i;
                i += 1;
                while i < len {
                    if chars[i] == '\\' && i + 1 < len {
                        i += 2;
                    } else if chars[i] == '"' {
                        i += 1;
                        break;
                    } else {
                        i += 1;
                    }
                }
                let s: String = chars[start..i].iter().collect();

                // Look ahead: if next non-whitespace is ':', this is a key
                let rest: String = chars[i..].iter().collect();
                let is_key = rest.trim_start().starts_with(':');

                let color = if is_key { Color::Cyan } else { Color::Green };
                spans.push(Span::styled(s, Style::default().fg(color)));
            }
            _ if ch.is_ascii_digit() || ch == '-' => {
                let start = i;
                while i < len && (chars[i].is_ascii_digit() || matches!(chars[i], '.' | '-' | 'e' | 'E' | '+')) {
                    i += 1;
                }
                let s: String = chars[start..i].iter().collect();
                spans.push(Span::styled(s, Style::default().fg(Color::Yellow)));
            }
            't' | 'f' | 'n' => {
                // true, false, null
                let rest: String = chars[i..].iter().collect();
                let word = if rest.starts_with("true") {
                    "true"
                } else if rest.starts_with("false") {
                    "false"
                } else if rest.starts_with("null") {
                    "null"
                } else {
                    let s = ch.to_string();
                    spans.push(Span::styled(s, Style::default().fg(Color::Gray)));
                    i += 1;
                    continue;
                };
                spans.push(Span::styled(
                    word.to_string(),
                    Style::default().fg(Color::Magenta),
                ));
                i += word.len();
            }
            _ if ch.is_whitespace() => {
                // Collapse whitespace for inline display
                while i < len && chars[i].is_whitespace() {
                    i += 1;
                }
                spans.push(Span::raw(" "));
            }
            _ => {
                spans.push(Span::styled(
                    ch.to_string(),
                    Style::default().fg(Color::Gray),
                ));
                i += 1;
            }
        }
    }

    spans
}

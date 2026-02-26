use regex::Regex;
use serde_json::Value;
use std::fmt;
use std::sync::LazyLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Verbose = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
    Fatal = 5,
}

impl LogLevel {
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            'V' => Some(Self::Verbose),
            'D' => Some(Self::Debug),
            'I' => Some(Self::Info),
            'W' => Some(Self::Warn),
            'E' => Some(Self::Error),
            'F' => Some(Self::Fatal),
            _ => None,
        }
    }

    pub fn as_char(self) -> char {
        match self {
            Self::Verbose => 'V',
            Self::Debug => 'D',
            Self::Info => 'I',
            Self::Warn => 'W',
            Self::Error => 'E',
            Self::Fatal => 'F',
        }
    }

    pub fn index(self) -> usize {
        self as usize
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_char())
    }
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub raw: String,
    pub timestamp: String,
    pub pid: u32,
    pub tid: u32,
    pub level: LogLevel,
    pub tag: String,
    pub message: String,
    pub pretty_json: Option<String>,
}

static LOGCAT_RE: LazyLock<Regex> = LazyLock::new(|| {
    // Handles both "MM-DD HH:MM:SS.mmm" and "YYYY-MM-DD HH:MM:SS.mmm" formats
    Regex::new(r"^(?:\d{4}-)?(\d{2}-\d{2}\s+\d{2}:\d{2}:\d{2}\.\d{3})\s+(\d+)\s+(\d+)\s+([VDIWEF])\s+(.+?):\s*(.*)$")
        .unwrap()
});

impl LogEntry {
    pub fn parse(line: &str) -> Option<Self> {
        let caps = LOGCAT_RE.captures(line)?;

        let level_char = caps[4].chars().next()?;
        let level = LogLevel::from_char(level_char)?;

        let message = caps[6].to_string();

        Some(LogEntry {
            raw: line.to_string(),
            timestamp: caps[1].to_string(),
            pid: caps[2].parse().ok()?,
            tid: caps[3].parse().ok()?,
            level,
            tag: caps[5].trim().to_string(),
            pretty_json: parse_pretty_json(&message),
            message,
        })
    }
}

fn parse_pretty_json(message: &str) -> Option<String> {
    let trimmed = message.trim_start();
    if !(trimmed.starts_with('{') || trimmed.starts_with('[')) {
        return None;
    }

    let value: Value = serde_json::from_str(trimmed).ok()?;
    serde_json::to_string_pretty(&value).ok()
}

use regex::Regex;
use std::sync::LazyLock;

use crate::parser::{LogEntry, LogLevel};

static CRASH_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(AndroidRuntime|FATAL EXCEPTION|FATAL|ANR|crash|System\.err)")
        .unwrap()
});

pub struct FilterSet {
    pub level_toggles: [bool; 6],
    pub tag_filter: Option<String>,
    pub regex_filter: Option<Regex>,
    pub pid_filter: Option<u32>,
}

impl Default for FilterSet {
    fn default() -> Self {
        Self {
            level_toggles: [true; 6],
            tag_filter: None,
            regex_filter: None,
            pid_filter: None,
        }
    }
}

impl FilterSet {
    pub fn matches(&self, entry: &LogEntry) -> bool {
        // Check level toggle
        if !self.level_toggles[entry.level.index()] {
            return false;
        }

        // Check tag filter
        if let Some(ref tag) = self.tag_filter {
            if !entry.tag.contains(tag.as_str()) {
                return false;
            }
        }

        // Check regex filter
        if let Some(ref re) = self.regex_filter {
            if !re.is_match(&entry.message) && !re.is_match(&entry.tag) {
                return false;
            }
        }

        // Check PID filter
        if let Some(pid) = self.pid_filter {
            if entry.pid != pid {
                return false;
            }
        }

        true
    }

    pub fn set_regex(&mut self, pattern: &str) {
        if pattern.is_empty() {
            self.regex_filter = None;
        } else {
            self.regex_filter = Regex::new(pattern).ok();
        }
    }

    pub fn toggle_level(&mut self, level: LogLevel) {
        let idx = level.index();
        self.level_toggles[idx] = !self.level_toggles[idx];
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

pub fn is_crash_entry(entry: &LogEntry) -> bool {
    entry.level >= LogLevel::Error || CRASH_RE.is_match(&entry.raw)
}

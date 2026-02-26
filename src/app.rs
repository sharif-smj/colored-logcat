use std::collections::VecDeque;
use std::env;

use crate::adb;
use crate::export;
use crate::filter::{is_crash_entry, FilterSet};
use crate::parser::{LogEntry, LogLevel};

const DEFAULT_MAX_LOG_ENTRIES: usize = 250_000;
const MIN_MAX_LOG_ENTRIES: usize = 10_000;
const HARD_MAX_LOG_ENTRIES: usize = 2_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Filter,
    Tag,
    Package,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelLayout {
    Single,
    SplitCrash,
    SplitDevice,
}

#[derive(Debug, Default)]
pub struct LogStats {
    pub counts: [usize; 6],
    pub errors: usize,
}

pub struct App {
    pub logs: VecDeque<LogEntry>,
    pub filtered_indices: Vec<usize>, // absolute indices
    pub crash_indices: Vec<usize>,    // absolute indices
    pub filters: FilterSet,
    pub input_mode: InputMode,
    pub filter_input: String,
    pub scroll_offset: usize,
    pub panels: PanelLayout,
    pub show_help: bool,
    pub device_list: Vec<String>,
    pub package_filter: Option<String>,
    pub stats: LogStats,
    pub status_message: Option<String>,
    pub should_quit: bool,
    log_base_index: usize, // absolute index of logs[0]
    max_log_entries: usize,
}

impl App {
    pub fn new() -> Self {
        let max_log_entries = configured_max_log_entries();
        let initial_capacity = max_log_entries.min(100_000);

        Self {
            logs: VecDeque::with_capacity(initial_capacity),
            filtered_indices: Vec::new(),
            crash_indices: Vec::new(),
            filters: FilterSet::default(),
            input_mode: InputMode::Normal,
            filter_input: String::new(),
            scroll_offset: 0,
            panels: PanelLayout::Single,
            show_help: false,
            device_list: Vec::new(),
            package_filter: None,
            stats: LogStats::default(),
            status_message: None,
            should_quit: false,
            log_base_index: 0,
            max_log_entries,
        }
    }

    pub fn add_entry(&mut self, entry: LogEntry) {
        // Update stats
        self.stats.counts[entry.level.index()] += 1;
        if entry.level >= LogLevel::Error {
            self.stats.errors += 1;
        }

        let idx = self.log_base_index + self.logs.len();

        // Check crash
        if is_crash_entry(&entry) {
            self.crash_indices.push(idx);
        }

        // Check filter
        let matches_filter = self.filters.matches(&entry);
        if matches_filter {
            self.filtered_indices.push(idx);
        }

        self.logs.push_back(entry);

        // Evict if over capacity
        if self.logs.len() > self.max_log_entries {
            self.logs.pop_front();
            self.log_base_index += 1;

            // Drop stale absolute indices; no shifting needed.
            self.filtered_indices.retain(|i| *i >= self.log_base_index);
            self.crash_indices.retain(|i| *i >= self.log_base_index);
        }

        // Keep paused viewport anchored to the same entries.
        // Without this, newly appended matching logs shift a paused view forward.
        if self.scroll_offset > 0 && matches_filter {
            self.scroll_offset = self.scroll_offset.saturating_add(1);
        }

        self.clamp_scroll_offset();
    }

    pub fn refilter(&mut self) {
        self.filtered_indices.clear();
        for (idx, entry) in self.logs.iter().enumerate() {
            if self.filters.matches(entry) {
                self.filtered_indices.push(self.log_base_index + idx);
            }
        }
        // Keep crash indices as-is (they don't depend on user filters)
        self.clamp_scroll_offset();
    }

    pub fn scroll_up(&mut self, amount: usize) {
        let max = self.filtered_indices.len().saturating_sub(1);
        self.scroll_offset = (self.scroll_offset + amount).min(max);
    }

    pub fn scroll_down(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = self.filtered_indices.len().saturating_sub(1);
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = 0;
    }

    pub fn entry_at(&self, absolute_idx: usize) -> Option<&LogEntry> {
        let relative_idx = absolute_idx.checked_sub(self.log_base_index)?;
        self.logs.get(relative_idx)
    }

    pub fn toggle_level(&mut self, level: LogLevel) {
        self.filters.toggle_level(level);
        self.refilter();
    }

    pub fn submit_filter_input(&mut self) {
        match self.input_mode {
            InputMode::Filter => {
                self.filters.set_regex(&self.filter_input);
                self.refilter();
            }
            InputMode::Tag => {
                if self.filter_input.is_empty() {
                    self.filters.tag_filter = None;
                } else {
                    self.filters.tag_filter = Some(self.filter_input.clone());
                }
                self.refilter();
            }
            InputMode::Package => {
                if self.filter_input.is_empty() {
                    self.package_filter = None;
                    self.filters.pid_filter = None;
                } else {
                    let pkg = self.filter_input.clone();
                    // Try to resolve PID immediately
                    if let Some(pid) = adb::get_package_pid(&pkg) {
                        self.filters.pid_filter = Some(pid);
                    }
                    self.package_filter = Some(pkg);
                }
                self.refilter();
            }
            InputMode::Normal => {}
        }
        self.filter_input.clear();
        self.input_mode = InputMode::Normal;
    }

    pub fn cancel_input(&mut self) {
        self.filter_input.clear();
        self.input_mode = InputMode::Normal;
    }

    pub fn clear_all_filters(&mut self) {
        self.filters.reset();
        self.package_filter = None;
        self.filter_input.clear();
        self.refilter();
    }

    pub fn clear_logs(&mut self) {
        let _ = adb::clear_buffer();
        self.logs.clear();
        self.filtered_indices.clear();
        self.crash_indices.clear();
        self.stats = LogStats::default();
        self.scroll_offset = 0;
        self.log_base_index = 0;
        self.status_message = Some("Buffer cleared".to_string());
    }

    pub fn export_logs(&mut self) {
        let entries: Vec<&LogEntry> = self.filtered_indices
            .iter()
            .filter_map(|&idx| self.entry_at(idx))
            .collect();

        match export::export_logs(&entries, None) {
            Ok(path) => {
                self.status_message = Some(format!("Saved to {}", path.display()));
            }
            Err(e) => {
                self.status_message = Some(format!("Export failed: {}", e));
            }
        }
    }

    pub fn refresh_devices(&mut self) {
        self.device_list = adb::list_devices();
    }

    pub fn toggle_crash_panel(&mut self) {
        self.panels = match self.panels {
            PanelLayout::SplitCrash => PanelLayout::Single,
            _ => PanelLayout::SplitCrash,
        };
    }

    pub fn toggle_device_panel(&mut self) {
        self.panels = match self.panels {
            PanelLayout::SplitDevice => PanelLayout::Single,
            _ => {
                self.refresh_devices();
                PanelLayout::SplitDevice
            }
        };
    }

    /// Try to resolve package PID if we have a package filter but no PID yet
    pub fn poll_package_pid(&mut self) {
        if let Some(ref pkg) = self.package_filter {
            if self.filters.pid_filter.is_none() {
                if let Some(pid) = adb::get_package_pid(pkg) {
                    self.filters.pid_filter = Some(pid);
                    self.status_message = Some(format!("Found PID {} for {}", pid, pkg));
                    self.refilter();
                }
            }
        }
    }

    fn clamp_scroll_offset(&mut self) {
        let max = self.filtered_indices.len().saturating_sub(1);
        self.scroll_offset = self.scroll_offset.min(max);
    }
}

fn configured_max_log_entries() -> usize {
    env::var("COLORED_LOGCAT_MAX_ENTRIES")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .map(|n| n.clamp(MIN_MAX_LOG_ENTRIES, HARD_MAX_LOG_ENTRIES))
        .unwrap_or(DEFAULT_MAX_LOG_ENTRIES)
}

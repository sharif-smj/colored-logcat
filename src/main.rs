mod adb;
mod app;
mod export;
mod filter;
mod json;
mod parser;
mod ui;

use std::io;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
    MouseButton, MouseEventKind,
};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use app::{App, InputMode};
use parser::LogLevel;

const MOUSE_SCROLL_LINES: usize = 1;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {}", e);
    }

    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new();

    // Start ADB logcat reader
    let (tx, rx) = mpsc::channel();
    let mut _adb_handle = match adb::spawn_logcat(tx) {
        Ok(handle) => handle,
        Err(e) => {
            app.status_message = Some(format!("ADB error: {}", e));
            // Still start the app so user can see the error
            // Create a dummy channel
            let (_dummy_tx, _) = mpsc::channel::<adb::AdbMessage>();
            // We'll just render with no log input
            loop {
                terminal.draw(|f| ui::render(f, &app))?;
                if event::poll(Duration::from_millis(100))? {
                    match event::read()? {
                        Event::Key(key) => {
                            if key.kind == KeyEventKind::Press
                                && (key.code == KeyCode::Char('q') || key.code == KeyCode::Esc)
                            {
                                return Ok(());
                            }
                        }
                        Event::Mouse(mouse) => {
                            if mouse.kind == MouseEventKind::Down(MouseButton::Right) {
                                app.scroll_to_bottom();
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    };

    let mut last_pid_poll = Instant::now();
    let pid_poll_interval = Duration::from_secs(2);

    'app_loop: loop {
        // Drain all available log entries (batched for performance)
        let mut new_entries = 0;
        while let Ok(msg) = rx.try_recv() {
            match msg {
                adb::AdbMessage::Entry(entry) => {
                    app.add_entry(entry);
                    new_entries += 1;
                }
                adb::AdbMessage::UnparsedLine => {
                    // Skip unparsed lines (beginning-of-logcat header, etc.)
                }
                adb::AdbMessage::Disconnected(reason) => {
                    app.status_message = Some(reason);
                }
            }
            // Batch limit: process max 1000 per frame to keep UI responsive
            if new_entries >= 1000 {
                break;
            }
        }

        // Poll for package PID if needed
        if app.package_filter.is_some()
            && app.filters.pid_filter.is_none()
            && last_pid_poll.elapsed() >= pid_poll_interval
        {
            app.poll_package_pid();
            last_pid_poll = Instant::now();
        }

        // Render
        terminal.draw(|f| ui::render(f, &app))?;

        // Handle input events
        if event::poll(Duration::from_millis(16))? {
            if handle_event(&mut app, event::read()?) {
                break;
            }

            while event::poll(Duration::from_millis(0))? {
                if handle_event(&mut app, event::read()?) {
                    break 'app_loop;
                }
            }
        }
    }

    Ok(())
}

fn handle_mouse(app: &mut App, kind: MouseEventKind) {
    match kind {
        // Scroll up to pause and browse older logs.
        MouseEventKind::ScrollUp => app.scroll_up(MOUSE_SCROLL_LINES),
        // Scroll down toward live tailing. At offset 0, stream follows again.
        MouseEventKind::ScrollDown => app.scroll_down(MOUSE_SCROLL_LINES),
        // Quick follow: right-click jumps to bottom and resumes tailing.
        MouseEventKind::Down(MouseButton::Right) => app.scroll_to_bottom(),
        _ => {}
    }
}

fn handle_event(app: &mut App, event: Event) -> bool {
    match event {
        Event::Key(key) => {
            if key.kind != KeyEventKind::Press {
                return false;
            }

            // Global quit
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                return true;
            }

            match app.input_mode {
                InputMode::Normal => handle_normal_key(app, key.code),
                InputMode::Filter | InputMode::Tag | InputMode::Package => {
                    handle_input_key(app, key.code);
                }
            }

            app.should_quit
        }
        Event::Mouse(mouse) => {
            if matches!(app.input_mode, InputMode::Normal) {
                handle_mouse(app, mouse.kind);
            }
            false
        }
        _ => false,
    }
}

fn handle_normal_key(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('h') | KeyCode::Char('?') => app.show_help = !app.show_help,

        // Filter modes
        KeyCode::Char('/') => {
            app.input_mode = InputMode::Filter;
            app.filter_input.clear();
        }
        KeyCode::Char('t') => {
            app.input_mode = InputMode::Tag;
            app.filter_input.clear();
        }
        KeyCode::Char('p') => {
            app.input_mode = InputMode::Package;
            app.filter_input.clear();
        }

        // Level toggles
        KeyCode::Char('1') => app.toggle_level(LogLevel::Verbose),
        KeyCode::Char('2') => app.toggle_level(LogLevel::Debug),
        KeyCode::Char('3') => app.toggle_level(LogLevel::Info),
        KeyCode::Char('4') => app.toggle_level(LogLevel::Warn),
        KeyCode::Char('5') => app.toggle_level(LogLevel::Error),
        KeyCode::Char('6') => app.toggle_level(LogLevel::Fatal),

        // Scrolling
        KeyCode::Char(' ') => {
            if app.scroll_offset == 0 {
                // Start paused at current position
                app.scroll_offset = 1;
            } else {
                app.scroll_to_bottom();
            }
        }
        KeyCode::Up | KeyCode::Char('k') => app.scroll_up(1),
        KeyCode::Down | KeyCode::Char('j') => app.scroll_down(1),
        KeyCode::PageUp => app.scroll_up(20),
        KeyCode::PageDown => app.scroll_down(20),
        KeyCode::Home => app.scroll_to_top(),
        KeyCode::End | KeyCode::Char('G') => app.scroll_to_bottom(),

        // Actions
        KeyCode::Char('c') => app.clear_logs(),
        KeyCode::Char('s') => app.export_logs(),
        KeyCode::Char('d') => app.toggle_device_panel(),
        KeyCode::Char('x') => app.toggle_crash_panel(),

        // Clear all filters
        KeyCode::Esc => {
            if app.show_help {
                app.show_help = false;
            } else {
                app.clear_all_filters();
                app.status_message = Some("Filters cleared".to_string());
            }
        }

        _ => {}
    }
}

fn handle_input_key(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Enter => app.submit_filter_input(),
        KeyCode::Esc => app.cancel_input(),
        KeyCode::Backspace => {
            app.filter_input.pop();
        }
        KeyCode::Char(c) => {
            app.filter_input.push(c);
        }
        _ => {}
    }
}

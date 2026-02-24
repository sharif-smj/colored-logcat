pub mod crash_panel;
pub mod device_panel;
pub mod filter_bar;
pub mod help;
pub mod log_view;
pub mod status_bar;

use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::Frame;

use crate::app::{App, PanelLayout};

pub fn render(f: &mut Frame, app: &App) {
    let size = f.area();

    // Main vertical layout: filter_bar | content | status_bar
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),  // filter bar
            Constraint::Min(5),    // content area
            Constraint::Length(1), // status bar
        ])
        .split(size);

    filter_bar::render(f, main_chunks[0], app);
    status_bar::render(f, main_chunks[2], app);

    // Content area depends on panel layout
    match app.panels {
        PanelLayout::Single => {
            log_view::render(f, main_chunks[1], app);
        }
        PanelLayout::SplitCrash => {
            let h_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                .split(main_chunks[1]);

            log_view::render(f, h_chunks[0], app);
            crash_panel::render(f, h_chunks[1], app);
        }
        PanelLayout::SplitDevice => {
            let h_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Min(40), Constraint::Length(35)])
                .split(main_chunks[1]);

            log_view::render(f, h_chunks[0], app);
            device_panel::render(f, h_chunks[1], app);
        }
    }

    // Help overlay
    if app.show_help {
        help::render(f, size);
    }
}

pub mod crash_panel;
pub mod device_panel;
pub mod filter_bar;
pub mod help;
pub mod log_view;
pub mod status_bar;

use ratatui::layout::{Constraint, Direction, Layout, Rect};
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

    let content_area = main_chunks[1];

    if app.show_help {
        if content_area.width >= 90 {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Min(30), Constraint::Length(44)])
                .split(content_area);

            render_content(f, chunks[0], app);
            help::render_sidebar(f, chunks[1]);
        } else {
            render_content(f, content_area, app);
            help::render_overlay(f, size);
        }
    } else {
        render_content(f, content_area, app);
    }
}

pub fn log_view_area(size: Rect, app: &App) -> Rect {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(5),
            Constraint::Length(1),
        ])
        .split(size);

    let content_area = main_chunks[1];
    let content_area = if app.show_help && content_area.width >= 90 {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(30), Constraint::Length(44)])
            .split(content_area)[0]
    } else {
        content_area
    };

    match app.panels {
        PanelLayout::Single => content_area,
        PanelLayout::SplitCrash => Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(content_area)[0],
        PanelLayout::SplitDevice => Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(40), Constraint::Length(35)])
            .split(content_area)[0],
    }
}

fn render_content(f: &mut Frame, area: Rect, app: &App) {
    // Content area depends on panel layout
    match app.panels {
        PanelLayout::Single => {
            log_view::render(f, area, app);
        }
        PanelLayout::SplitCrash => {
            let h_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                .split(area);

            log_view::render(f, h_chunks[0], app);
            crash_panel::render(f, h_chunks[1], app);
        }
        PanelLayout::SplitDevice => {
            let h_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Min(40), Constraint::Length(35)])
                .split(area);

            log_view::render(f, h_chunks[0], app);
            device_panel::render(f, h_chunks[1], app);
        }
    }
}

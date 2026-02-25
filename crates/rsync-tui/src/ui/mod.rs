pub mod about;
pub mod history;
pub mod job_form;
pub mod job_output;
pub mod jobs;
pub mod popup;
pub mod settings;
pub mod statistics;
pub mod status_bar;
pub mod tabs;
pub mod text_input;
pub mod tools;

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};

use crate::app::{App, Page};

/// Draw the full UI.
pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // tab bar
            Constraint::Min(0),   // main content
            Constraint::Length(1), // status bar
        ])
        .split(f.area());

    tabs::draw_tabs(f, app, chunks[0]);
    status_bar::draw_status_bar(f, app, chunks[2]);

    // Popup takes priority over everything
    if let Some(ref popup_kind) = app.popup {
        draw_page(f, app, chunks[1]);
        popup::draw_popup(f, popup_kind, chunks[1]);
        return;
    }

    // Job output viewer overlays the main content
    if app.job_output.is_some() {
        job_output::draw_job_output(f, app, chunks[1]);
        return;
    }

    // Job form overlays the main content
    if app.job_form.is_some() {
        job_form::draw_job_form(f, app, chunks[1]);
        return;
    }

    draw_page(f, app, chunks[1]);
}

fn draw_page(f: &mut Frame, app: &App, area: Rect) {
    match app.current_page {
        Page::Jobs => jobs::draw_jobs(f, app, area),
        Page::History => history::draw_history(f, app, area),
        Page::Statistics => statistics::draw_statistics(f, app, area),
        Page::Tools => tools::draw_tools(f, app, area),
        Page::Settings => settings::draw_settings(f, app, area),
        Page::About => about::draw_about(f, area),
    }
}

/// Center a rect of given size within a parent rect.
pub fn centered_rect(width: u16, height: u16, r: Rect) -> Rect {
    let x = r.x + (r.width.saturating_sub(width)) / 2;
    let y = r.y + (r.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width.min(r.width), height.min(r.height))
}

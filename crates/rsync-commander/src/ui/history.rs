use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Row, Table};

use rsync_core::models::backup::InvocationStatus;

use crate::app::App;

pub fn draw_history(f: &mut Frame, app: &App, area: Rect) {
    if app.history_state.viewing_log {
        draw_log_viewer(f, app, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(area);

    let header = Row::new(vec!["Job", "Started", "Status", "Exit", "Files", "Trigger"])
        .style(
            Style::default()
                .fg(app.theme.highlight)
                .add_modifier(Modifier::BOLD),
        );

    let rows: Vec<Row> = app
        .history_state
        .invocations
        .iter()
        .enumerate()
        .map(|(i, inv)| {
            let job_name = app
                .job_service
                .get_job(&inv.job_id)
                .map(|j| j.name.clone())
                .unwrap_or_else(|_| inv.job_id.to_string()[..8].to_string());

            let started = inv.started_at.format("%Y-%m-%d %H:%M").to_string();
            let status = format!("{:?}", inv.status);
            let exit_code = inv
                .exit_code
                .map(|c| c.to_string())
                .unwrap_or_else(|| "-".to_string());
            let files = inv.files_transferred.to_string();
            let trigger = format!("{:?}", inv.trigger);

            let style = if i == app.history_state.selected {
                Style::default()
                    .fg(app.theme.selected)
                    .add_modifier(Modifier::BOLD)
            } else {
                let color = match inv.status {
                    InvocationStatus::Succeeded => app.theme.success,
                    InvocationStatus::Failed => app.theme.error,
                    InvocationStatus::Cancelled => ratatui::style::Color::Yellow,
                    InvocationStatus::Running => app.theme.highlight,
                };
                Style::default().fg(color)
            };

            Row::new(vec![job_name, started, status, exit_code, files, trigger]).style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(25),
            Constraint::Percentage(20),
            Constraint::Percentage(15),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(20),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(format!(
                " History ({}) ",
                app.history_state.invocations.len()
            ))
            .borders(Borders::ALL)
            .style(Style::default().fg(app.theme.border)),
    );

    f.render_widget(table, chunks[0]);

    let help = Line::from(vec![
        Span::styled(" j/k", Style::default().fg(app.theme.highlight)),
        Span::styled(":navigate ", Style::default().fg(app.theme.muted)),
        Span::styled("Enter", Style::default().fg(app.theme.highlight)),
        Span::styled(":view log ", Style::default().fg(app.theme.muted)),
        Span::styled("d", Style::default().fg(app.theme.highlight)),
        Span::styled(":delete", Style::default().fg(app.theme.muted)),
    ]);

    f.render_widget(Paragraph::new(help), chunks[1]);
}

fn draw_log_viewer(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(area);

    let block = Block::default()
        .title(format!(
            " Log ({} lines) ",
            app.history_state.log_lines.len()
        ))
        .borders(Borders::ALL)
        .style(Style::default().fg(app.theme.border));

    let inner = block.inner(chunks[0]);
    f.render_widget(block, chunks[0]);

    let visible_height = inner.height as usize;
    let start = app.history_state.log_scroll;

    let lines: Vec<Line> = app
        .history_state
        .log_lines
        .iter()
        .skip(start)
        .take(visible_height)
        .map(|line| {
            let style = if line.contains("STDERR") {
                Style::default().fg(app.theme.error)
            } else {
                Style::default().fg(app.theme.fg)
            };
            Line::from(Span::styled(line.as_str(), style))
        })
        .collect();

    f.render_widget(Paragraph::new(lines), inner);

    let help = Line::from(vec![
        Span::styled(" j/k", Style::default().fg(app.theme.highlight)),
        Span::styled(":scroll ", Style::default().fg(app.theme.muted)),
        Span::styled("g/G", Style::default().fg(app.theme.highlight)),
        Span::styled(":top/bottom ", Style::default().fg(app.theme.muted)),
        Span::styled("PgUp/PgDn", Style::default().fg(app.theme.highlight)),
        Span::styled(":page ", Style::default().fg(app.theme.muted)),
        Span::styled("Esc", Style::default().fg(app.theme.highlight)),
        Span::styled(":close", Style::default().fg(app.theme.muted)),
    ]);

    f.render_widget(Paragraph::new(help), chunks[1]);
}

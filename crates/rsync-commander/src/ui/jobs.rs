use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Row, Table};

use crate::app::App;
use crate::ui::text_input::TextInputWidget;

pub fn draw_jobs(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(if app.jobs_state.search_active { 3 } else { 0 }),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .split(area);

    // Search bar
    if app.jobs_state.search_active {
        let search_block = Block::default()
            .title(" Search (Esc to close) ")
            .borders(Borders::ALL)
            .style(Style::default().fg(app.theme.border));
        let inner = search_block.inner(chunks[0]);
        f.render_widget(search_block, chunks[0]);
        f.render_widget(
            TextInputWidget::new(&app.jobs_state.search_input)
                .focused_style(Style::default().fg(app.theme.fg))
                .unfocused_style(Style::default().fg(app.theme.muted)),
            inner,
        );
    }

    // Jobs table
    let filtered = app.filtered_jobs();

    let header = Row::new(vec!["Name", "Source", "Destination", "Mode", "Enabled", "Status"])
        .style(
            Style::default()
                .fg(app.theme.highlight)
                .add_modifier(Modifier::BOLD),
        );

    let rows: Vec<Row> = filtered
        .iter()
        .enumerate()
        .map(|(i, job)| {
            let is_running = app.job_executor.is_running(&job.id);
            let status = if is_running {
                "Running"
            } else {
                "Idle"
            };

            let source = format_location(&job.source);
            let dest = format_location(&job.destination);
            let mode = format_mode(&job.backup_mode);
            let enabled = if job.enabled { "Yes" } else { "No" };

            let style = if i == app.jobs_state.selected {
                Style::default()
                    .fg(app.theme.selected)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(app.theme.fg)
            };

            Row::new(vec![
                job.name.clone(),
                source,
                dest,
                mode,
                enabled.to_string(),
                status.to_string(),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(25),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(12),
            Constraint::Percentage(8),
            Constraint::Percentage(15),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(format!(" Jobs ({}) ", filtered.len()))
            .borders(Borders::ALL)
            .style(Style::default().fg(app.theme.border)),
    );

    f.render_widget(table, chunks[1]);

    // Bottom help
    let help = Line::from(vec![
        Span::styled(" n", Style::default().fg(app.theme.highlight)),
        Span::styled(":new ", Style::default().fg(app.theme.muted)),
        Span::styled("Enter", Style::default().fg(app.theme.highlight)),
        Span::styled(":edit ", Style::default().fg(app.theme.muted)),
        Span::styled("r", Style::default().fg(app.theme.highlight)),
        Span::styled(":run ", Style::default().fg(app.theme.muted)),
        Span::styled("d", Style::default().fg(app.theme.highlight)),
        Span::styled(":dry-run ", Style::default().fg(app.theme.muted)),
        Span::styled("c", Style::default().fg(app.theme.highlight)),
        Span::styled(":cancel ", Style::default().fg(app.theme.muted)),
        Span::styled("x", Style::default().fg(app.theme.highlight)),
        Span::styled(":delete ", Style::default().fg(app.theme.muted)),
        Span::styled("o", Style::default().fg(app.theme.highlight)),
        Span::styled(":output ", Style::default().fg(app.theme.muted)),
        Span::styled("/", Style::default().fg(app.theme.highlight)),
        Span::styled(":search", Style::default().fg(app.theme.muted)),
    ]);

    f.render_widget(Paragraph::new(help), chunks[2]);
}

fn format_location(loc: &rsync_core::models::job::StorageLocation) -> String {
    match loc {
        rsync_core::models::job::StorageLocation::Local { path } => truncate(path, 25),
        rsync_core::models::job::StorageLocation::RemoteSsh { user, host, path, .. } => {
            truncate(&format!("{}@{}:{}", user, host, path), 25)
        }
        rsync_core::models::job::StorageLocation::RemoteRsync { host, module, path } => {
            truncate(&format!("{}::{}:{}", host, module, path), 25)
        }
    }
}

fn format_mode(mode: &rsync_core::models::job::BackupMode) -> String {
    match mode {
        rsync_core::models::job::BackupMode::Mirror => "Mirror".to_string(),
        rsync_core::models::job::BackupMode::Versioned { .. } => "Versioned".to_string(),
        rsync_core::models::job::BackupMode::Snapshot { .. } => "Snapshot".to_string(),
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}...", &s[..max.saturating_sub(3)])
    } else {
        s.to_string()
    }
}

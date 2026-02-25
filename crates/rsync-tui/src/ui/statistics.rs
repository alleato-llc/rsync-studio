use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Row, Table};

use crate::app::App;

pub fn draw_statistics(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8), // Summary
            Constraint::Min(0),   // Per-job table
            Constraint::Length(2), // Help
        ])
        .split(area);

    // Summary block
    let summary_block = Block::default()
        .title(" Overall Statistics ")
        .borders(Borders::ALL)
        .style(Style::default().fg(app.theme.border));

    let summary_inner = summary_block.inner(chunks[0]);
    f.render_widget(summary_block, chunks[0]);

    if let Some(ref agg) = app.statistics_state.aggregated {
        let lines = vec![
            Line::from(vec![
                Span::styled("  Total Runs:       ", Style::default().fg(app.theme.muted)),
                Span::styled(
                    agg.total_jobs_run.to_string(),
                    Style::default().fg(app.theme.fg).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("  Files Transferred: ", Style::default().fg(app.theme.muted)),
                Span::styled(
                    agg.total_files_transferred.to_string(),
                    Style::default().fg(app.theme.fg),
                ),
            ]),
            Line::from(vec![
                Span::styled("  Bytes Transferred: ", Style::default().fg(app.theme.muted)),
                Span::styled(
                    format_bytes(agg.total_bytes_transferred),
                    Style::default().fg(app.theme.fg),
                ),
            ]),
            Line::from(vec![
                Span::styled("  Total Duration:    ", Style::default().fg(app.theme.muted)),
                Span::styled(
                    format_duration(agg.total_duration_secs),
                    Style::default().fg(app.theme.fg),
                ),
            ]),
            Line::from(vec![
                Span::styled("  Time Saved:        ", Style::default().fg(app.theme.muted)),
                Span::styled(
                    format_duration(agg.total_time_saved_secs),
                    Style::default().fg(app.theme.success),
                ),
            ]),
        ];
        f.render_widget(Paragraph::new(lines), summary_inner);
    } else {
        f.render_widget(
            Paragraph::new("  No statistics recorded yet.")
                .style(Style::default().fg(app.theme.muted)),
            summary_inner,
        );
    }

    // Per-job table
    let header = Row::new(vec!["Job", "Runs", "Files", "Bytes", "Duration", "Time Saved"])
        .style(
            Style::default()
                .fg(app.theme.highlight)
                .add_modifier(Modifier::BOLD),
        );

    let rows: Vec<Row> = app
        .statistics_state
        .per_job
        .iter()
        .enumerate()
        .map(|(i, (name, stats))| {
            let style = if i == app.statistics_state.selected {
                Style::default()
                    .fg(app.theme.selected)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(app.theme.fg)
            };

            Row::new(vec![
                name.clone(),
                stats.total_jobs_run.to_string(),
                stats.total_files_transferred.to_string(),
                format_bytes(stats.total_bytes_transferred),
                format_duration(stats.total_duration_secs),
                format_duration(stats.total_time_saved_secs),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(25),
            Constraint::Percentage(10),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(20),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(" Per-Job Statistics ")
            .borders(Borders::ALL)
            .style(Style::default().fg(app.theme.border)),
    );

    f.render_widget(table, chunks[1]);

    // Help
    let help = Line::from(vec![
        Span::styled(" r", Style::default().fg(app.theme.highlight)),
        Span::styled(":reset ", Style::default().fg(app.theme.muted)),
        Span::styled("e", Style::default().fg(app.theme.highlight)),
        Span::styled(":export ", Style::default().fg(app.theme.muted)),
        Span::styled("j/k", Style::default().fg(app.theme.highlight)),
        Span::styled(":navigate", Style::default().fg(app.theme.muted)),
    ]);

    f.render_widget(Paragraph::new(help), chunks[2]);
}

fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

fn format_duration(secs: f64) -> String {
    if secs < 60.0 {
        format!("{:.1}s", secs)
    } else if secs < 3600.0 {
        format!("{:.0}m {:.0}s", secs / 60.0, secs % 60.0)
    } else {
        let hours = secs / 3600.0;
        let mins = (secs % 3600.0) / 60.0;
        format!("{:.0}h {:.0}m", hours, mins)
    }
}

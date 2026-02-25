use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use rsync_core::models::job::JobStatus;

use crate::app::App;

pub fn draw_job_output(f: &mut Frame, app: &App, area: Rect) {
    let output = match &app.job_output {
        Some(o) => o,
        None => return,
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // progress bar / status
            Constraint::Min(0),   // log lines
            Constraint::Length(2), // help
        ])
        .split(area);

    // Status / Progress bar
    let status_text = if let Some(ref status) = output.status {
        match status.status {
            JobStatus::Completed => format!("Completed (exit code: {})", status.exit_code.unwrap_or(0)),
            JobStatus::Failed => format!(
                "Failed: {}",
                status.error_message.as_deref().unwrap_or("unknown error")
            ),
            JobStatus::Cancelled => "Cancelled".to_string(),
            JobStatus::Running => "Running...".to_string(),
            _ => "Unknown".to_string(),
        }
    } else {
        "Starting...".to_string()
    };

    let progress_text = if let Some(ref prog) = output.progress {
        format!(
            " {:.1}% | {} files | {} | {}",
            prog.percentage, prog.files_transferred, prog.transfer_rate, prog.elapsed
        )
    } else {
        String::new()
    };

    let status_color = if let Some(ref status) = output.status {
        match status.status {
            JobStatus::Completed => app.theme.success,
            JobStatus::Failed => app.theme.error,
            JobStatus::Cancelled => Color::Yellow,
            _ => app.theme.highlight,
        }
    } else {
        app.theme.highlight
    };

    let status_block = Block::default()
        .title(format!(" {} ", output.job_name))
        .borders(Borders::ALL)
        .style(Style::default().fg(app.theme.border));

    let status_inner = status_block.inner(chunks[0]);
    f.render_widget(status_block, chunks[0]);
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(status_text, Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
            Span::styled(progress_text, Style::default().fg(app.theme.muted)),
        ])),
        status_inner,
    );

    // Log lines
    let log_block = Block::default()
        .title(format!(
            " Output ({} lines){} ",
            output.log_lines.len(),
            if output.follow { " [FOLLOW]" } else { "" }
        ))
        .borders(Borders::ALL)
        .style(Style::default().fg(app.theme.border));

    let log_inner = log_block.inner(chunks[1]);
    f.render_widget(log_block, chunks[1]);

    let visible_height = log_inner.height as usize;
    let total = output.log_lines.len();
    let start = if output.follow {
        total.saturating_sub(visible_height)
    } else {
        output.scroll_offset
    };

    let lines: Vec<Line> = output
        .log_lines
        .iter()
        .skip(start)
        .take(visible_height)
        .map(|ll| {
            let style = if ll.is_stderr {
                Style::default().fg(app.theme.error)
            } else {
                Style::default().fg(app.theme.fg)
            };
            Line::from(Span::styled(&ll.line, style))
        })
        .collect();

    f.render_widget(Paragraph::new(lines), log_inner);

    // Help
    let help = Line::from(vec![
        Span::styled(" j/k", Style::default().fg(app.theme.highlight)),
        Span::styled(":scroll ", Style::default().fg(app.theme.muted)),
        Span::styled("f", Style::default().fg(app.theme.highlight)),
        Span::styled(":follow ", Style::default().fg(app.theme.muted)),
        Span::styled("g/G", Style::default().fg(app.theme.highlight)),
        Span::styled(":top/bottom ", Style::default().fg(app.theme.muted)),
        Span::styled("c", Style::default().fg(app.theme.highlight)),
        Span::styled(":cancel ", Style::default().fg(app.theme.muted)),
        Span::styled("Esc", Style::default().fg(app.theme.highlight)),
        Span::styled(":close", Style::default().fg(app.theme.muted)),
    ]);

    f.render_widget(Paragraph::new(help), chunks[2]);
}

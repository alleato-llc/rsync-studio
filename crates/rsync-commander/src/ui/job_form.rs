use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::{App, JobFormMode};
use crate::ui::text_input::TextInputWidget;

pub fn draw_job_form(f: &mut Frame, app: &App, area: Rect) {
    let form = match &app.job_form {
        Some(f) => f,
        None => return,
    };

    let title = match &form.mode {
        JobFormMode::Create => " New Job (Esc to cancel, Ctrl+S to save) ",
        JobFormMode::Edit(_) => " Edit Job (Esc to cancel, Ctrl+S to save) ",
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().fg(app.theme.border));

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Field definitions
    let fields: Vec<(&str, String)> = vec![
        ("Name", form.job.name.clone()),
        ("Description", form.job.description.clone().unwrap_or_default()),
        ("Source Path", format_location(&form.job.source)),
        ("Destination Path", format_location(&form.job.destination)),
        ("Backup Mode", format_mode(&form.job.backup_mode)),
        ("Archive", bool_str(form.job.options.archive)),
        ("Compress", bool_str(form.job.options.compress)),
        ("Verbose", bool_str(form.job.options.verbose)),
        ("Delete", bool_str(form.job.options.delete)),
        ("Dry Run", bool_str(form.job.options.dry_run)),
        ("Save", "[Press Enter to save]".to_string()),
    ];

    let num_fields = fields.len();
    let max_visible = inner.height as usize;

    // Scroll the form if needed
    let scroll = if form.field_index >= max_visible {
        form.field_index - max_visible + 1
    } else {
        0
    };

    let constraints: Vec<Constraint> = (0..max_visible.min(num_fields))
        .map(|_| Constraint::Length(2))
        .collect();

    let field_areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    for (i, field_area) in field_areas.iter().enumerate() {
        let field_idx = i + scroll;
        if field_idx >= num_fields {
            break;
        }

        let (label, current_value) = &fields[field_idx];
        let is_selected = field_idx == form.field_index;
        let is_editing = form.editing && is_selected;

        let label_style = if is_selected {
            Style::default()
                .fg(app.theme.highlight)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(app.theme.muted)
        };

        let row_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(18), Constraint::Min(0)])
            .split(*field_area);

        // Label
        let indicator = if is_selected { "> " } else { "  " };
        f.render_widget(
            Paragraph::new(format!("{}{}", indicator, label)).style(label_style),
            row_chunks[0],
        );

        // Value or text input
        if is_editing && field_idx < form.field_inputs.len() {
            f.render_widget(
                TextInputWidget::new(&form.field_inputs[field_idx])
                    .focused_style(Style::default().fg(app.theme.fg))
                    .unfocused_style(Style::default().fg(app.theme.muted)),
                row_chunks[1],
            );
        } else {
            let val_style = if is_selected {
                Style::default().fg(app.theme.fg)
            } else {
                Style::default().fg(app.theme.muted)
            };
            f.render_widget(
                Paragraph::new(current_value.as_str()).style(val_style),
                row_chunks[1],
            );
        }
    }
}

fn format_location(loc: &rsync_core::models::job::StorageLocation) -> String {
    match loc {
        rsync_core::models::job::StorageLocation::Local { path } => path.clone(),
        rsync_core::models::job::StorageLocation::RemoteSsh { user, host, path, .. } => {
            format!("{}@{}:{}", user, host, path)
        }
        rsync_core::models::job::StorageLocation::RemoteRsync { host, module, path } => {
            format!("{}::{}:{}", host, module, path)
        }
    }
}

fn format_mode(mode: &rsync_core::models::job::BackupMode) -> String {
    match mode {
        rsync_core::models::job::BackupMode::Mirror => "Mirror".to_string(),
        rsync_core::models::job::BackupMode::Versioned { backup_dir } => {
            format!("Versioned ({})", backup_dir)
        }
        rsync_core::models::job::BackupMode::Snapshot { .. } => "Snapshot".to_string(),
    }
}

fn bool_str(v: bool) -> String {
    if v { "[x]".to_string() } else { "[ ]".to_string() }
}

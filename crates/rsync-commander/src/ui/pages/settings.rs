use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;
use crate::ui::text_input::TextInputWidget;

pub fn draw_settings(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(area);

    let block = Block::default()
        .title(" Settings ")
        .borders(Borders::ALL)
        .style(Style::default().fg(app.theme.border));
    let inner = block.inner(chunks[0]);
    f.render_widget(block, chunks[0]);

    let settings: Vec<(&str, String)> = vec![
        ("Log Directory", app.pages.settings.log_directory.clone()),
        (
            "Max Log Age (days)",
            app.pages.settings.max_log_age_days.to_string(),
        ),
        (
            "Max History/Job",
            app.pages.settings.max_history_per_job.to_string(),
        ),
        (
            "Auto Trailing Slash",
            if app.pages.settings.auto_trailing_slash {
                "Yes".to_string()
            } else {
                "No".to_string()
            },
        ),
        ("TUI Theme", app.pages.settings.tui_theme.clone()),
    ];

    let row_constraints: Vec<Constraint> = settings.iter().map(|_| Constraint::Length(2)).collect();
    let row_areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints(row_constraints)
        .split(inner);

    for (i, (label, value)) in settings.iter().enumerate() {
        if i >= row_areas.len() {
            break;
        }

        let is_selected = i == app.pages.settings.selected;
        let is_editing = app.pages.settings.editing && is_selected;

        let label_style = if is_selected {
            Style::default()
                .fg(app.theme.highlight)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(app.theme.muted)
        };

        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(24), Constraint::Min(0)])
            .split(row_areas[i]);

        let indicator = if is_selected { "> " } else { "  " };
        f.render_widget(
            Paragraph::new(format!("{}{}", indicator, label)).style(label_style),
            cols[0],
        );

        if is_editing {
            f.render_widget(
                TextInputWidget::new(&app.pages.settings.edit_input)
                    .focused_style(Style::default().fg(app.theme.fg))
                    .unfocused_style(Style::default().fg(app.theme.muted)),
                cols[1],
            );
        } else {
            let val_style = if is_selected {
                Style::default().fg(app.theme.fg)
            } else {
                Style::default().fg(app.theme.muted)
            };
            f.render_widget(Paragraph::new(value.as_str()).style(val_style), cols[1]);
        }
    }

    // Help
    let help = Line::from(vec![
        Span::styled(" j/k", Style::default().fg(app.theme.highlight)),
        Span::styled(":navigate ", Style::default().fg(app.theme.muted)),
        Span::styled("Enter", Style::default().fg(app.theme.highlight)),
        Span::styled(":edit/toggle ", Style::default().fg(app.theme.muted)),
        Span::styled("e", Style::default().fg(app.theme.highlight)),
        Span::styled(":export jobs", Style::default().fg(app.theme.muted)),
    ]);

    f.render_widget(Paragraph::new(help), chunks[1]);
}

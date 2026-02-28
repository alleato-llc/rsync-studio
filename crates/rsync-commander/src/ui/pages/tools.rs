use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::App;
use crate::ui::text_input::TextInputWidget;

pub fn draw_tools(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Sub-tabs
            Constraint::Min(0),   // Content
            Constraint::Length(2), // Help
        ])
        .split(area);

    // Sub-tabs
    let tabs = vec!["Explainer", "Log Scrubber"];
    let tab_line = Line::from(
        tabs.iter()
            .enumerate()
            .map(|(i, label)| {
                let style = if i == app.pages.tools.active_tab {
                    Style::default()
                        .fg(app.theme.highlight)
                        .add_modifier(Modifier::BOLD | Modifier::REVERSED)
                } else {
                    Style::default().fg(app.theme.fg)
                };
                Span::styled(format!(" {} ", label), style)
            })
            .collect::<Vec<_>>(),
    );
    f.render_widget(Paragraph::new(tab_line), chunks[0]);

    match app.pages.tools.active_tab {
        0 => draw_explainer(f, app, chunks[1]),
        1 => draw_scrubber(f, app, chunks[1]),
        _ => {}
    }

    let help = Line::from(vec![
        Span::styled(" Tab", Style::default().fg(app.theme.highlight)),
        Span::styled(":switch tool ", Style::default().fg(app.theme.muted)),
        Span::styled("Enter/i", Style::default().fg(app.theme.highlight)),
        Span::styled(":edit input ", Style::default().fg(app.theme.muted)),
    ]);
    f.render_widget(Paragraph::new(help), chunks[2]);
}

fn draw_explainer(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Input
            Constraint::Min(0),   // Output
        ])
        .split(area);

    // Input
    let input_block = Block::default()
        .title(" rsync command (Enter to explain) ")
        .borders(Borders::ALL)
        .style(Style::default().fg(app.theme.border));
    let input_inner = input_block.inner(chunks[0]);
    f.render_widget(input_block, chunks[0]);
    f.render_widget(
        TextInputWidget::new(&app.pages.tools.command_input)
            .focused_style(Style::default().fg(app.theme.fg))
            .unfocused_style(Style::default().fg(app.theme.muted)),
        input_inner,
    );

    // Output
    let output_block = Block::default()
        .title(" Explanation ")
        .borders(Borders::ALL)
        .style(Style::default().fg(app.theme.border));
    let output_inner = output_block.inner(chunks[1]);
    f.render_widget(output_block, chunks[1]);

    if let Some(ref err) = app.pages.tools.explanation_error {
        f.render_widget(
            Paragraph::new(format!("Error: {}", err))
                .style(Style::default().fg(app.theme.error))
                .wrap(Wrap { trim: false }),
            output_inner,
        );
    } else if let Some(ref explanation) = app.pages.tools.explanation {
        let mut lines = vec![
            Line::from(Span::styled(
                &explanation.summary,
                Style::default()
                    .fg(app.theme.fg)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
        ];

        for arg_exp in &explanation.arguments {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {:12}", arg_exp.argument),
                    Style::default()
                        .fg(app.theme.highlight)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(&arg_exp.description, Style::default().fg(app.theme.fg)),
            ]));
        }

        f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), output_inner);
    } else {
        f.render_widget(
            Paragraph::new("Enter an rsync command above and press Enter to see an explanation.")
                .style(Style::default().fg(app.theme.muted)),
            output_inner,
        );
    }
}

fn draw_scrubber(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Input
            Constraint::Min(0),   // Instructions
        ])
        .split(area);

    let input_block = Block::default()
        .title(" Search pattern ")
        .borders(Borders::ALL)
        .style(Style::default().fg(app.theme.border));
    let input_inner = input_block.inner(chunks[0]);
    f.render_widget(input_block, chunks[0]);
    f.render_widget(
        TextInputWidget::new(&app.pages.tools.scrub_input)
            .focused_style(Style::default().fg(app.theme.fg))
            .unfocused_style(Style::default().fg(app.theme.muted)),
        input_inner,
    );

    let instructions = Paragraph::new(
        "Enter a regex pattern to search through log files.\n\
         The log scrubber can find and redact sensitive information in log files."
    )
    .style(Style::default().fg(app.theme.muted))
    .block(
        Block::default()
            .title(" Log Scrubber ")
            .borders(Borders::ALL)
            .style(Style::default().fg(app.theme.border)),
    )
    .wrap(Wrap { trim: false });

    f.render_widget(instructions, chunks[1]);
}

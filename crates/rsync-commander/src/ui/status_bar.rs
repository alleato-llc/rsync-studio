use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::app::App;

pub fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let running = app.running_count();
    let running_text = if running > 0 {
        format!(" Running: {} job{} ", running, if running == 1 { "" } else { "s" })
    } else {
        " Idle ".to_string()
    };

    let running_style = if running > 0 {
        Style::default()
            .fg(app.theme.success)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(app.theme.muted)
    };

    let help_text = " q:quit ?:help ";

    let line = Line::from(vec![
        Span::styled(running_text, running_style),
        Span::styled("│", Style::default().fg(app.theme.border)),
        Span::styled(help_text, Style::default().fg(app.theme.muted)),
    ]);

    let bar = Paragraph::new(line)
        .style(Style::default().bg(app.theme.bg));

    f.render_widget(bar, area);
}

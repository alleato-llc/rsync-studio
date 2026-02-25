use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::app::{App, Page};

pub fn draw_tabs(f: &mut Frame, app: &App, area: Rect) {
    let mut spans = Vec::new();

    for (i, page) in Page::ALL.iter().enumerate() {
        let label = format!(" {}:{} ", i + 1, page.label());
        let style = if *page == app.current_page {
            Style::default()
                .fg(app.theme.highlight)
                .add_modifier(Modifier::BOLD | Modifier::REVERSED)
        } else {
            Style::default().fg(app.theme.fg)
        };
        spans.push(Span::styled(label, style));
    }

    let tabs_line = Line::from(spans);
    let tabs = Paragraph::new(tabs_line)
        .style(Style::default().bg(app.theme.bg));

    f.render_widget(tabs, area);
}

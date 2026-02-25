use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

pub fn draw_about(f: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" About ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  Rsync Studio",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  A cross-platform desktop & terminal UI for rsync",
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Version:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                env!("CARGO_PKG_VERSION"),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Stack:    ", Style::default().fg(Color::DarkGray)),
            Span::styled("Rust + ratatui + crossterm", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  Core:     ", Style::default().fg(Color::DarkGray)),
            Span::styled("rsync-core (shared library)", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  Built with rsync-core, shared between the GUI and TUI frontends.",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  License: MIT",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

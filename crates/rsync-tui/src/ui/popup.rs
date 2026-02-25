use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use crate::app::PopupKind;

pub fn draw_popup(f: &mut Frame, popup: &PopupKind, area: Rect) {
    match popup {
        PopupKind::Help => draw_help(f, area),
        PopupKind::Confirm { title, message, .. } => draw_confirm(f, title, message, area),
        PopupKind::Error(msg) => draw_error(f, msg, area),
    }
}

fn draw_help(f: &mut Frame, area: Rect) {
    let text = Text::from(vec![
        Line::from("Global Keybindings").style(Style::default().add_modifier(Modifier::BOLD)),
        Line::from(""),
        Line::from("  1-6          Switch pages"),
        Line::from("  Tab/S-Tab    Cycle pages"),
        Line::from("  q / Ctrl+C   Quit"),
        Line::from("  ?            This help"),
        Line::from(""),
        Line::from("Jobs Page").style(Style::default().add_modifier(Modifier::BOLD)),
        Line::from(""),
        Line::from("  j/k          Navigate"),
        Line::from("  n            New job"),
        Line::from("  Enter        Edit job"),
        Line::from("  r            Run job"),
        Line::from("  d            Dry-run"),
        Line::from("  c            Cancel running job"),
        Line::from("  x            Delete job"),
        Line::from("  o            View output"),
        Line::from("  /            Search"),
        Line::from(""),
        Line::from("Output Viewer").style(Style::default().add_modifier(Modifier::BOLD)),
        Line::from(""),
        Line::from("  j/k          Scroll"),
        Line::from("  g/G          Top/Bottom"),
        Line::from("  f            Toggle follow"),
        Line::from("  c            Cancel job"),
        Line::from("  PgUp/PgDn    Page scroll"),
        Line::from("  Esc          Close"),
        Line::from(""),
        Line::from("History Page").style(Style::default().add_modifier(Modifier::BOLD)),
        Line::from(""),
        Line::from("  j/k          Navigate"),
        Line::from("  Enter        View log"),
        Line::from("  d            Delete invocation"),
        Line::from(""),
        Line::from("Statistics").style(Style::default().add_modifier(Modifier::BOLD)),
        Line::from(""),
        Line::from("  r            Reset all"),
        Line::from("  e            Export"),
        Line::from("  j/k          Navigate per-job"),
        Line::from(""),
        Line::from("Press Esc/q/? to close"),
    ]);

    let height = (text.lines.len() + 2).min(area.height as usize) as u16;
    let width = 45.min(area.width);
    let popup_area = super::centered_rect(width, height, area);

    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .style(Style::default());

    f.render_widget(Clear, popup_area);
    f.render_widget(
        Paragraph::new(text)
            .block(block)
            .wrap(Wrap { trim: false }),
        popup_area,
    );
}

fn draw_confirm(f: &mut Frame, title: &str, message: &str, area: Rect) {
    let text = Text::from(vec![
        Line::from(message.to_string()),
        Line::from(""),
        Line::from("  [y] Yes   [n] No"),
    ]);

    let height = 6.min(area.height);
    let width = 50.min(area.width);
    let popup_area = super::centered_rect(width, height, area);

    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .style(Style::default().fg(ratatui::style::Color::Yellow));

    f.render_widget(Clear, popup_area);
    f.render_widget(
        Paragraph::new(text)
            .block(block)
            .wrap(Wrap { trim: false }),
        popup_area,
    );
}

fn draw_error(f: &mut Frame, msg: &str, area: Rect) {
    let text = Text::from(vec![
        Line::from(msg.to_string()),
        Line::from(""),
        Line::from("Press Enter/Esc to dismiss"),
    ]);

    let height = 6.min(area.height);
    let width = 60.min(area.width);
    let popup_area = super::centered_rect(width, height, area);

    let block = Block::default()
        .title(" Info ")
        .borders(Borders::ALL)
        .style(Style::default().fg(ratatui::style::Color::Cyan));

    f.render_widget(Clear, popup_area);
    f.render_widget(
        Paragraph::new(text)
            .block(block)
            .wrap(Wrap { trim: false }),
        popup_area,
    );
}

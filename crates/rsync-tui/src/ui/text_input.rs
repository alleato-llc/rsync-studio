use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;

#[derive(Debug, Clone)]
pub struct TextInput {
    value: String,
    cursor: usize,
    pub is_focused: bool,
}

impl TextInput {
    pub fn new() -> Self {
        Self {
            value: String::new(),
            cursor: 0,
            is_focused: false,
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn set_value(&mut self, val: &str) {
        self.value = val.to_string();
        self.cursor = self.value.len();
    }

    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor = 0;
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        // Handle Ctrl+ shortcuts before plain Char
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('a') => { self.cursor = 0; return; }
                KeyCode::Char('e') => { self.cursor = self.value.len(); return; }
                KeyCode::Char('u') => {
                    self.value = self.value[self.cursor..].to_string();
                    self.cursor = 0;
                    return;
                }
                KeyCode::Char('k') => { self.value.truncate(self.cursor); return; }
                _ => {}
            }
        }

        match key.code {
            KeyCode::Char(c) => {
                self.value.insert(self.cursor, c);
                self.cursor += 1;
            }
            KeyCode::Backspace => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.value.remove(self.cursor);
                }
            }
            KeyCode::Delete => {
                if self.cursor < self.value.len() {
                    self.value.remove(self.cursor);
                }
            }
            KeyCode::Left => {
                self.cursor = self.cursor.saturating_sub(1);
            }
            KeyCode::Right => {
                self.cursor = (self.cursor + 1).min(self.value.len());
            }
            KeyCode::Home => {
                self.cursor = 0;
            }
            KeyCode::End => {
                self.cursor = self.value.len();
            }
            _ => {}
        }
    }

    pub fn cursor_position(&self) -> usize {
        self.cursor
    }
}

/// A renderable widget for TextInput.
pub struct TextInputWidget<'a> {
    input: &'a TextInput,
    focused_style: Style,
    unfocused_style: Style,
}

impl<'a> TextInputWidget<'a> {
    pub fn new(input: &'a TextInput) -> Self {
        Self {
            input,
            focused_style: Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            unfocused_style: Style::default().fg(Color::DarkGray),
        }
    }

    pub fn focused_style(mut self, style: Style) -> Self {
        self.focused_style = style;
        self
    }

    pub fn unfocused_style(mut self, style: Style) -> Self {
        self.unfocused_style = style;
        self
    }
}

impl Widget for TextInputWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = if self.input.is_focused {
            self.focused_style
        } else {
            self.unfocused_style
        };

        let display = if self.input.value.is_empty() && !self.input.is_focused {
            "(empty)".to_string()
        } else {
            self.input.value.clone()
        };

        let width = area.width as usize;
        // Scroll the display so the cursor is visible
        let scroll = if self.input.cursor > width.saturating_sub(2) {
            self.input.cursor - width.saturating_sub(2)
        } else {
            0
        };

        let visible: String = display.chars().skip(scroll).take(width).collect();
        buf.set_string(area.x, area.y, &visible, style);

        // Draw cursor if focused
        if self.input.is_focused && area.width > 0 {
            let cursor_x = (self.input.cursor - scroll).min(width.saturating_sub(1)) as u16;
            if area.x + cursor_x < area.x + area.width {
                let cell = &mut buf[(area.x + cursor_x, area.y)];
                cell.set_style(Style::default().bg(Color::White).fg(Color::Black));
            }
        }
    }
}

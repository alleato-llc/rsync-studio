use ratatui::style::Color;

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: &'static str,
    pub bg: Color,
    pub fg: Color,
    pub highlight: Color,
    pub border: Color,
    pub selected: Color,
    pub error: Color,
    pub success: Color,
    pub muted: Color,
}

pub const THEMES: &[Theme] = &[
    Theme {
        name: "Default",
        bg: Color::Reset,
        fg: Color::Reset,
        highlight: Color::Cyan,
        border: Color::DarkGray,
        selected: Color::LightCyan,
        error: Color::Red,
        success: Color::Green,
        muted: Color::DarkGray,
    },
    Theme {
        name: "Dark",
        bg: Color::Black,
        fg: Color::White,
        highlight: Color::LightBlue,
        border: Color::DarkGray,
        selected: Color::LightBlue,
        error: Color::LightRed,
        success: Color::LightGreen,
        muted: Color::Gray,
    },
    Theme {
        name: "Solarized",
        bg: Color::Rgb(0, 43, 54),
        fg: Color::Rgb(131, 148, 150),
        highlight: Color::Rgb(38, 139, 210),
        border: Color::Rgb(88, 110, 117),
        selected: Color::Rgb(42, 161, 152),
        error: Color::Rgb(220, 50, 47),
        success: Color::Rgb(133, 153, 0),
        muted: Color::Rgb(88, 110, 117),
    },
    Theme {
        name: "Nord",
        bg: Color::Rgb(46, 52, 64),
        fg: Color::Rgb(216, 222, 233),
        highlight: Color::Rgb(136, 192, 208),
        border: Color::Rgb(76, 86, 106),
        selected: Color::Rgb(129, 161, 193),
        error: Color::Rgb(191, 97, 106),
        success: Color::Rgb(163, 190, 140),
        muted: Color::Rgb(76, 86, 106),
    },
];

pub fn get_theme(name: &str) -> &'static Theme {
    THEMES
        .iter()
        .find(|t| t.name.eq_ignore_ascii_case(name))
        .unwrap_or(&THEMES[0])
}

pub fn theme_names() -> Vec<&'static str> {
    THEMES.iter().map(|t| t.name).collect()
}

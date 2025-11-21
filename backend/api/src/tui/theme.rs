use ratatui::style::Color;
use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
pub enum ThemeKind {
    Dracula,
    OneDark,
    Material,
}

impl ThemeKind {
    pub fn next(self) -> Self {
        match self {
            ThemeKind::Dracula => ThemeKind::OneDark,
            ThemeKind::OneDark => ThemeKind::Material,
            ThemeKind::Material => ThemeKind::Dracula,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            ThemeKind::Dracula => "dracula",
            ThemeKind::OneDark => "onedark",
            ThemeKind::Material => "material",
        }
    }
}

impl FromStr for ThemeKind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "dracula" => Ok(ThemeKind::Dracula),
            "onedark" | "one-dark" | "one_dark" => Ok(ThemeKind::OneDark),
            "material" => Ok(ThemeKind::Material),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ThemePalette {
    pub bg: Color,
    pub panel_bg: Color,
    pub text: Color,
    pub text_muted: Color,
    pub accent: Color,
    pub accent_alt: Color,
    pub border: Color,
    pub header_fg: Color,
    pub header_border: Color,
    pub input_label: Color,
    pub input_label_focus: Color,
    pub submit_fg: Color,
    pub submit_fg_focus: Color,
    pub error_fg: Color,
    pub error_border: Color,
    pub table_header_bg: Color,
    pub table_header_fg: Color,
    pub table_slug_fg: Color,
    pub table_row_even_bg: Color,
    pub table_row_odd_bg: Color,
    pub highlight_bg: Color,
    pub highlight_fg: Color,
    pub logs_title_fg: Color,
    pub logs_border: Color,
    pub footer_fg: Color,
}

pub fn theme_palette(theme: ThemeKind) -> ThemePalette {
    match theme {
        ThemeKind::Dracula => {
            let bg = Color::Rgb(5, 10, 20);
            let panel = bg;
            let accent = Color::Blue;
            ThemePalette {
                bg,
                panel_bg: panel,
                text: Color::Gray,
                text_muted: Color::DarkGray,
                accent,
                accent_alt: accent,
                border: Color::DarkGray,
                header_fg: accent,
                header_border: accent,
                input_label: Color::Gray,
                input_label_focus: accent,
                submit_fg: accent,
                submit_fg_focus: accent,
                error_fg: Color::Red,
                error_border: Color::Red,
                table_header_bg: panel,
                table_header_fg: Color::Gray,
                table_slug_fg: accent,
                table_row_even_bg: panel,
                table_row_odd_bg: panel,
                highlight_bg: accent,
                highlight_fg: Color::White,
                logs_title_fg: accent,
                logs_border: Color::DarkGray,
                footer_fg: Color::DarkGray,
            }
        }
        ThemeKind::OneDark => {
            let bg = Color::Rgb(12, 16, 22);
            let panel = bg;
            let accent = Color::Rgb(97, 175, 239);
            ThemePalette {
                bg,
                panel_bg: panel,
                text: Color::Rgb(171, 178, 191),
                text_muted: Color::Rgb(92, 99, 112),
                accent,
                accent_alt: accent,
                border: Color::Rgb(40, 44, 52),
                header_fg: accent,
                header_border: accent,
                input_label: Color::Rgb(171, 178, 191),
                input_label_focus: accent,
                submit_fg: accent,
                submit_fg_focus: accent,
                error_fg: Color::Rgb(224, 108, 117),
                error_border: Color::Rgb(224, 108, 117),
                table_header_bg: panel,
                table_header_fg: Color::Rgb(171, 178, 191),
                table_slug_fg: accent,
                table_row_even_bg: panel,
                table_row_odd_bg: panel,
                highlight_bg: accent,
                highlight_fg: Color::Black,
                logs_title_fg: accent,
                logs_border: Color::Rgb(40, 44, 52),
                footer_fg: Color::Rgb(92, 99, 112),
            }
        }
        ThemeKind::Material => {
            let bg = Color::Rgb(18, 18, 18);
            let panel = bg;
            let accent = Color::Rgb(3, 169, 244);
            ThemePalette {
                bg,
                panel_bg: panel,
                text: Color::Rgb(224, 224, 224),
                text_muted: Color::Rgb(158, 158, 158),
                accent,
                accent_alt: accent,
                border: Color::Rgb(66, 66, 66),
                header_fg: accent,
                header_border: accent,
                input_label: Color::Rgb(189, 189, 189),
                input_label_focus: accent,
                submit_fg: accent,
                submit_fg_focus: accent,
                error_fg: Color::Rgb(244, 67, 54),
                error_border: Color::Rgb(244, 67, 54),
                table_header_bg: panel,
                table_header_fg: Color::Rgb(224, 224, 224),
                table_slug_fg: accent,
                table_row_even_bg: panel,
                table_row_odd_bg: panel,
                highlight_bg: accent,
                highlight_fg: Color::Black,
                logs_title_fg: accent,
                logs_border: Color::Rgb(66, 66, 66),
                footer_fg: Color::Rgb(158, 158, 158),
            }
        }
    }
}


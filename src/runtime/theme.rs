use crate::config::Theme;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ThemePalette {
    pub header: &'static str,
    pub label: &'static str,
    pub value: &'static str,
    pub bar_low: &'static str,
    pub bar_mid: &'static str,
    pub bar_high: &'static str,
    pub dim: &'static str,
    pub warning: &'static str,
    pub reset: &'static str,
    pub bold: &'static str,
}

pub fn resolve_theme(theme: Theme) -> ThemePalette {
    match theme {
        Theme::Dark | Theme::Auto => ThemePalette {
            header: "\x1b[38;5;87m",
            label: "\x1b[38;5;252m",
            value: "\x1b[38;5;255m",
            bar_low: "\x1b[38;5;78m",
            bar_mid: "\x1b[38;5;220m",
            bar_high: "\x1b[38;5;196m",
            dim: "\x1b[38;5;242m",
            warning: "\x1b[38;5;220m",
            reset: "\x1b[0m",
            bold: "\x1b[1m",
        },
        Theme::Light => ThemePalette {
            header: "\x1b[38;5;27m",
            label: "\x1b[38;5;236m",
            value: "\x1b[38;5;232m",
            bar_low: "\x1b[38;5;28m",
            bar_mid: "\x1b[38;5;172m",
            bar_high: "\x1b[38;5;160m",
            dim: "\x1b[38;5;245m",
            warning: "\x1b[38;5;160m",
            reset: "\x1b[0m",
            bold: "\x1b[1m",
        },
        Theme::Classic => ThemePalette {
            header: "\x1b[36m",
            label: "\x1b[37m",
            value: "\x1b[97m",
            bar_low: "\x1b[32m",
            bar_mid: "\x1b[33m",
            bar_high: "\x1b[31m",
            dim: "\x1b[90m",
            warning: "\x1b[33m",
            reset: "\x1b[0m",
            bold: "\x1b[1m",
        },
    }
}

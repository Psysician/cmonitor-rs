use crate::config::Theme;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ThemePalette {
    pub accent: &'static str,
    pub warning: &'static str,
    pub subtle: &'static str,
}

pub fn resolve_theme(theme: Theme) -> ThemePalette {
    match theme {
        Theme::Light => ThemePalette {
            accent: "blue",
            warning: "red",
            subtle: "black",
        },
        Theme::Dark => ThemePalette {
            accent: "cyan",
            warning: "yellow",
            subtle: "white",
        },
        Theme::Classic => ThemePalette {
            accent: "green",
            warning: "magenta",
            subtle: "white",
        },
        Theme::Auto => ThemePalette {
            accent: "cyan",
            warning: "yellow",
            subtle: "default",
        },
    }
}

use crate::parser::LogLevel;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemePreference {
    #[default]
    Auto,
    Dark,
    Light,
}

impl ThemePreference {
    pub const ALL: [Self; 3] = [Self::Auto, Self::Dark, Self::Light];

    pub fn label(self) -> &'static str {
        match self {
            Self::Auto => "Auto",
            Self::Dark => "Dark",
            Self::Light => "Light",
        }
    }

    pub fn cycle(self, forward: bool) -> Self {
        let idx = Self::ALL.iter().position(|p| *p == self).unwrap_or(0);
        let next = if forward {
            (idx + 1) % Self::ALL.len()
        } else {
            (idx + Self::ALL.len() - 1) % Self::ALL.len()
        };
        Self::ALL[next]
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub shell_fg: Color,
    pub shell_muted: Color,
    pub shell_divider: Color,
    pub shell_hint: Color,
    pub level_verbose: Color,
    pub level_debug: Color,
    pub level_info: Color,
    pub level_warn: Color,
    pub level_error: Color,
    pub focus_fg: Color,
    pub focus_bg: Color,
    pub selection_fg: Color,
    pub selection_bg: Color,
    pub find_fg: Color,
    pub find_bg: Color,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            shell_fg: rgb(212, 212, 212),
            shell_muted: rgb(140, 140, 140),
            shell_divider: rgb(80, 80, 80),
            shell_hint: rgb(120, 120, 120),
            level_verbose: rgb(128, 128, 128),
            level_debug: rgb(86, 156, 214),
            level_info: rgb(212, 212, 212),
            level_warn: rgb(255, 204, 0),
            level_error: rgb(244, 71, 71),
            focus_fg: Color::Black,
            focus_bg: rgb(255, 204, 0),
            selection_fg: Color::White,
            selection_bg: rgb(0, 100, 200),
            find_fg: Color::Black,
            find_bg: rgb(255, 204, 0),
        }
    }

    pub fn light() -> Self {
        Self {
            shell_fg: Color::Black,
            shell_muted: rgb(90, 90, 90),
            shell_divider: rgb(180, 180, 180),
            shell_hint: rgb(100, 100, 100),
            level_verbose: rgb(100, 100, 100),
            level_debug: rgb(0, 92, 153),
            level_info: Color::Black,
            level_warn: rgb(180, 120, 0),
            level_error: rgb(200, 40, 40),
            focus_fg: Color::Black,
            focus_bg: rgb(255, 204, 0),
            selection_fg: Color::White,
            selection_bg: rgb(0, 102, 204),
            find_fg: Color::Black,
            find_bg: rgb(255, 204, 0),
        }
    }

    pub fn resolve(pref: ThemePreference) -> Self {
        match pref {
            ThemePreference::Dark => Self::dark(),
            ThemePreference::Light => Self::light(),
            ThemePreference::Auto => {
                if detect_light_background() {
                    Self::light()
                } else {
                    Self::dark()
                }
            }
        }
    }

    pub fn level_color(&self, level: LogLevel) -> Color {
        match level {
            LogLevel::Verbose => self.level_verbose,
            LogLevel::Debug => self.level_debug,
            LogLevel::Info => self.level_info,
            LogLevel::Warn => self.level_warn,
            LogLevel::Error | LogLevel::Fatal => self.level_error,
        }
    }
}

fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb(r, g, b)
}

/// Heuristic from `$COLORFGBG` (fg;bg). Light when background index >= 7.
/// Windows terminals often omit `$COLORFGBG`; assume light there.
fn detect_light_background() -> bool {
    if let Some(bg) = std::env::var("COLORFGBG")
        .ok()
        .and_then(|value| value.rsplit(';').next()?.parse::<u8>().ok())
    {
        return bg >= 7;
    }
    cfg!(windows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dark_and_light_info_differ() {
        assert_ne!(Theme::dark().level_info, Theme::light().level_info);
    }

    #[test]
    fn theme_preference_cycles() {
        assert_eq!(
            ThemePreference::Auto.cycle(true),
            ThemePreference::Dark
        );
        assert_eq!(
            ThemePreference::Dark.cycle(false),
            ThemePreference::Auto
        );
    }

    #[test]
    fn auto_resolves_to_light_on_windows_without_colorfgbg() {
        if std::env::var("COLORFGBG").is_ok() {
            return;
        }
        let theme = Theme::resolve(ThemePreference::Auto);
        if cfg!(windows) {
            assert_eq!(theme.shell_fg, Color::Black);
            assert_eq!(theme.level_info, Color::Black);
        } else {
            assert_eq!(theme.shell_fg, Theme::dark().shell_fg);
        }
    }
}

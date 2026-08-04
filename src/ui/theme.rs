use crate::parser::LogLevel;
use ratatui::style::Color;

/// Fixed semantic accent palette. Shell chrome inherits the terminal default;
/// only levels, focus, selection, and find use these colors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
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
    /// Single fixed accent palette (named/ANSI colors that follow the host table).
    pub fn accents() -> Self {
        Self {
            level_verbose: Color::DarkGray,
            level_debug: Color::Cyan,
            level_info: Color::Gray,
            level_warn: Color::Yellow,
            level_error: Color::Red,
            focus_fg: Color::Black,
            focus_bg: Color::Yellow,
            selection_fg: Color::White,
            selection_bg: Color::Blue,
            find_fg: Color::Black,
            find_bg: Color::Yellow,
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

impl Default for Theme {
    fn default() -> Self {
        Self::accents()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accents_are_stable_default() {
        let theme = Theme::default();
        assert_eq!(theme, Theme::accents());
        assert_eq!(theme.level_error, Color::Red);
        assert_eq!(theme.level_warn, Color::Yellow);
        assert_ne!(theme.level_error, theme.level_info);
        assert_ne!(theme.find_bg, theme.selection_bg);
    }

    #[test]
    fn level_color_maps_error_and_fatal() {
        let theme = Theme::accents();
        assert_eq!(theme.level_color(LogLevel::Error), theme.level_error);
        assert_eq!(theme.level_color(LogLevel::Fatal), theme.level_error);
        assert_eq!(theme.level_color(LogLevel::Info), theme.level_info);
    }
}

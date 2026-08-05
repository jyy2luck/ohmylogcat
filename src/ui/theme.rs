use crate::parser::LogLevel;
use ratatui::style::Color;

/// Semantic accent palette. Shell chrome inherits the terminal default;
/// only levels, focus, selection, and find use these colors.
/// Level colors come from Android Studio → Color Scheme → Android Logcat;
/// light vs dark is chosen once at startup via host-background detection.
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
    /// Android Studio Logcat light scheme level colors (HEX → RGB).
    pub fn light_accents() -> Self {
        Self {
            level_verbose: rgb(0, 0, 0),             // #000000
            level_debug: rgb(56, 159, 214),          // #389FD6
            level_info: rgb(89, 168, 105),           // #59A869
            level_warn: rgb(100, 86, 7),             // #645607
            level_error: rgb(205, 0, 0),             // #CD0000
            ..Self::interaction_accents()
        }
    }

    /// Android Studio Logcat dark scheme level colors (HEX → RGB).
    pub fn dark_accents() -> Self {
        Self {
            level_verbose: rgb(187, 187, 187),       // #BBBBBB
            level_debug: rgb(41, 153, 153),          // #299999
            level_info: rgb(171, 192, 35),           // #ABC023
            level_warn: rgb(187, 181, 41),           // #BBB529
            level_error: rgb(255, 107, 104),         // #FF6B68
            ..Self::interaction_accents()
        }
    }

    /// Pick light or dark accents from `$COLORFGBG`; dark when unset/unparsable.
    pub fn resolve() -> Self {
        if detect_light_background() {
            Self::light_accents()
        } else {
            Self::dark_accents()
        }
    }

    fn interaction_accents() -> Self {
        Self {
            level_verbose: Color::Reset,
            level_debug: Color::Reset,
            level_info: Color::Reset,
            level_warn: Color::Reset,
            level_error: Color::Reset,
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
        Self::resolve()
    }
}

fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb(r, g, b)
}

/// Heuristic from `$COLORFGBG` (fg;bg). Light when background index >= 7.
/// When unset/unparsable, assume dark.
fn detect_light_background() -> bool {
    detect_light_background_from(std::env::var("COLORFGBG").ok().as_deref())
}

fn detect_light_background_from(colorfgbg: Option<&str>) -> bool {
    if let Some(bg) = colorfgbg.and_then(|value| value.rsplit(';').next()?.parse::<u8>().ok()) {
        return bg >= 7;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_and_dark_level_rgb_match_as_logcat() {
        let light = Theme::light_accents();
        assert_eq!(light.level_verbose, Color::Rgb(0, 0, 0)); // #000000
        assert_eq!(light.level_debug, Color::Rgb(56, 159, 214)); // #389FD6
        assert_eq!(light.level_info, Color::Rgb(89, 168, 105)); // #59A869
        assert_eq!(light.level_warn, Color::Rgb(100, 86, 7)); // #645607
        assert_eq!(light.level_error, Color::Rgb(205, 0, 0)); // #CD0000

        let dark = Theme::dark_accents();
        assert_eq!(dark.level_verbose, Color::Rgb(187, 187, 187)); // #BBBBBB
        assert_eq!(dark.level_debug, Color::Rgb(41, 153, 153)); // #299999
        assert_eq!(dark.level_info, Color::Rgb(171, 192, 35)); // #ABC023
        assert_eq!(dark.level_warn, Color::Rgb(187, 181, 41)); // #BBB529
        assert_eq!(dark.level_error, Color::Rgb(255, 107, 104)); // #FF6B68

        assert_ne!(dark.level_info, light.level_info);
        assert_ne!(dark.level_error, dark.level_info);
        assert_ne!(light.level_error, light.level_info);
        assert_eq!(dark.find_bg, light.find_bg);
        assert_eq!(dark.find_fg, Color::Black);
        assert_eq!(dark.find_bg, Color::Yellow);
        assert_ne!(dark.find_bg, dark.selection_bg);
    }

    #[test]
    fn resolve_without_colorfgbg_yields_dark() {
        if std::env::var("COLORFGBG").is_ok() {
            assert!(!detect_light_background_from(None));
            assert_eq!(
                if detect_light_background_from(None) {
                    Theme::light_accents()
                } else {
                    Theme::dark_accents()
                }
                .level_info,
                Theme::dark_accents().level_info
            );
            return;
        }
        let theme = Theme::resolve();
        assert_eq!(theme.level_info, Theme::dark_accents().level_info);
        assert_eq!(theme, Theme::dark_accents());
    }

    #[test]
    fn light_colorfgbg_selects_light_accents() {
        assert!(detect_light_background_from(Some("15;15")));
        assert!(detect_light_background_from(Some("0;7")));
        assert!(!detect_light_background_from(Some("15;0")));
        assert!(!detect_light_background_from(Some("not-a-number")));
        assert!(!detect_light_background_from(None));

        let theme = if detect_light_background_from(Some("15;15")) {
            Theme::light_accents()
        } else {
            Theme::dark_accents()
        };
        assert_eq!(theme.level_info, Theme::light_accents().level_info);
        assert_eq!(theme.level_info, Color::Rgb(89, 168, 105)); // #59A869
    }

    #[test]
    fn level_color_maps_error_and_fatal() {
        let theme = Theme::dark_accents();
        assert_eq!(theme.level_color(LogLevel::Error), theme.level_error);
        assert_eq!(theme.level_color(LogLevel::Fatal), theme.level_error);
        assert_eq!(theme.level_color(LogLevel::Info), theme.level_info);
    }
}

//! Shared single-line text input with char-boundary cursor.

use crossterm::cursor::SetCursorStyle;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Terminal insertion cursor style while a text field is focused.
/// Switch to `SteadyBar` if a terminal blinks incorrectly.
pub const TEXT_INPUT_CURSOR_STYLE: SetCursorStyle = SetCursorStyle::BlinkingBar;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextInput {
    pub text: String,
    pub cursor: usize,
}

impl TextInput {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_text(text: String) -> Self {
        let cursor = text.chars().count();
        Self { text, cursor }
    }

    pub fn set_cursor_end(&mut self) {
        self.cursor = self.text.chars().count();
    }

    /// Handle editing keys. Returns `true` when `text` changed.
    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Left => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
                false
            }
            KeyCode::Right => {
                let len = self.text.chars().count();
                if self.cursor < len {
                    self.cursor += 1;
                }
                false
            }
            KeyCode::Home => {
                self.cursor = 0;
                false
            }
            KeyCode::End => {
                self.set_cursor_end();
                false
            }
            KeyCode::Backspace => {
                if self.cursor == 0 {
                    return false;
                }
                let byte_idx = self.nth_char_byte_index(self.cursor - 1);
                let next_byte = self
                    .text
                    .char_indices()
                    .nth(self.cursor)
                    .map(|(i, _)| i)
                    .unwrap_or(self.text.len());
                self.text.replace_range(byte_idx..next_byte, "");
                self.cursor -= 1;
                true
            }
            KeyCode::Delete => {
                let len = self.text.chars().count();
                if self.cursor >= len {
                    return false;
                }
                let byte_idx = self.nth_char_byte_index(self.cursor);
                let next_byte = self
                    .text
                    .char_indices()
                    .nth(self.cursor + 1)
                    .map(|(i, _)| i)
                    .unwrap_or(self.text.len());
                self.text.replace_range(byte_idx..next_byte, "");
                true
            }
            KeyCode::Char(c)
                if !key.modifiers.contains(KeyModifiers::CONTROL)
                    && !key.modifiers.contains(KeyModifiers::SUPER) =>
            {
                let byte_idx = self.nth_char_byte_index(self.cursor);
                self.text.insert(byte_idx, c);
                self.cursor += 1;
                true
            }
            _ => false,
        }
    }

    /// Map a terminal column click to a char index into `text`.
    pub fn cursor_from_click(click_col: u16, value_start_col: u16, text: &str) -> usize {
        let offset = click_col.saturating_sub(value_start_col) as usize;
        offset.min(text.chars().count())
    }

    /// Display width (monospace char count) from start of string to cursor.
    pub fn display_width_before_cursor(&self) -> u16 {
        self.cursor as u16
    }

    fn nth_char_byte_index(&self, char_idx: usize) -> usize {
        self.text
            .char_indices()
            .nth(char_idx)
            .map(|(i, _)| i)
            .unwrap_or(self.text.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    fn char_key(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::empty())
    }

    #[test]
    fn insert_at_cursor() {
        let mut input = TextInput::from_text("ab".into());
        input.cursor = 1;
        assert!(input.handle_key(char_key('X')));
        assert_eq!(input.text, "aXb");
        assert_eq!(input.cursor, 2);
    }

    #[test]
    fn backspace_at_cursor() {
        let mut input = TextInput::from_text("abc".into());
        input.cursor = 2;
        assert!(input.handle_key(key(KeyCode::Backspace)));
        assert_eq!(input.text, "ac");
        assert_eq!(input.cursor, 1);
    }

    #[test]
    fn delete_at_cursor() {
        let mut input = TextInput::from_text("abc".into());
        input.cursor = 1;
        assert!(input.handle_key(key(KeyCode::Delete)));
        assert_eq!(input.text, "ac");
        assert_eq!(input.cursor, 1);
    }

    #[test]
    fn arrow_and_home_end() {
        let mut input = TextInput::from_text("abc".into());
        input.handle_key(key(KeyCode::Home));
        assert_eq!(input.cursor, 0);
        input.handle_key(key(KeyCode::Right));
        assert_eq!(input.cursor, 1);
        input.handle_key(key(KeyCode::End));
        assert_eq!(input.cursor, 3);
    }

    #[test]
    fn cursor_from_click_clamps() {
        assert_eq!(TextInput::cursor_from_click(12, 10, "hi"), 2);
        assert_eq!(TextInput::cursor_from_click(11, 10, "hi"), 1);
        assert_eq!(TextInput::cursor_from_click(99, 10, "hi"), 2);
    }
}

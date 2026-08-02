//! Shared single-line text input with char-boundary cursor.

use crossterm::cursor::SetCursorStyle;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use unicode_width::UnicodeWidthStr;

/// Terminal insertion cursor style while a text field is focused.
/// Switch to `SteadyBar` if a terminal blinks incorrectly.
pub const TEXT_INPUT_CURSOR_STYLE: SetCursorStyle = SetCursorStyle::BlinkingBar;

/// Terminal display width of `s` (CJK / fullwidth count as 2).
pub fn str_display_width(s: &str) -> u16 {
    s.width() as u16
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextInput {
    pub text: String,
    pub cursor: usize,
}

impl TextInput {
    pub fn new() -> Self {
        Self::default()
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
        let target = click_col.saturating_sub(value_start_col) as usize;
        let mut width = 0usize;
        for (i, ch) in text.chars().enumerate() {
            if width >= target {
                return i;
            }
            // Isolate each char so width() accounts for East Asian Width.
            let mut buf = [0u8; 4];
            width += ch.encode_utf8(&mut buf).width();
        }
        text.chars().count()
    }

    /// Display width (terminal columns) from start of string to cursor.
    pub fn display_width_before_cursor(&self) -> u16 {
        let end = self.nth_char_byte_index(self.cursor);
        str_display_width(&self.text[..end])
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

    fn from_text(text: String) -> TextInput {
        let cursor = text.chars().count();
        TextInput { text, cursor }
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    fn char_key(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::empty())
    }

    #[test]
    fn insert_at_cursor() {
        let mut input = from_text("ab".into());
        input.cursor = 1;
        assert!(input.handle_key(char_key('X')));
        assert_eq!(input.text, "aXb");
        assert_eq!(input.cursor, 2);
    }

    #[test]
    fn backspace_at_cursor() {
        let mut input = from_text("abc".into());
        input.cursor = 2;
        assert!(input.handle_key(key(KeyCode::Backspace)));
        assert_eq!(input.text, "ac");
        assert_eq!(input.cursor, 1);
    }

    #[test]
    fn delete_at_cursor() {
        let mut input = from_text("abc".into());
        input.cursor = 1;
        assert!(input.handle_key(key(KeyCode::Delete)));
        assert_eq!(input.text, "ac");
        assert_eq!(input.cursor, 1);
    }

    #[test]
    fn arrow_and_home_end() {
        let mut input = from_text("abc".into());
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

    #[test]
    fn display_width_counts_cjk_as_two() {
        let mut input = from_text("标签".into());
        input.cursor = 1;
        assert_eq!(input.display_width_before_cursor(), 2);
        input.cursor = 2;
        assert_eq!(input.display_width_before_cursor(), 4);
        assert_eq!(str_display_width("消息包含: ["), 11); // 4*2 + 3 ascii = 11
    }

    #[test]
    fn cursor_from_click_respects_cjk_width() {
        // "中" is 2 cols; click at col+1 (mid glyph) → after the char
        assert_eq!(TextInput::cursor_from_click(11, 10, "中a"), 1);
        assert_eq!(TextInput::cursor_from_click(12, 10, "中a"), 1);
        assert_eq!(TextInput::cursor_from_click(13, 10, "中a"), 2);
    }
}

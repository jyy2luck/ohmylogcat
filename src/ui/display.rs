//! Display helpers for long log lines (lazy wrap, visible slices).

/// Number of terminal rows needed to show `s` at `width` columns.
pub fn wrap_line_count(s: &str, width: usize) -> usize {
    let width = width.max(1);
    let chars = s.chars().count();
    if chars == 0 {
        1
    } else {
        chars.div_ceil(width)
    }
}

/// Lazily yield wrapped chunks without materializing the full line upfront.
pub struct WrapChunks<'a> {
    chars: std::str::Chars<'a>,
    width: usize,
    done: bool,
}

impl<'a> WrapChunks<'a> {
    pub fn new(s: &'a str, width: usize) -> Self {
        Self {
            chars: s.chars(),
            width: width.max(1),
            done: s.is_empty(),
        }
    }
}

impl Iterator for WrapChunks<'_> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let chunk: String = self.chars.by_ref().take(self.width).collect();
        if chunk.is_empty() {
            None
        } else {
            Some(chunk)
        }
    }
}

/// Visible slice: skip `col_offset` chars, take at most `max_width` chars.
pub fn visible_chars(s: &str, col_offset: usize, max_width: usize) -> String {
    s.chars()
        .skip(col_offset)
        .take(max_width)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test-local mirror of the scroll-bottom algorithm used by `App::scroll_to_bottom`.
    fn scroll_bottom_position(heights: &[usize], viewport_height: usize) -> (usize, usize) {
        let n = heights.len();
        if n == 0 {
            return (0, 0);
        }
        let height = viewport_height.max(1);
        let mut idx = n - 1;
        let mut lines_after = heights[idx].max(1);
        while lines_after < height && idx > 0 {
            idx -= 1;
            lines_after += heights[idx].max(1);
        }
        (idx, lines_after.saturating_sub(height))
    }

    #[test]
    fn lazy_wrap_matches_count() {
        let s = "hello world";
        let width = 5;
        let chunks: Vec<_> = WrapChunks::new(s, width).collect();
        assert_eq!(chunks, vec!["hello", " worl", "d"]);
        assert_eq!(wrap_line_count(s, width), chunks.len());
    }

    #[test]
    fn visible_chars_skips_and_truncates() {
        assert_eq!(visible_chars("abcdef", 2, 3), "cde");
    }

    #[test]
    fn scroll_bottom_shows_new_entry_after_huge_line() {
        // One 100-line entry then a new 1-line entry; viewport fits 30 lines.
        let (offset, skip) = scroll_bottom_position(&[100, 1], 30);
        assert_eq!(offset, 0);
        assert_eq!(skip, 71);
        let (offset, skip) = scroll_bottom_position(&[100, 1, 1], 30);
        assert_eq!(offset, 0);
        assert_eq!(skip, 72);
    }

    #[test]
    fn scroll_bottom_single_short_entry() {
        let (offset, skip) = scroll_bottom_position(&[5], 30);
        assert_eq!(offset, 0);
        assert_eq!(skip, 0);
    }

    #[test]
    fn scroll_bottom_uses_viewport_height_for_huge_line() {
        // Same 100-line entry: default height 10 vs real height 40 must differ.
        let (offset_small, skip_small) = scroll_bottom_position(&[100], 10);
        let (offset_large, skip_large) = scroll_bottom_position(&[100], 40);
        assert_eq!(offset_small, 0);
        assert_eq!(skip_small, 90);
        assert_eq!(offset_large, 0);
        assert_eq!(skip_large, 60);
        assert_ne!(skip_small, skip_large);
    }
}

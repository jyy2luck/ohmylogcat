//! Display helpers for long log lines (lazy wrap, visible slices).

pub fn effective_hang_indent(indent: usize, width: usize) -> usize {
    if indent == 0 || indent >= width {
        0
    } else {
        indent
    }
}

fn chunk_capacity(width: usize, hang: usize, first: bool) -> usize {
    if hang == 0 {
        width
    } else if first {
        width
    } else {
        width - hang
    }
}

/// Number of terminal rows needed to show `s` at `width` with optional hang indent.
pub fn wrap_line_count(s: &str, width: usize, indent: usize) -> usize {
    let width = width.max(1);
    let hang = effective_hang_indent(indent, width);
    let chars = s.chars().count();
    if chars == 0 {
        return 1;
    }
    if hang == 0 {
        chars.div_ceil(width)
    } else if chars <= width {
        1
    } else {
        let cont_width = width - hang;
        1 + (chars - width).div_ceil(cont_width)
    }
}

/// Lazily yield wrapped chunks as `(logical_char_start, chunk_text)`.
pub struct WrapChunks<'a> {
    chars: std::str::Chars<'a>,
    width: usize,
    hang: usize,
    logical_start: usize,
    first: bool,
    done: bool,
}

impl<'a> WrapChunks<'a> {
    pub fn with_indent(s: &'a str, width: usize, indent: usize) -> Self {
        let width = width.max(1);
        Self {
            chars: s.chars(),
            width,
            hang: effective_hang_indent(indent, width),
            logical_start: 0,
            first: true,
            done: s.is_empty(),
        }
    }
}

impl Iterator for WrapChunks<'_> {
    type Item = (usize, String);

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let cap = chunk_capacity(self.width, self.hang, self.first);
        let chunk: String = self.chars.by_ref().take(cap).collect();
        if chunk.is_empty() {
            None
        } else {
            let start = self.logical_start;
            let len = chunk.chars().count();
            self.logical_start += len;
            self.first = false;
            Some((start, chunk))
        }
    }
}

/// Display text for a wrap chunk (adds hang-indent pad on continuation rows).
pub fn wrap_display_text(chunk: &str, logical_start: usize, indent: usize, width: usize) -> String {
    let hang = effective_hang_indent(indent, width);
    if logical_start == 0 || hang == 0 {
        chunk.to_string()
    } else {
        format!("{}{}", " ".repeat(hang), chunk)
    }
}

/// Which wrap display row (0-based within entry) contains logical column `col`.
pub fn wrap_display_row_for_col(s: &str, width: usize, indent: usize, col: usize) -> usize {
    if s.is_empty() {
        return 0;
    }
    for (i, (start, chunk)) in WrapChunks::with_indent(s, width, indent).enumerate() {
        let len = chunk.chars().count();
        if len == 0 {
            continue;
        }
        if col >= start && col < start + len {
            return i;
        }
    }
    wrap_line_count(s, width, indent).saturating_sub(1)
}

/// Info about the wrap chunk containing logical column `col`: (row_index, chunk_start, chunk_len).
pub fn wrap_chunk_at_col(s: &str, width: usize, indent: usize, col: usize) -> (usize, usize, usize) {
    if s.is_empty() {
        return (0, 0, 0);
    }
    let mut last = (0usize, 0usize, 0usize);
    for (i, (start, chunk)) in WrapChunks::with_indent(s, width, indent).enumerate() {
        let len = chunk.chars().count();
        last = (i, start, len);
        if len == 0 {
            continue;
        }
        if col >= start && col < start + len {
            return (i, start, len);
        }
    }
    last
}

/// Info for wrap chunk at display row `index`: (row_index, chunk_start, chunk_len).
pub fn wrap_chunk_by_index(s: &str, width: usize, indent: usize, index: usize) -> (usize, usize, usize) {
    for (i, (start, chunk)) in WrapChunks::with_indent(s, width, indent).enumerate() {
        if i == index {
            let len = chunk.chars().count();
            return (i, start, len);
        }
    }
    (0, 0, 0)
}

/// Screen column within the wrap display row that contains logical column `col`.
pub fn wrap_display_col(s: &str, width: usize, indent: usize, col: usize) -> usize {
    let (_, chunk_start, _) = wrap_chunk_at_col(s, width, indent, col);
    let hang = effective_hang_indent(indent, width);
    if chunk_start > 0 && hang > 0 {
        hang + col.saturating_sub(chunk_start)
    } else {
        col.saturating_sub(chunk_start)
    }
}

/// Logical column for a screen column on wrap display row `chunk_index`.
pub fn wrap_logical_col_from_display(
    s: &str,
    width: usize,
    indent: usize,
    chunk_index: usize,
    display_col: usize,
) -> usize {
    let (_, chunk_start, chunk_len) = wrap_chunk_by_index(s, width, indent, chunk_index);
    if chunk_len == 0 {
        return chunk_start;
    }
    let hang = effective_hang_indent(indent, width);
    let offset_in_chunk = if chunk_start > 0 && hang > 0 {
        display_col.saturating_sub(hang)
    } else {
        display_col
    };
    chunk_start + offset_in_chunk.min(chunk_len - 1)
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
        let chunks: Vec<_> = WrapChunks::with_indent(s, width, 0).map(|(_, c)| c).collect();
        assert_eq!(chunks, vec!["hello", " worl", "d"]);
        assert_eq!(wrap_line_count(s, width, 0), chunks.len());
    }

    #[test]
    fn hang_indent_wrap_row_count() {
        let s = "0123456789abcdefghij";
        let width = 10;
        let indent = 5;
        assert_eq!(wrap_line_count(s, width, indent), 3);
    }

    #[test]
    fn hang_indent_first_vs_continuation_capacity() {
        let s = "0123456789abcdefghij";
        let width = 10;
        let indent = 5;
        let chunks: Vec<_> = WrapChunks::with_indent(s, width, indent).collect();
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0], (0, "0123456789".to_string()));
        assert_eq!(chunks[1], (10, "abcde".to_string()));
        assert_eq!(chunks[2], (15, "fghij".to_string()));
    }

    #[test]
    fn hang_indent_narrow_viewport_fallback() {
        let s = "0123456789";
        let width = 8;
        let indent = 10;
        assert_eq!(effective_hang_indent(indent, width), 0);
        assert_eq!(wrap_line_count(s, width, indent), 2);
        let chunks: Vec<_> = WrapChunks::with_indent(s, width, indent).collect();
        assert_eq!(chunks[0], (0, "01234567".to_string()));
        assert_eq!(chunks[1], (8, "89".to_string()));
    }

    #[test]
    fn wrap_display_text_adds_pad_on_continuation() {
        assert_eq!(wrap_display_text("abc", 10, 5, 20), "     abc");
        assert_eq!(wrap_display_text("abc", 0, 5, 20), "abc");
    }

    #[test]
    fn wrap_display_col_roundtrip_with_hang_indent() {
        let s = "01234: 5678901234abcd";
        let width = 10;
        let indent = 7;
        for col in [0, 8, 10, 12, 15] {
            let (chunk, _, _) = wrap_chunk_at_col(s, width, indent, col);
            let display = wrap_display_col(s, width, indent, col);
            let back = wrap_logical_col_from_display(s, width, indent, chunk, display);
            assert_eq!(back, col, "col {col} display {display} chunk {chunk}");
        }
    }

    #[test]
    fn wrap_display_col_vertical_column_stable() {
        let s = "01234: 5678901234abcd";
        let width = 10;
        let indent = 7;
        // Logical cols 8 and 11 both sit at display column 8 on their respective rows.
        assert_eq!(wrap_display_col(s, width, indent, 8), 8);
        assert_eq!(wrap_display_col(s, width, indent, 11), 8);
    }

    #[test]
    fn visible_chars_skips_and_truncates() {
        assert_eq!(visible_chars("abcdef", 2, 3), "cde");
    }

    #[test]
    fn scroll_bottom_shows_new_entry_after_huge_line() {
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
        let (offset_small, skip_small) = scroll_bottom_position(&[100], 10);
        let (offset_large, skip_large) = scroll_bottom_position(&[100], 40);
        assert_eq!(offset_small, 0);
        assert_eq!(skip_small, 90);
        assert_eq!(offset_large, 0);
        assert_eq!(skip_large, 60);
        assert_ne!(skip_small, skip_large);
    }
}

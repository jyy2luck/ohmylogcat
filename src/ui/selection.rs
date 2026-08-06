//! Mouse text selection and caret geometry in the log viewport.

use crate::ui::display::{effective_hang_indent, WrapChunks};
use crate::ui::format::message_column_indent_line;
use crate::ui::Theme;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;

/// Character position in the filtered log list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LogPos {
    pub row: usize,
    pub col: usize,
}

#[derive(Debug, Default, Clone)]
pub struct TextSelection {
    anchor: Option<LogPos>,
    cursor: Option<LogPos>,
    dragging: bool,
}

impl TextSelection {
    pub fn clear(&mut self) {
        self.anchor = None;
        self.cursor = None;
        self.dragging = false;
    }

    pub fn is_active(&self) -> bool {
        self.anchor.is_some() && self.cursor.is_some()
    }

    pub fn dragging(&self) -> bool {
        self.dragging
    }

    pub fn start(&mut self, pos: LogPos) {
        self.anchor = Some(pos);
        self.cursor = Some(pos);
        self.dragging = true;
    }

    pub fn extend_to(&mut self, pos: LogPos) {
        self.cursor = Some(pos);
    }

    /// Set a non-dragging selection range (word/line click, keyboard Shift).
    pub fn set_range(&mut self, anchor: LogPos, cursor: LogPos) {
        self.anchor = Some(anchor);
        self.cursor = Some(cursor);
        self.dragging = false;
    }

    pub fn finish_drag(&mut self) {
        self.dragging = false;
    }

    /// Half-open range `[start, end)` ordered by gap order.
    /// `end` is exclusive: a selection ending at `col == line_len` includes the
    /// last character of that line.
    pub fn normalized_range(&self) -> Option<(LogPos, LogPos)> {
        match (self.anchor, self.cursor) {
            (Some(a), Some(c)) if a <= c => Some((a, c)),
            (Some(a), Some(c)) => Some((c, a)),
            _ => None,
        }
    }

    pub fn has_extent(&self) -> bool {
        match (self.anchor, self.cursor) {
            (Some(a), Some(c)) => a != c,
            _ => false,
        }
    }

    pub fn contains(&self, row: usize, col: usize) -> bool {
        let Some((start, end)) = self.normalized_range() else {
            return false;
        };
        let pos = LogPos { row, col };
        pos >= start && pos < end
    }

    /// Extract selected plain text from formatted log lines using half-open
    /// endpoints `[start, end)`. A selection whose end gap is `col == line_len`
    /// includes the last character of that line.
    pub fn extract_text(&self, line_at: impl Fn(usize) -> Option<String>) -> Option<String> {
        let (start, end) = self.normalized_range()?;
        let mut out = String::new();
        for row in start.row..=end.row {
            let line = line_at(row)?;
            let chars: Vec<char> = line.chars().collect();
            if chars.is_empty() {
                if row < end.row {
                    out.push('\n');
                }
                continue;
            }
            let from = if row == start.row {
                start.col.min(chars.len())
            } else {
                0
            };
            let to = if row == end.row {
                end.col.min(chars.len())
            } else {
                chars.len()
            };
            if row > start.row {
                out.push('\n');
            }
            if from < to {
                out.extend(chars[from..to].iter());
            }
        }
        if out.is_empty() {
            None
        } else {
            Some(out)
        }
    }
}

/// Viewport state needed to map screen coordinates to log positions.
pub struct ViewportMap {
    pub area: Rect,
    pub scroll_offset: usize,
    pub wrap_skip: usize,
    pub col_offset: usize,
    pub soft_wrap: bool,
    pub viewport_width: usize,
    pub viewport_height: usize,
}

pub fn mouse_to_log_pos(
    col: u16,
    row: u16,
    map: &ViewportMap,
    line_at: impl Fn(usize) -> Option<String>,
    row_count: usize,
) -> Option<LogPos> {
    if row_count == 0 || !contains(map.area, col, row) {
        return None;
    }

    if map.soft_wrap {
        mouse_to_log_pos_wrapped(col, row, map, line_at, row_count)
    } else {
        mouse_to_log_pos_nowrap(col, row, map, line_at, row_count)
    }
}

fn mouse_to_log_pos_nowrap(
    col: u16,
    row: u16,
    map: &ViewportMap,
    line_at: impl Fn(usize) -> Option<String>,
    row_count: usize,
) -> Option<LogPos> {
    let display_row = (row - map.area.y) as usize;
    if display_row >= map.viewport_height {
        return None;
    }
    let log_row = map.scroll_offset + display_row;
    if log_row >= row_count {
        return None;
    }
    let line = line_at(log_row)?;
    let line_len = line.chars().count();
    if line_len == 0 {
        return Some(LogPos { row: log_row, col: 0 });
    }
    let col_in = (col.saturating_sub(map.area.x)) as usize;
    let target = map.col_offset + col_in;
    // Click within a cell maps to that cell's left gap; clicking past the last
    // character maps to the line-end gap (col == line_len).
    let char_col = if target >= line_len { line_len } else { target };
    Some(LogPos {
        row: log_row,
        col: char_col,
    })
}

fn mouse_to_log_pos_wrapped(
    col: u16,
    row: u16,
    map: &ViewportMap,
    line_at: impl Fn(usize) -> Option<String>,
    row_count: usize,
) -> Option<LogPos> {
    let target_display = (row - map.area.y) as usize;
    let width = map.viewport_width.max(1);
    let col_in = (col.saturating_sub(map.area.x)) as usize;

    let mut display_row = 0usize;
    let mut idx = map.scroll_offset;
    let mut skip = map.wrap_skip;

    while idx < row_count {
        let line = line_at(idx)?;
        let indent = message_column_indent_line(&line);
        let hang = effective_hang_indent(indent, width);
        for (chunk_start, chunk) in WrapChunks::with_indent(&line, width, indent) {
            if skip > 0 {
                skip -= 1;
                continue;
            }
            if display_row == target_display {
                let chunk_len = chunk.chars().count();
                let is_continuation = chunk_start > 0 && hang > 0;
                if is_continuation && col_in < hang {
                    return Some(LogPos {
                        row: idx,
                        col: chunk_start,
                    });
                }
                let content_col = if is_continuation {
                    col_in.saturating_sub(hang)
                } else {
                    col_in
                };
                if chunk_len == 0 {
                    return Some(LogPos { row: idx, col: chunk_start });
                }
                let total = line.chars().count();
                let is_final_chunk = chunk_start + chunk_len == total;
                // Click within a cell maps to that cell's left gap; past the last
                // char of the final chunk maps to the line-end gap.
                let col_in_chunk = if content_col >= chunk_len {
                    if is_final_chunk {
                        chunk_len
                    } else {
                        chunk_len - 1
                    }
                } else {
                    content_col
                };
                return Some(LogPos {
                    row: idx,
                    col: chunk_start + col_in_chunk,
                });
            }
            display_row += 1;
            if display_row > target_display {
                break;
            }
        }
        idx += 1;
    }
    None
}

/// Map a log caret position to a screen cell. Returns `None` when off-screen.
pub fn log_pos_to_screen(
    pos: LogPos,
    map: &ViewportMap,
    line_at: impl Fn(usize) -> Option<String>,
    row_count: usize,
) -> Option<(u16, u16)> {
    if row_count == 0 || pos.row >= row_count {
        return None;
    }
    if map.soft_wrap {
        log_pos_to_screen_wrapped(pos, map, line_at, row_count)
    } else {
        log_pos_to_screen_nowrap(pos, map, line_at)
    }
}

fn log_pos_to_screen_nowrap(
    pos: LogPos,
    map: &ViewportMap,
    line_at: impl Fn(usize) -> Option<String>,
) -> Option<(u16, u16)> {
    if pos.row < map.scroll_offset {
        return None;
    }
    let display_row = pos.row - map.scroll_offset;
    if display_row >= map.viewport_height {
        return None;
    }
    let line = line_at(pos.row)?;
    let line_len = line.chars().count();
    let col = pos.col.min(line_len);
    if col < map.col_offset {
        return None;
    }
    let col_in = col - map.col_offset;
    let width = map.viewport_width.max(1);
    // A real character (col < line_len) is visible only when col_in < width.
    // The line-end gap (col == line_len) may sit at col_in == width (right edge
    // of the last cell when the line fills the viewport); terminals allow a
    // cursor column equal to the viewport width.
    if col_in > width {
        return None;
    }
    if col_in == width && col != line_len {
        return None;
    }
    let x = map.area.x.saturating_add(col_in as u16);
    let y = map.area.y.saturating_add(display_row as u16);
    Some((x, y))
}

fn log_pos_to_screen_wrapped(
    pos: LogPos,
    map: &ViewportMap,
    line_at: impl Fn(usize) -> Option<String>,
    row_count: usize,
) -> Option<(u16, u16)> {
    let width = map.viewport_width.max(1);
    let mut display_row = 0usize;
    let mut idx = map.scroll_offset;
    let mut skip = map.wrap_skip;

    while idx < row_count && display_row < map.viewport_height {
        let line = line_at(idx)?;
        let line_len = line.chars().count();
        if line_len == 0 {
            if skip > 0 {
                skip -= 1;
            } else {
                if idx == pos.row {
                    let x = map.area.x;
                    let y = map.area.y.saturating_add(display_row as u16);
                    return Some((x, y));
                }
                display_row += 1;
            }
            idx += 1;
            continue;
        }
        let indent = message_column_indent_line(&line);
        let hang = effective_hang_indent(indent, width);
        for (chunk_start, chunk) in WrapChunks::with_indent(&line, width, indent) {
            if skip > 0 {
                skip -= 1;
                continue;
            }
            let chunk_chars = chunk.chars().count();
            let chunk_end = chunk_start + chunk_chars;
            // The line-end gap (col == line_len) belongs to the final chunk.
            let is_final_chunk = chunk_end == line_len;
            let in_chunk = if is_final_chunk {
                pos.col >= chunk_start && pos.col <= chunk_end
            } else {
                pos.col >= chunk_start && pos.col < chunk_end
            };
            if idx == pos.row && in_chunk {
                let is_continuation = chunk_start > 0 && hang > 0;
                let col_in = (pos.col - chunk_start).min(chunk_chars);
                let cap = if is_continuation {
                    width - hang
                } else {
                    width
                };
                // Line-end gap on a chunk that fills the available width sits at
                // col_in == cap (right edge); allow it.
                if col_in > cap {
                    return None;
                }
                if col_in == cap && pos.col != chunk_end {
                    return None;
                }
                let screen_x_offset = if is_continuation {
                    hang + col_in
                } else {
                    col_in
                };
                let x = map.area.x.saturating_add(screen_x_offset as u16);
                let y = map.area.y.saturating_add(display_row as u16);
                return Some((x, y));
            }
            display_row += 1;
            if display_row >= map.viewport_height {
                return None;
            }
        }
        idx += 1;
    }
    None
}

/// Word characters for double-click expand: ASCII letters, digits, underscore.
pub fn is_word_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Half-open `[start, end_exclusive)` columns for the word (or single non-word
/// char) at `col`. An empty line returns `(0, 0)` (zero-width selection).
pub fn expand_word(line: &str, col: usize) -> (usize, usize) {
    let chars: Vec<char> = line.chars().collect();
    if chars.is_empty() {
        return (0, 0);
    }
    let col = col.min(chars.len() - 1);
    if !is_word_char(chars[col]) {
        return (col, col + 1);
    }
    let mut start = col;
    while start > 0 && is_word_char(chars[start - 1]) {
        start -= 1;
    }
    let mut end = col;
    while end + 1 < chars.len() && is_word_char(chars[end + 1]) {
        end += 1;
    }
    (start, end + 1)
}

/// Half-open columns covering the whole logical line: `(0, line_len)`.
/// An empty line returns `(0, 0)`.
pub fn expand_line(line: &str) -> (usize, usize) {
    let len = line.chars().count();
    (0, len)
}

/// Clamp a caret into a valid position for the filtered list.
pub fn clamp_log_pos(
    pos: LogPos,
    row_count: usize,
    line_at: impl Fn(usize) -> Option<String>,
) -> Option<LogPos> {
    if row_count == 0 {
        return None;
    }
    let row = pos.row.min(row_count - 1);
    let line = line_at(row)?;
    let len = line.chars().count();
    let col = pos.col.min(len);
    Some(LogPos { row, col })
}

/// Step caret left/right across logical line boundaries.
pub fn step_caret_horizontal(
    pos: LogPos,
    delta: isize,
    row_count: usize,
    line_len_at: impl Fn(usize) -> usize,
) -> LogPos {
    if row_count == 0 || delta == 0 {
        return pos;
    }
    let mut row = pos.row.min(row_count - 1);
    let mut col = pos.col;
    if delta < 0 {
        if col > 0 {
            col -= 1;
        } else if row > 0 {
            row -= 1;
            col = line_len_at(row);
        }
    } else {
        let len = line_len_at(row);
        if col < len {
            col += 1;
        } else if row + 1 < row_count {
            row += 1;
            col = 0;
        }
    }
    LogPos { row, col }
}

/// Build styled spans for a text segment with selection and optional find highlights.
///
/// `display_pad` is the number of leading display-only characters in `text` (e.g. hang-indent
/// spaces on soft-wrap continuation rows). They are not part of the logical line indices.
pub fn line_spans(
    text: &str,
    log_row: usize,
    line_char_start: usize,
    display_pad: usize,
    base_color: Color,
    theme: &Theme,
    selection: &TextSelection,
    find_q: &str,
    is_find_current: bool,
) -> Vec<Span<'static>> {
    if text.is_empty() {
        return vec![Span::raw(String::new())];
    }

    if find_q.is_empty() && !selection.is_active() {
        return vec![Span::styled(
            text.to_string(),
            base_style(theme, base_color, is_find_current, false),
        )];
    }

    let chars: Vec<char> = text.chars().collect();
    let logical_len = chars.len().saturating_sub(display_pad);
    let lower: Vec<char> = if find_q.is_empty() {
        Vec::new()
    } else {
        text.to_lowercase().chars().collect()
    };
    let q: Vec<char> = find_q.chars().collect();

    let mut spans = Vec::new();
    let mut i = 0usize;
    while i < chars.len() {
        let selected = is_display_selected(
            &selection,
            log_row,
            line_char_start,
            display_pad,
            logical_len,
            i,
        );

        // Find match at this position (logical text only, skip display pad)
        if !find_q.is_empty()
            && i >= display_pad
            && i + q.len() <= chars.len()
            && lower[i..i + q.len()] == q[..]
        {
            let matched: String = chars[i..i + q.len()].iter().collect();
            let logical_i = i - display_pad;
            let mut any_selected = false;
            for k in 0..q.len() {
                if selection.contains(log_row, line_char_start + logical_i + k) {
                    any_selected = true;
                    break;
                }
            }
            spans.push(Span::styled(
                matched,
                find_style(theme, is_find_current, any_selected),
            ));
            i += q.len();
            continue;
        }

        // Coalesce run with same (find-current, selected) flags
        let mut j = i + 1;
        while j < chars.len() {
            let sel_j = is_display_selected(
                &selection,
                log_row,
                line_char_start,
                display_pad,
                logical_len,
                j,
            );
            if sel_j != selected {
                break;
            }
            if !find_q.is_empty()
                && j >= display_pad
                && j + q.len() <= chars.len()
                && lower[j..j + q.len()] == q[..]
            {
                break;
            }
            j += 1;
        }
        let run: String = chars[i..j].iter().collect();
        spans.push(Span::styled(
            run,
            base_style(theme, base_color, is_find_current && find_q.is_empty(), selected),
        ));
        i = j;
    }

    if spans.is_empty() {
        spans.push(Span::styled(
            text.to_string(),
            base_style(theme, base_color, is_find_current, false),
        ));
    }
    spans
}

fn is_display_selected(
    selection: &TextSelection,
    log_row: usize,
    line_char_start: usize,
    display_pad: usize,
    logical_len: usize,
    display_i: usize,
) -> bool {
    if display_i < display_pad {
        if logical_len == 0 {
            return selection.contains(log_row, line_char_start);
        }
        (line_char_start..line_char_start + logical_len).any(|c| selection.contains(log_row, c))
    } else {
        selection.contains(log_row, line_char_start + (display_i - display_pad))
    }
}

fn find_style(theme: &Theme, is_current: bool, selected: bool) -> Style {
    if selected {
        Style::default()
            .fg(theme.selection_fg)
            .bg(theme.selection_bg)
            .add_modifier(Modifier::BOLD)
    } else {
        let mut style = Style::default().fg(theme.find_fg).bg(theme.find_bg);
        if is_current {
            style = style.add_modifier(Modifier::BOLD | Modifier::UNDERLINED);
        }
        style
    }
}

fn base_style(theme: &Theme, base: Color, is_find_current: bool, selected: bool) -> Style {
    if selected {
        Style::default()
            .fg(theme.selection_fg)
            .bg(theme.selection_bg)
            .add_modifier(Modifier::BOLD)
    } else {
        let mut style = Style::default().fg(base);
        if is_find_current {
            style = style.add_modifier(Modifier::REVERSED);
        }
        style
    }
}

fn contains(rect: Rect, col: u16, row: u16) -> bool {
    col >= rect.x
        && col < rect.x.saturating_add(rect.width)
        && row >= rect.y
        && row < rect.y.saturating_add(rect.height)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines<'a>(rows: &'a [&'a str]) -> impl Fn(usize) -> Option<String> + 'a {
        move |i| rows.get(i).map(|s| (*s).to_string())
    }

    fn map(soft_wrap: bool, scroll: usize, wrap_skip: usize, col_offset: usize) -> ViewportMap {
        ViewportMap {
            area: Rect {
                x: 2,
                y: 3,
                width: 10,
                height: 5,
            },
            scroll_offset: scroll,
            wrap_skip,
            col_offset,
            soft_wrap,
            viewport_width: 10,
            viewport_height: 5,
        }
    }

    #[test]
    fn line_spans_hang_indent_selection_includes_pad() {
        let theme = Theme::dark_accents();
        let mut sel = TextSelection::default();
        sel.start(LogPos { row: 0, col: 10 });
        sel.extend_to(LogPos { row: 0, col: 13 });

        let spans = line_spans(
            "   abc",
            0,
            10,
            3,
            Color::White,
            &theme,
            &sel,
            "",
            false,
        );

        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].style.bg, Some(theme.selection_bg));
        assert_eq!(spans[0].content, "   abc");
    }

    #[test]
    fn line_spans_hang_indent_pad_not_selected_before_chunk() {
        let theme = Theme::dark_accents();
        let mut sel = TextSelection::default();
        sel.start(LogPos { row: 0, col: 5 });
        sel.extend_to(LogPos { row: 0, col: 8 });

        let spans = line_spans(
            "   abc",
            0,
            10,
            3,
            Color::White,
            &theme,
            &sel,
            "",
            false,
        );

        assert_eq!(spans.len(), 1);
        assert_ne!(spans[0].style.bg, Some(theme.selection_bg));
    }

    #[test]
    fn normalized_range_orders_positions() {
        let mut sel = TextSelection::default();
        sel.start(LogPos { row: 2, col: 5 });
        sel.extend_to(LogPos { row: 1, col: 3 });
        let (a, b) = sel.normalized_range().unwrap();
        assert_eq!(a, LogPos { row: 1, col: 3 });
        assert_eq!(b, LogPos { row: 2, col: 5 });
    }

    #[test]
    fn extract_text_joins_lines() {
        let mut sel = TextSelection::default();
        sel.start(LogPos { row: 0, col: 0 });
        sel.extend_to(LogPos { row: 1, col: 3 });
        let text = sel
            .extract_text(|row| match row {
                0 => Some("hello".into()),
                1 => Some("world".into()),
                _ => None,
            })
            .unwrap();
        assert_eq!(text, "hello\nwor");
    }

    #[test]
    fn log_pos_to_screen_nowrap_roundtrip() {
        let rows = ["abcdefghijXXXX", "second"];
        let m = map(false, 0, 0, 0);
        let pos = mouse_to_log_pos(2 + 3, 3 + 1, &m, lines(&rows), 2).unwrap();
        assert_eq!(pos, LogPos { row: 1, col: 3 });
        assert_eq!(
            log_pos_to_screen(pos, &m, lines(&rows), 2),
            Some((2 + 3, 3 + 1))
        );
    }

    #[test]
    fn log_pos_to_screen_nowrap_respects_col_offset() {
        let rows = ["abcdefghijklmnop"];
        let m = map(false, 0, 0, 4);
        let pos = LogPos { row: 0, col: 6 };
        assert_eq!(
            log_pos_to_screen(pos, &m, lines(&rows), 1),
            Some((2 + 2, 3))
        );
        assert_eq!(
            log_pos_to_screen(LogPos { row: 0, col: 2 }, &m, lines(&rows), 1),
            None
        );
    }

    #[test]
    fn log_pos_to_screen_wrapped_edges() {
        // width 10: "0123456789ABCDEF" -> chunk0 cols 0-9, chunk1 cols 10-15
        let rows = ["0123456789ABCDEF"];
        let m = map(true, 0, 0, 0);
        assert_eq!(
            log_pos_to_screen(LogPos { row: 0, col: 0 }, &m, lines(&rows), 1),
            Some((2, 3))
        );
        assert_eq!(
            log_pos_to_screen(LogPos { row: 0, col: 9 }, &m, lines(&rows), 1),
            Some((2 + 9, 3))
        );
        assert_eq!(
            log_pos_to_screen(LogPos { row: 0, col: 10 }, &m, lines(&rows), 1),
            Some((2, 3 + 1))
        );
        let m_skip = map(true, 0, 1, 0);
        assert_eq!(
            log_pos_to_screen(LogPos { row: 0, col: 10 }, &m_skip, lines(&rows), 1),
            Some((2, 3))
        );
        assert_eq!(
            log_pos_to_screen(LogPos { row: 0, col: 0 }, &m_skip, lines(&rows), 1),
            None
        );
    }

    #[test]
    fn log_pos_to_screen_wrapped_hang_indent() {
        // "01234: 5678901234abcd" — indent 7 (message starts at col 7)
        let rows = ["01234: 5678901234abcd"];
        let m = map(true, 0, 0, 0);
        // col 12 is on continuation row (logical start 10), screen x = hang(7) + 2
        assert_eq!(
            log_pos_to_screen(LogPos { row: 0, col: 12 }, &m, lines(&rows), 1),
            Some((2 + 7 + 2, 3 + 1))
        );
        // click in pad maps to chunk start
        let pos = mouse_to_log_pos(2 + 3, 3 + 1, &m, lines(&rows), 1).unwrap();
        assert_eq!(pos, LogPos { row: 0, col: 10 });
    }

    #[test]
    fn expand_word_identifier_and_non_word() {
        assert_eq!(expand_word("foo MyApp_1 bar", 6), (4, 11));
        assert_eq!(expand_word("a=b", 1), (1, 2));
        assert_eq!(expand_word("", 0), (0, 0));
    }

    #[test]
    fn expand_line_covers_full_text() {
        assert_eq!(expand_line("hello"), (0, 5));
        assert_eq!(expand_line(""), (0, 0));
    }

    #[test]
    fn clamp_log_pos_shrinks() {
        let rows = ["ab", "cdef"];
        assert_eq!(
            clamp_log_pos(LogPos { row: 9, col: 99 }, 2, lines(&rows)),
            Some(LogPos { row: 1, col: 4 })
        );
        assert_eq!(clamp_log_pos(LogPos { row: 0, col: 0 }, 0, lines(&rows)), None);
    }

    #[test]
    fn step_caret_horizontal_crosses_line_bounds() {
        let lens = |row: usize| match row {
            0 => 3,
            1 => 1,
            2 => 0,
            _ => 0,
        };
        // Right within line 0 (gap before last char -> line-end gap)
        assert_eq!(
            step_caret_horizontal(LogPos { row: 0, col: 2 }, 1, 3, lens),
            LogPos { row: 0, col: 3 }
        );
        // Right from line-end gap of line 0 -> start of line 1
        assert_eq!(
            step_caret_horizontal(LogPos { row: 0, col: 3 }, 1, 3, lens),
            LogPos { row: 1, col: 0 }
        );
        // Right at end of last line stays at line-end gap
        assert_eq!(
            step_caret_horizontal(LogPos { row: 2, col: 0 }, 1, 3, lens),
            LogPos { row: 2, col: 0 }
        );
        // Left from start of line 1 -> line-end gap of line 0
        assert_eq!(
            step_caret_horizontal(LogPos { row: 1, col: 0 }, -1, 3, lens),
            LogPos { row: 0, col: 3 }
        );
        // Empty buffer: unchanged
        assert_eq!(
            step_caret_horizontal(LogPos { row: 0, col: 0 }, 1, 0, lens),
            LogPos { row: 0, col: 0 }
        );
        // At buffer start, Left stays
        assert_eq!(
            step_caret_horizontal(LogPos { row: 0, col: 0 }, -1, 3, lens),
            LogPos { row: 0, col: 0 }
        );
    }

    #[test]
    fn extract_text_half_open_single_char() {
        let mut sel = TextSelection::default();
        sel.start(LogPos { row: 0, col: 2 });
        sel.extend_to(LogPos { row: 0, col: 3 });
        let text = sel
            .extract_text(|row| match row {
                0 => Some("hello".into()),
                _ => None,
            })
            .unwrap();
        assert_eq!(text, "l");
    }

    #[test]
    fn extract_text_half_open_full_line() {
        let mut sel = TextSelection::default();
        sel.start(LogPos { row: 0, col: 0 });
        sel.extend_to(LogPos { row: 0, col: 5 });
        let text = sel
            .extract_text(|row| match row {
                0 => Some("hello".into()),
                _ => None,
            })
            .unwrap();
        assert_eq!(text, "hello");
    }

    #[test]
    fn extract_text_half_open_multiline_end_at_len() {
        let mut sel = TextSelection::default();
        sel.start(LogPos { row: 0, col: 0 });
        sel.extend_to(LogPos { row: 1, col: 5 });
        let text = sel
            .extract_text(|row| match row {
                0 => Some("hello".into()),
                1 => Some("world".into()),
                _ => None,
            })
            .unwrap();
        assert_eq!(text, "hello\nworld");
    }

    #[test]
    fn extract_text_shift_end_includes_last_char() {
        // Simulates Shift+End selecting through the last character: anchor at
        // col 0, caret at line_len. The last char must be included.
        let mut sel = TextSelection::default();
        sel.start(LogPos { row: 0, col: 0 });
        sel.extend_to(LogPos { row: 0, col: 5 });
        let text = sel
            .extract_text(|row| match row {
                0 => Some("hello".into()),
                _ => None,
            })
            .unwrap();
        assert_eq!(text, "hello");
        // contains uses strict upper bound: col 5 (line_len) is NOT in selection,
        // but col 4 (last char) IS.
        assert!(!sel.contains(0, 5));
        assert!(sel.contains(0, 4));
    }

    #[test]
    fn log_pos_to_screen_nowrap_line_end_gap() {
        let rows = ["abc"];
        let m = map(false, 0, 0, 0);
        // col == line_len (3) maps to the right edge of the last char.
        assert_eq!(
            log_pos_to_screen(LogPos { row: 0, col: 3 }, &m, lines(&rows), 1),
            Some((2 + 3, 3))
        );
    }

    #[test]
    fn log_pos_to_screen_nowrap_full_width_line_guard() {
        // Line fills the viewport exactly (width 10, line_len 10): the line-end
        // gap sits at col_in == width (one past the last visible cell).
        let rows = ["0123456789"];
        let m = map(false, 0, 0, 0);
        assert_eq!(
            log_pos_to_screen(LogPos { row: 0, col: 10 }, &m, lines(&rows), 1),
            Some((2 + 10, 3))
        );
        // A real char at col 10 doesn't exist; col 9 (last char) is visible.
        assert_eq!(
            log_pos_to_screen(LogPos { row: 0, col: 9 }, &m, lines(&rows), 1),
            Some((2 + 9, 3))
        );
    }

    #[test]
    fn log_pos_to_screen_wrapped_line_end_gap() {
        // width 10: "0123456789ABCDEF" -> chunk0 "0123456789", chunk1 "ABCDEF".
        // Line-end gap (col 16) belongs to the final chunk's end.
        let rows = ["0123456789ABCDEF"];
        let m = map(true, 0, 0, 0);
        assert_eq!(
            log_pos_to_screen(LogPos { row: 0, col: 16 }, &m, lines(&rows), 1),
            Some((2 + 6, 3 + 1))
        );
    }

    #[test]
    fn log_pos_to_screen_wrapped_full_width_chunk_guard() {
        // Final chunk fills the available width exactly: line-end gap at the
        // right edge. "0123456789AB" -> chunk0 "0123456789" (10), chunk1 "AB" (2).
        let rows = ["0123456789AB"];
        let m = map(true, 0, 0, 0);
        // col 12 == line_len, final chunk "AB" (start 10, len 2), col_in = 2.
        assert_eq!(
            log_pos_to_screen(LogPos { row: 0, col: 12 }, &m, lines(&rows), 1),
            Some((2 + 2, 3 + 1))
        );
    }

    #[test]
    fn mouse_to_log_pos_nowrap_past_last_char_is_line_end_gap() {
        let rows = ["abc"];
        let m = map(false, 0, 0, 0);
        // Click on the last char (screen col 2) -> left gap (col 2).
        assert_eq!(
            mouse_to_log_pos(2 + 2, 3, &m, lines(&rows), 1),
            Some(LogPos { row: 0, col: 2 })
        );
        // Click past the last char (screen col 3) -> line-end gap (col 3).
        assert_eq!(
            mouse_to_log_pos(2 + 3, 3, &m, lines(&rows), 1),
            Some(LogPos { row: 0, col: 3 })
        );
    }

    #[test]
    fn mouse_to_log_pos_wrapped_past_last_char_is_line_end_gap() {
        // "0123456789ABCDEF" width 10: chunk0 row 0, chunk1 "ABCDEF" row 1.
        let rows = ["0123456789ABCDEF"];
        let m = map(true, 0, 0, 0);
        // Click on the last char of chunk1 ('F' at screen col 5) -> left gap (col 15).
        assert_eq!(
            mouse_to_log_pos(2 + 5, 3 + 1, &m, lines(&rows), 1),
            Some(LogPos { row: 0, col: 15 })
        );
        // Click past the last char (screen col 6) -> line-end gap (col 16).
        assert_eq!(
            mouse_to_log_pos(2 + 6, 3 + 1, &m, lines(&rows), 1),
            Some(LogPos { row: 0, col: 16 })
        );
    }
}

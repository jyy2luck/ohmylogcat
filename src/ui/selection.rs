//! Mouse text selection in the log viewport.

use crate::ui::display::WrapChunks;
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

    pub fn finish_drag(&mut self) {
        self.dragging = false;
    }

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
        pos >= start && pos <= end
    }

    /// Extract selected plain text from formatted log lines.
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
                start.col.min(chars.len() - 1)
            } else {
                0
            };
            let to = if row == end.row {
                end.col.min(chars.len() - 1)
            } else {
                chars.len() - 1
            };
            if row > start.row {
                out.push('\n');
            }
            if from <= to {
                out.extend(chars[from..=to].iter());
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
    let char_col = (map.col_offset + col_in).min(line_len - 1);
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
        for (ci, chunk) in WrapChunks::new(&line, width).enumerate() {
            if skip > 0 {
                skip -= 1;
                continue;
            }
            if display_row == target_display {
                let chunk_start = ci * width;
                let chunk_len = chunk.chars().count();
                if chunk_len == 0 {
                    return Some(LogPos { row: idx, col: chunk_start });
                }
                let col_in_chunk = col_in.min(chunk_len - 1);
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

/// Build styled spans for a text segment with selection and optional find highlights.
pub fn line_spans(
    text: &str,
    log_row: usize,
    line_char_start: usize,
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
    let lower: Vec<char> = if find_q.is_empty() {
        Vec::new()
    } else {
        text.to_lowercase().chars().collect()
    };
    let q: Vec<char> = find_q.chars().collect();

    let mut spans = Vec::new();
    let mut i = 0usize;
    while i < chars.len() {
        let abs_col = line_char_start + i;
        let selected = selection.contains(log_row, abs_col);

        // Find match at this position
        if !find_q.is_empty() && i + q.len() <= chars.len() && lower[i..i + q.len()] == q[..] {
            let matched: String = chars[i..i + q.len()].iter().collect();
            let mut any_selected = false;
            for k in 0..q.len() {
                if selection.contains(log_row, abs_col + k) {
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
            let abs_j = line_char_start + j;
            let sel_j = selection.contains(log_row, abs_j);
            if sel_j != selected {
                break;
            }
            if !find_q.is_empty() && j + q.len() <= chars.len() && lower[j..j + q.len()] == q[..] {
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
        sel.extend_to(LogPos { row: 1, col: 2 });
        let text = sel
            .extract_text(|row| match row {
                0 => Some("hello".into()),
                1 => Some("world".into()),
                _ => None,
            })
            .unwrap();
        assert_eq!(text, "hello\nwor");
    }
}

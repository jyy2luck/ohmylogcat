## 1. Caret column semantics (gap index)

- [x] 1.1 Audit `LogPos.col` usages across `src/ui/selection.rs` and `src/app.rs`; confirm every site assumes `[0, len]` after the change (no remaining `len - 1` clamps for caret placement)
- [x] 1.2 Update `clamp_log_pos` to clamp `col = pos.col.min(len)` (empty line → 0)
- [x] 1.3 Update `clamp_col_for_row` to `preferred.min(len)` (empty line → 0)
- [x] 1.4 Update `move_caret_line_bound` so `End` sets `col = self.line_len_at(row)` (remove `saturating_sub(1)`); `Home` stays `0`
- [x] 1.5 Update `step_caret_horizontal`: `Right` → if `col < len` then `col += 1` else wrap to next row `col = 0` (stop at last line); `Left` → if `col > 0` then `col -= 1` else wrap to prev row `col = line_len_at(row)` (stop at first line); empty line `len = 0` advances to next row

## 2. Selection subsystem (half-open)

- [x] 2.1 Update `TextSelection::normalized_range` to return `(start, end)` ordered by gap order (no semantics change, but document half-open contract)
- [x] 2.2 Update `TextSelection::contains(row, col)` to test `start <= pos && pos < end` (strict upper bound)
- [x] 2.3 Update `TextSelection::extract_text` to slice `chars[from..to)` per row: `to = end.col.min(chars.len())` on the end row, `to = chars.len()` on intermediate rows, `from = start.col` on the start row; verify copied text is unchanged for existing selections and includes the last char when `end.col == len`
- [x] 2.4 Update `expand_word` to return `(start, end_exclusive)` with `end_exclusive = end + 1`; empty line → `(0, 0)`
- [x] 2.5 Update `expand_line` to return `(0, line_len)` (empty line → `(0, 0)`)
- [x] 2.6 Audit all `TextSelection` construction sites (drag, click, Shift+move, double/triple-click, find) to pass half-open endpoints

## 3. Caret ↔ screen mapping

- [x] 3.1 Update `log_pos_to_screen_nowrap`: for `pos.col < line_len` keep `x = area.x + (pos.col - col_offset)`; for `pos.col == line_len` use `x = area.x + (line_len - col_offset)`; guard the "past last cell" case when the line fills the viewport (clamp to `area.x + viewport_width - 1` if a backend rejects the right-edge column)
- [x] 3.2 Verify `col_offset` and screen x are in **display columns** (not char indices); if wide chars make `line_len - col_offset` wrong, compute x from `str_display_width` of the visible prefix instead
- [x] 3.3 Update `log_pos_to_screen_wrapped`: change `chunk_end` to `chunk_start + chunk_chars`; allow `pos.col == chunk_end` on the final chunk; compute continuation x as `area.x + hang + chunk_chars` (or `area.x + chunk_chars` on the first row) with the same "past last cell" guard
- [x] 3.4 Update `wrap_display_row_for_col` (and any wrap caret helpers) so `col == line_len` maps to the last chunk's end gap

## 4. Mouse hit-test to gap

- [x] 4.1 Update `mouse_to_log_pos_nowrap`: `target = col_offset + (col - area.x)`; if `target >= line_len` → `col = line_len` else `col = target` (left gap of the clicked cell)
- [x] 4.2 Update `mouse_to_log_pos_wrapped` to map the clicked chunk-local column to a gap and add `chunk_start`; past the last char of the chunk on the final chunk → `col = line_len`
- [x] 4.3 Verify single-click places the caret at the line-end gap when clicking past the last character

## 5. Viewport follow and preferred column

- [x] 5.1 Update `ensure_caret_visible` nowrap branch: change `else if caret.col >= self.col_offset + w` to strict `> `; keep pan `col_offset = caret.col + 1 - w` when beyond the right edge
- [x] 5.2 Confirm `caret_preferred_col` is stored in `[0, max_line_len]` and preserved across short lines via `clamp_col_for_row`
- [x] 5.3 Verify `display_col_of` returns a value consistent with the new gap range (no `len - 1` clamp)

## 6. Tests

- [x] 6.1 Update `step_caret_horizontal_crosses_line_bounds` and related `selection.rs` unit tests to gap-index expectations (Right at `len` wraps, Left at `0` wraps to prev `len`)
- [x] 6.2 Add tests: `End` → `col == line_len`; `Right` at last-line end stays; `Left` at first-line start stays; `Shift+End` selection includes last char (`extract_text` contains it)
- [x] 6.3 Add/update `extract_text` half-open tests: single char `[k, k+1)`, full line `[0, len)`, multi-line with end at `len`
- [x] 6.4 Add `log_pos_to_screen` tests for `col == line_len` (right edge) in nowrap and wrapped modes, including the full-width-line guard
- [x] 6.5 Add mouse hit-test tests: click past last char → line-end gap; click within a cell → left gap
- [x] 6.6 Run `cargo test` and fix any remaining failures from the semantic change

## 7. Manual verification

- [x] 7.1 Run the app, press `End` on a log line: bar appears at the right edge of the last character
- [x] 7.2 From the line-end gap, press `Right`: caret moves to next line col 0 (or stops on the last line)
- [x] 7.3 From col 0, press `Left` on a non-first line: caret moves to the previous line's line-end gap
- [x] 7.4 `Shift+End` selects through the last character; Ctrl/Cmd+C copies the full line content
- [x] 7.5 Double-click a word and triple-click a line: selection and copy contents unchanged
- [x] 7.6 Click past the last character of a line: caret placed at the line-end gap
- [x] 7.7 Soft-Wrap on: repeat End/Right/Left and click-past-last on a wrapped long line; caret lands on the final chunk's end gap

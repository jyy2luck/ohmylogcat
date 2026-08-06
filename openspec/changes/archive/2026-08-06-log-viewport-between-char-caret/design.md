## Context

The log viewport caret and selection subsystem (`src/ui/selection.rs`, `src/app.rs`) currently treat the caret column as a **character index** in `[0, line_len - 1]` (the "on-character" model). Endpoints are inclusive `[start, end]`, `extract_text` slices `chars[from..=to]`, and `log_pos_to_screen` draws the hardware bar at the left edge of the character at `col`. The text inputs (`src/ui/text_input.rs`) already use the standard "between-character" model (`cursor ∈ [0, len]`, `End → len`, `Right` at end stops). The mismatch makes `End` land on the last character (bar on its left edge) and `Right` from there wrap to the next line.

See `proposal.md` for motivation; see `specs/log-display/spec.md` for the target behavior.

## Goals / Non-Goals

**Goals:**
- Make the log viewport caret a gap index in `[0, line_len]` so `End` reaches the line-end gap and `Right`/`Left` wrap predictably.
- Convert the selection subsystem to half-open `[start, end)` endpoints without changing copied text contents.
- Keep mouse drag/click, double/triple-click, find, and copy behavior externally consistent with the new gap model.

**Non-Goals:**
- `Ctrl+Home`/`Ctrl+End` buffer jumps.
- Path-friendly (vim-style) word boundaries.
- Changing the text input cursor model (already between-character).
- Find match navigation semantics.

## Decisions

### Decision 1: Caret column is a gap index; selection is half-open

`LogPos.col` becomes a gap index in `[0, line_len]`. `TextSelection` endpoints become half-open: a selection covers characters `[start, end)` where `end.col` may equal `line_len`. `normalized_range` returns `(start, end)` with `start <= end` by gap order; `contains(row, col)` tests `start <= pos && pos < end` (strict upper bound). `extract_text` slices `chars[from..to)` per row, where `to = end.col.min(chars.len())` and the last row's `to` is the end gap; intermediate rows use `to = chars.len()`.

**Rationale:** This is the model editors and the app's own text inputs use. Half-open ranges make "select to end of line" naturally include the last character without off-by-one clamps, and unify single-character selection (`[k, k+1)`) with multi-line selection.

**Alternatives considered:**
- Keep on-character, only stop `Right` at line end (no wrap): leaves `End` visually wrong (bar on last char's left edge) and breaks the "Right wraps to next line" expectation.
- Keep on-character, draw the bar at the right edge only when the caret reached `col = len-1` via `End`: desynchronizes the hardware cursor from the selection highlight and the stored `col`; fragile.

### Decision 2: Screen mapping for `col = line_len`

`log_pos_to_screen_nowrap`: when `pos.col < line_len`, screen x = `area.x + (pos.col - col_offset)` (left edge of that char, unchanged). When `pos.col == line_len`, screen x = `area.x + (line_len - col_offset)` (right edge of the last char). Clamp/guard: if `line_len - col_offset >= viewport_width` (line fills the viewport exactly), the line-end gap x would be `area.x + viewport_width`, one past the last visible cell. Position the hardware cursor there; terminals allow a cursor column equal to the viewport width (no write occurs at the caret, so no auto-wrap is triggered). If a backend ever rejects that column, clamp to `area.x + viewport_width - 1` as a fallback (visually still the right edge of the last visible char).

`log_pos_to_screen_wrapped`: change `chunk_end` from `chunk_start + chunk_chars.saturating_sub(1)` to `chunk_start + chunk_chars`. A caret at `col == line_len` belongs to the last chunk of the line (`pos.col == chunk_end` on the final chunk); render at `area.x + hang + chunk_chars` on continuation rows, or `area.x + chunk_chars` on the first row. The same "past last cell" guard applies when a chunk exactly fills the available width.

**Rationale:** The bar must visually sit between characters; the only new position is the line-end gap, which is the right edge of the last char cell.

### Decision 3: Horizontal step and line-bound moves

`step_caret_horizontal(delta=+1)`: if `col < line_len` → `col += 1`; else if `row + 1 < row_count` → `row += 1, col = 0`; else stay. `delta=-1`: if `col > 0` → `col -= 1`; else if `row > 0` → `row -= 1, col = line_len_at(row)`; else stay. Empty lines have `line_len = 0`, so `col` is always `0` and `Right` advances to the next row.

`move_caret_line_bound(End)`: `col = self.line_len_at(row)` (no `saturating_sub(1)`). `Home`: `col = 0` (unchanged).

**Rationale:** Gap-index semantics make the wrap rules symmetric and remove the `len-1` special case.

### Decision 4: Clamping and preferred column

`clamp_log_pos`: `col = pos.col.min(len)` (was `len - 1`); empty line → `col = 0`. `clamp_col_for_row`: `preferred.min(len)` (was `len - 1`). `caret_preferred_col` now lives in `[0, max_line_len]`; vertical movement clamps per row to `[0, len]`, so a preferred column of `len` is preserved across short lines and re-attains the line-end gap on equally-long lines.

**Rationale:** Preferred-column memory must respect the new upper bound so `End`-then-`Down` keeps the caret at the line-end gap on lines of the same length.

### Decision 5: `ensure_caret_visible` horizontal pan

The existing branch `else if caret.col >= self.col_offset + w` becomes `else if caret.col > self.col_offset + w` (strict), because `col = col_offset + w` is now a valid in-viewport gap (the right edge of the last visible cell). When `col == line_len` and the line exactly fills the viewport, the gap sits at the right viewport edge and is considered visible (no pan). When the gap is beyond the right edge, pan so `col_offset = caret.col - w + 1` as before (this places the gap at the right edge of the new viewport).

**Rationale:** Avoids an off-by-one pan when the caret is exactly at the rightmost visible gap.

### Decision 6: Mouse hit-test to gap

`mouse_to_log_pos_nowrap`: compute `target = col_offset + (col - area.x)`. Map to gap: if `target >= line_len` → `col = line_len` (line-end gap). Otherwise `col = target` (gap at the left edge of the character at that screen column). We do **not** split a character cell into left/right halves (would require per-char width and East-Asian-width handling); a click within a cell places the caret at that cell's left gap, and clicking past the last character reaches the line-end gap. `mouse_to_log_pos_wrapped` follows the same rule within the clicked chunk, mapping the chunk-local column to a gap and adding `chunk_start`.

**Rationale:** Simple, deterministic, and consistent with how the text inputs map clicks (`cursor_from_click` returns a char index, not a half-cell gap). The spec's "nearer side" allowance is satisfied by "past the last char → line-end gap"; within a cell we pick the left gap for predictability.

**Alternatives considered:** Left/right half split per cell — rejected for complexity and East-Asian-width fragility, with no real ergonomic gain for a read-only log viewer.

### Decision 7: Word and line expand return half-open ranges

`expand_word` returns `(start, end_exclusive)` where `end_exclusive = end + 1` (so the word's characters are `[start, end_exclusive)`). `expand_line` returns `(0, line_len)` (was `(0, len - 1)`). Callers that build a `TextSelection` from these use the half-open endpoints directly. Empty line → `(0, 0)` (zero-width, no selection).

**Rationale:** Keeps the expand helpers consistent with the half-open selection model and removes the `len - 1` special case.

## Risks / Trade-offs

- **Terminal cursor at the right viewport edge**: placing the hardware cursor at column `area.x + viewport_width` (line-end gap when the line fills the viewport) may render as the cursor sitting on the terminal's last column or, on some terminals, trigger an implicit wrap on the next write. → Mitigation: we never write at the caret position; we only `MoveTo`. If a backend misbehaves, clamp to `area.x + viewport_width - 1` (still visually the right edge of the last char). Add a manual test on Windows Terminal + the configured backend.
- **Selection highlight at the line-end gap**: a half-open selection ending at `col = line_len` highlights no extra cell (the gap has no cell). This is correct but means a "select to end of line" highlight stops at the last char with no trailing marker, which some users expect to see extended. → Mitigation: accepted; matches editor behavior where the bar sits after the last char and the highlight covers exactly the selected characters.
- **Behavioral break for existing users**: users accustomed to `End` landing on the last character and `Right` wrapping immediately will see different behavior. → Mitigation: this is the intended fix; document in README keybindings table (no change to the table wording, which already says "Caret to start / end of current logical line").
- **Test churn**: `selection.rs` and `app.rs` unit tests encode the old inclusive/`len-1` semantics. → Mitigation: update tests as part of the change; add explicit scenarios for `End → line_len`, `Right` at line end, `Left` at line start, and half-open copy.
- **Wide (East-Asian) characters**: gap-index math uses char indices, not display widths, so wide chars are handled at the screen-mapping layer only. The line-end gap x computation uses `line_len - col_offset` (char count) which is wrong for lines containing wide chars when `col_offset` is in display columns. → Mitigation: confirm `col_offset` and screen x are in **display columns**, not char indices; if the current code already mixes them, fix screen x to use `str_display_width` of the visible prefix rather than `line_len - col_offset`. Audit during implementation (Task: verify width vs char-index in nowrap screen mapping).

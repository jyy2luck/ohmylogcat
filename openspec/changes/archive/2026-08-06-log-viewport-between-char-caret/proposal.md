## Why

The log viewport caret uses an "on-character" model where `col ∈ [0, len-1]` lands the blinking bar on a character cell (its left edge). `End` therefore stops on the last character instead of after it, and `Right` from that position wraps to the next line because there is no in-line position to the right of the last character. This contradicts editor muscle memory and is inconsistent with the app's own text inputs (Find, filter modals), which use the standard "between-character" model (`cursor ∈ [0, len]`, `End` → after the last character, `Right` at line end stops or wraps predictably).

## What Changes

- **BREAKING**: Switch the log viewport caret from the on-character model to the between-character model. The caret column becomes a gap index in `[0, line_len]`, where `col = line_len` is the position after the last character of the logical formatted line.
- **BREAKING**: `End` moves the caret to `col = line_len` (after the last character); `Home` stays at `col = 0`. `Shift+End` selects through the end of the line (half-open, inclusive of the last character).
- **BREAKING**: `Right` at `col = line_len` moves to `col = 0` of the next logical line (or stops at the end of the last line); `Left` at `col = 0` moves to `col = line_len` of the previous logical line.
- Convert the selection subsystem from inclusive `[start, end]` endpoints to half-open `[start, end)` endpoints, including `contains`, `normalized_range`, and `extract_text`. Copy output is unchanged (still includes the last selected character).
- Update caret↔screen mapping (`log_pos_to_screen` nowrap and wrapped), mouse hit-testing (`mouse_to_log_pos`), clamping (`clamp_log_pos`, `clamp_col_for_row`), vertical movement preferred column, and `ensure_caret_visible` horizontal pan so that `col = line_len` maps to the right edge of the last character cell.
- No changes to text input cursor model (already between-character); no new key bindings; no new dependencies.

## Capabilities

### New Capabilities

- (none)

### Modified Capabilities

- `log-display`: caret column semantics (gap index in `[0, line_len]`), `End`/`Home`/`Right`/`Left` line-bound and wrap behavior, selection endpoint semantics (half-open), caret screen mapping at the line-end gap, mouse hit-test mapping to the line-end gap

> `tui-shell`'s "Log viewport editor-like keyboard bindings" requirement delegates caret behavior to `log-display`, so it has no spec-level change here.

## Impact

- Primary: `src/ui/selection.rs` — `step_caret_horizontal`, `clamp_log_pos`, `log_pos_to_screen_nowrap`, `log_pos_to_screen_wrapped` (including `chunk_end` and continuation mapping), `mouse_to_log_pos_nowrap`, `mouse_to_log_pos_wrapped`, `TextSelection::contains` / `normalized_range` / `extract_text`, word/line expand extents
- Primary: `src/app.rs` — `move_caret_line_bound` (`End` → `line_len`), `clamp_col_for_row`, `caret_preferred_col` range, `ensure_caret_visible` `col_offset` math, `display_col_of`, drag/click caret placement
- Tests: `selection.rs` unit tests (`step_caret_horizontal_crosses_line_bounds`, selection ordering, extract_text) and any `app.rs` caret/selection tests must be updated to the new semantics
- Specs: `openspec/specs/log-display`, `openspec/specs/tui-shell`
- Out of scope: `Ctrl+Home`/`Ctrl+End` buffer jump; path-friendly word boundaries; text input cursor model; find match navigation

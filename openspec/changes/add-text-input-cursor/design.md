## Context

Tag/Message filter modals and the Find bar store filter/query strings in `String` fields and handle input with append-only `push`/`pop`. The UI renders values inside bracketed labels (`Tag contains: [value]`, `Find:[query]`) with no insertion index and no terminal cursor position. Log viewport mouse handling already sets contextual pointer shapes via crossterm `SetCursorStyle`.

## Goals / Non-Goals

**Goals:**

- Shared `TextInput { text, cursor }` model with char-boundary cursor index
- Visible terminal insertion cursor (`BlinkingBar`) at the correct screen cell while Tag/Message modal or Find bar is focused
- Keyboard: Left, Right, Home, End, Backspace, Delete, printable insert at cursor
- Mouse: I-beam over input hit regions; click maps column to cursor index
- Open with cursor at end of existing text; independent cursor state per surface
- Filter/query live-apply only on text change (cursor moves do not refilter)

**Non-Goals:**

- Input viewport horizontal scrolling for long strings (v1 shows full value; cursor may extend past modal width)
- Cursor support for Export path or Settings text fields (future reuse of `TextInput`)
- h/l as cursor keys in text inputs
- Select-all-on-open or overwrite mode

## Decisions

### 1. Shared `TextInput` helper in `src/ui/text_input.rs`

**Choice:** One struct plus `handle_key`, `cursor_from_click`, and `cursor_screen_offset(prefix_display_width)` helpers.

**Rationale:** Tag, Message, and Find share identical editing semantics; avoids three copies of Left/Delete logic.

**Alternative:** Inline cursor fields in `app.rs` and `FindState` — rejected as duplicated and error-prone.

### 2. Terminal hardware cursor (ratatui `set_cursor_position`)

**Choice:** After drawing the active text field, compute screen `(x, y)` from field origin + prefix width + display width of text before cursor; call `frame.set_cursor_position` and `frame.set_cursor_visibility(true)`. Set crossterm `SetCursorStyle::BlinkingBar` while editing.

**Rationale:** Native blink and shape; minimal draw code.

**Alternative:** Render a `▌` span — rejected for v1; would need manual blink tick.

**Fallback:** A module-level constant can switch to `SteadyBar` if a terminal blinks incorrectly.

### 3. Cursor index as char offset

**Choice:** `cursor: usize` counts Unicode scalar values (`str::chars`), consistent with existing `truncate_input` / filter strings.

**Rationale:** Tag/Message/Find values are ASCII-heavy; char boundaries match `String::insert`/`remove`.

### 4. Hit regions and pointer shape

**Choice:** Extend `HitMap` with `filter_modal_input: Option<Rect>` (value brackets inside modal) and `find_input: Option<Rect>`. `apply_pointer_shape` priority:

1. Mouse over text-input hit rect → I-beam (`BlinkingBar` or `SteadyBar` on hover only; editing uses positioned cursor)
2. Mouse over log viewport (no modal) → I-beam steady bar (existing)
3. Else → Default

Modal chrome outside the value brackets keeps default arrow.

**Click mapping:** `index = clamp((click_col - value_start_col).display_width_to_char_index, 0, text.len())`. Clicks right of text snap to end.

### 5. Find bar integration

**Choice:** Add `cursor: usize` to `FindState` (or embed `TextInput`). Left/Right/Home/End/Delete route through shared handler before existing Enter/Shift+Enter match navigation. Closing Find clears query and cursor (existing `close()` behavior).

### 6. Filter modal integration

**Choice:** Keep `filter_tag` / `filter_message` as app-level strings; add `filter_tag_cursor` / `filter_message_cursor` (or wrap in `TextInput`). On `open_filter_edit`, set cursor to `text.len()`.

## Risks / Trade-offs

- **[Risk] Cursor screen position miscalculation with modal borders/padding** → Mitigation: compute from rendered `Rect` + measured prefix display width; manual smoke on common terminal widths.
- **[Risk] Some terminals ignore `BlinkingBar` or blink oddly** → Mitigation: single constant to fall back to `SteadyBar`.
- **[Risk] Long filter strings extend past modal; cursor off-screen** → Accepted for v1 (non-goal: input scroll).
- **[Risk] Mouse click vs. bracket characters** → Mitigation: hit rect covers only `[value]` interior, not label or closing bracket.

## Migration Plan

No data migration. Ship in one release. Export/Settings remain append-only until a follow-up wires them to `TextInput`.

## Open Questions

_(none — explore session resolved open/close cursor placement, scope, and mouse support)_

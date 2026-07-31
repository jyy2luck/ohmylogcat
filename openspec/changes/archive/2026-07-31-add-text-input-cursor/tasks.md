# Tasks: add-text-input-cursor

## 1. TextInput module

- [x] 1.1 Add `src/ui/text_input.rs` with `TextInput { text, cursor }`, char-boundary insert/remove, and `handle_key` for Left/Right/Home/End/Backspace/Delete/printable
- [x] 1.2 Add `cursor_from_click(col, value_start_col, text)` for click-to-index mapping
- [x] 1.3 Add helper to compute display width from start of string to cursor for screen position
- [x] 1.4 Export `TextInput` from `src/ui/mod.rs`

## 2. Filter modal integration

- [x] 2.1 Add `filter_tag_cursor` / `filter_message_cursor` (or embedded `TextInput`) to `OhmylogcatApp`
- [x] 2.2 On `open_filter_edit`, set cursor to end of current filter string
- [x] 2.3 Replace append-only `handle_filter_edit_modal_key` with shared `TextInput` key handling; keep Esc and live `mark_filter_dirty()` on text change only
- [x] 2.4 Update `draw_modal` FilterEdit branch: record `hit_map.filter_modal_input` for value brackets; call `set_cursor_position` + `set_cursor_visibility(true)` when modal focused
- [x] 2.5 Handle mouse click in filter modal input region to set cursor index

## 3. Find bar integration

- [x] 3.1 Add cursor index to `FindState` (or embed `TextInput` for query)
- [x] 3.2 On `open_find`, set cursor to end of query; reset on `close()`
- [x] 3.3 Route Left/Right/Home/End/Backspace/Delete through `TextInput` in `handle_find_key`; preserve Enter/Shift+Enter match navigation
- [x] 3.4 Update `draw_find`: record `hit_map.find_input` for `[query]` region; position terminal cursor when Find focused
- [x] 3.5 Handle mouse click in Find input region to set cursor index

## 4. Pointer shape and draw lifecycle

- [x] 4.1 Extend `HitMap` with `filter_modal_input` and `find_input`
- [x] 4.2 Update `apply_pointer_shape` for I-beam over text-input hit regions; hide hardware cursor when not editing
- [x] 4.3 Set `SetCursorStyle::BlinkingBar` while text input focused (constant fallback to SteadyBar if needed)

## 5. Verification

- [x] 5.1 Manual smoke: Tag modal — open with existing value, move cursor mid-string, insert/delete/backspace, Esc keeps filter
- [x] 5.2 Manual smoke: Message modal — same cursor behaviors
- [x] 5.3 Manual smoke: Find bar — cursor edit + Enter/Shift+Enter still jump matches; mouse click positions cursor
- [x] 5.4 Manual smoke: pointer I-beam over input regions; default over toolbar/filter chrome
- [x] 5.5 `cargo build` and `cargo test` pass

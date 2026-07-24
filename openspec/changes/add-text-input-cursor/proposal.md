## Why

Tag/Message filter modals and the Find bar accept text input but only support append-at-end editing: Backspace always deletes the last character, and there is no visible insertion cursor or way to edit in the middle of a string. This makes correcting typos or inserting substrings awkward and unlike normal text fields.

## What Changes

- Add a movable text insertion cursor (visible terminal I-beam) to Tag filter modal, Message filter modal, and Find bar.
- Support keyboard navigation: Left/Right, Home/End, Backspace at cursor, Delete at cursor, and character insert at cursor.
- When a text-input surface opens, place the cursor at the end of the current string.
- Support mouse click within the input value area to position the cursor; show I-beam pointer over text-input hit regions.
- Use terminal hardware cursor (`BlinkingBar` by default) positioned via ratatui `set_cursor_position` while a text input is focused.
- Cursor state is independent per surface (Tag, Message, Find); no input viewport scrolling in v1.
- Export path and Settings text fields remain append-only for this change.

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

- `tui-shell`: text-input cursor behavior, keyboard editing semantics, and contextual pointer shape for Tag/Message filter modals and Find bar

## Impact

- `src/ui/` — new shared `TextInput` helper (state, key handling, cursor position math, optional draw helper)
- `src/app.rs` — integrate cursor into filter modals and Find bar; extend `HitMap` and `apply_pointer_shape`; mouse click-to-cursor in input regions
- `src/ui/find.rs` — store cursor index alongside query (or delegate to `TextInput`)
- `openspec/specs/tui-shell/spec.md` — requirement updates for text-input cursor and pointer behavior

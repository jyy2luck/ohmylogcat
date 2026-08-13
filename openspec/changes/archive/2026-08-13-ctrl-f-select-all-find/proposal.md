## Why

Ctrl+F currently opens Find only from the log viewport, and when Find is already focused it does nothing. Re-focusing also places the insertion cursor at the end of the query, so typing appends instead of replacing. Editors treat Ctrl+F as “focus the find field and select the current query,” so Enter continues the same search and typing starts a new one.

## What Changes

- Ctrl+F (and Cmd+F when the terminal delivers it) always focuses the Find bar when no modal is open, including when Find already has focus.
- If the Find query is non-empty, Ctrl+F selects all of that query (visually and for replace-on-type).
- Enter / Shift+Enter still go to next / previous match and do not clear the selection.
- Typing, Backspace, or Delete while selected replaces or clears the whole query; arrow keys, Home, End, or a mouse click in the Find input collapse the selection into a normal insertion cursor.
- `/` is unchanged: from the log viewport it opens or re-focuses Find with the cursor at the end of the query; while Find is focused it remains a literal `/` character.
- Tag / Message filter modals, export path, and Settings text fields are unchanged. Ctrl+F does not steal keys from an open modal.

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

- `log-display`: Ctrl+F / Cmd+F always focuses Find (when no modal is open) and selects the current query when it is non-empty
- `tui-shell`: Find-bar select-all state, visual selection of the query, replace-on-type, and collapse rules; Find via Ctrl+F is an exception to “cursor at end when opening”

## Impact

- `src/app.rs` — intercept Ctrl/Cmd+F above per-focus dispatch when no modal is open; wire select-all on that shortcut
- `src/ui/find.rs` — select-all on Ctrl+F; keep `/` / `open_bar` cursor-at-end behavior
- `src/ui/text_input.rs` — Find-only select-all flag (or equivalent) with replace / collapse key handling
- `src/app.rs` `draw_find` — highlight the selected query inside the brackets
- `README.md` — shortcut table: Ctrl+F focuses Find and selects the query
- Tests for TextInput select-all and App Ctrl+F focus/select behavior

## 1. TextInput select-all

- [x] 1.1 Add `select_all: bool` to `TextInput` (default false) and a `select_all()` method that sets the flag
- [x] 1.2 In `handle_key`, when `select_all` is set: printable replaces the whole string; Backspace/Delete clears it; Left/Home collapse to index 0; Right/End collapse to the end; all of those clear the flag
- [x] 1.3 Unit tests in `text_input.rs`: replace-on-type, Backspace clears, arrows collapse without changing text, empty `select_all` is a no-op for display

## 2. Ctrl+F activate path

- [x] 2.1 Add a dedicated Find activate helper: open the bar if closed, set `Focus::Find`, then `input.select_all()` (do not reuse `open_bar` for the select-all step)
- [x] 2.2 Keep `open_bar` as open + cursor at end + `select_all = false`; `/` from the log viewport still calls it via `open_find`
- [x] 2.3 In `handle_key`, after copy/quit checks and only when `modal.is_none()`, intercept Ctrl+F / Cmd+F, call the activate helper, and return — before the `Focus` match
- [x] 2.4 Remove the Ctrl/Cmd+F branch from `handle_logs_key` so Logs, Find, and Level share the new path
- [x] 2.5 On Find input mouse click, set cursor from click and clear `select_all`

## 3. Draw selected query

- [x] 3.1 Split `draw_find` into prefix / query / suffix spans; when Find is focused, `select_all` is set, and the query is non-empty, style the query with `theme.selection_fg` / `selection_bg`
- [x] 3.2 Keep the hardware cursor at the end of the query while selected; prefix, brackets, counter, and help stay on the existing field style

## 4. Verification

- [x] 4.1 App tests: Ctrl+F from Logs, Find, and Level focuses Find and selects a non-empty query; `/` from Logs focuses Find without selecting; Ctrl+F with a modal open does not steal focus; Enter with selection advances the match and leaves `select_all` set
- [x] 4.2 App test: typing after Ctrl+F select-all replaces the query and recomputes matches
- [x] 4.3 Update README shortcut table: Ctrl/Cmd+F focuses Find and selects the current query
- [x] 4.4 `cargo test` passes

# Tasks: add-log-text-selection

## 1. Selection module

- [x] 1.1 Add `src/ui/selection.rs` with `LogPos`, `TextSelection`, coordinate mapping, span helper, text extraction
- [x] 1.2 Export selection types from `src/ui/mod.rs`

## 2. App integration

- [x] 2.1 Extend `handle_mouse` for Moved / Drag / Up; clear selection on chrome click
- [x] 2.2 Add Cmd+C / Ctrl+C copy when selection active in `handle_key`
- [x] 2.3 Apply selection highlights in `draw_logs` (wrap and no-wrap paths)
- [x] 2.4 Clear selection on filter apply, clear buffer, dropped-front events
- [x] 2.5 Implement `update_mouse_cursor` with SteadyBar / DefaultUserShape

## 3. Dependencies and shell

- [x] 3.1 Add `arboard` to `Cargo.toml`
- [x] 3.2 Restore default cursor on exit in `main.rs`

## 4. Docs

- [x] 4.1 Update README shortcuts for drag-select and copy
- [x] 4.2 Sync main `openspec/specs` from change deltas

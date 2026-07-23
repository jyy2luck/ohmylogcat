## 1. Quit routing

- [x] 1.1 Remove Ctrl+C → `should_quit` handler from `handle_key`
- [x] 1.2 Add `is_top_layer()` helper (`modal.is_none() && !find.open`)
- [x] 1.3 Handle top-layer `q` / `Q` at start of `handle_key`; remove duplicate quit branch from `handle_logs_key`
- [x] 1.4 Ensure overlay contexts never set `should_quit` on `q` (text overlays insert `q`; non-text overlays no-op)

## 2. Filter modal

- [x] 2.1 Add `FilterField` enum and `ModalKind::FilterEdit { field }` to `app.rs`
- [x] 2.2 Implement `open_filter_edit(field)` and wire `t` / `m` from log viewport plus mouse click on filter summaries
- [x] 2.3 Implement `handle_filter_edit_modal_key`: text input with live `mark_filter_dirty()`; Esc closes modal and sets focus to Logs
- [x] 2.4 Add `draw_modal` branch for filter edit popup with hint text (`Live filter · Esc done`)
- [x] 2.5 Remove `Focus::Tag` / `Focus::Message`, `handle_text_field_key` filter paths, and inline filter focus from mouse/Tab handlers
- [x] 2.6 Update `draw_filters` to read-only summaries and revised hint line (no inline edit focus styling)

## 3. Toolbar and docs

- [x] 3.1 Append `[q]Quit` label to toolbar in `draw_toolbar`
- [x] 3.2 Update README keyboard table: quit is top-layer `q` only; document Tag/Message modal flow; remove Ctrl+C quit

## 4. Verification

- [x] 4.1 Manual smoke: top-layer `q` quits; Ctrl+C does not quit
- [x] 4.2 Manual smoke: Tag/Message modal — type including `q`, live filter, Esc closes with value kept
- [x] 4.3 Manual smoke: Devices / Export menu — `q` no-op, Esc returns; Find — `q` types, does not quit
- [x] 4.4 Manual smoke: Level inline unchanged; toolbar shows `[q]Quit`

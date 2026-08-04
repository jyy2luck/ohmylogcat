## 1. Formatting: fixed PID/TID + message indent

- [x] 1.1 Update `format_log_line` to pad PID/TID with `{:5}` and add a helper that returns the message-column indent (char length of the prefix through `{tag}: `)
- [x] 1.2 Add unit tests for short PID/TID padding and indent length on a sample entry

## 2. Hang-indent wrap primitives

- [x] 2.1 Extend `src/ui/display.rs` with hang-indent-aware wrap count + chunk iterator yielding `(logical_char_start, chunk)`; fallback to equal-width when `indent == 0` or `indent >= width`
- [x] 2.2 Add unit tests for row count, first vs continuation capacities, and narrow-viewport fallback

## 3. Paint and scroll

- [x] 3.1 Update `draw_logs` Soft-Wrap path to use hang-indent chunks, prefix continuation rows with display-only spaces, and pass correct logical `line_char_start` into `line_spans`
- [x] 3.2 Update `entry_wrap_height` and any wrap scroll / follow / `wrap_skip` math to use hang-indent row counts with per-entry indent

## 4. Selection and caret mapping

- [x] 4.1 Update `mouse_to_log_pos_wrapped` so pad clicks map to chunk start and content clicks use logical offsets
- [x] 4.2 Update `log_pos_to_screen_wrapped` (and any related wrap caret movement helpers) to add indent to screen x on continuation rows
- [x] 4.3 Add/adjust selection unit tests for hang-indent mouse and screen mapping

## 5. Verification

- [x] 5.1 Run unit tests for format/display/selection
- [x] 5.2 Manual smoke: Soft-Wrap on — long line hangs under message; Soft-Wrap off — single row + pan; short PID rows align Level column; find/copy exclude hang pad spaces

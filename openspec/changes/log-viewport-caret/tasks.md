## 1. Caret model and geometry

- [ ] 1.1 Add caret state on `App` (`Option<LogPos>`), seed/clamp helpers for empty list, filter shrink, and clear
- [ ] 1.2 Add wrap-aware `log_pos_to_screen` (inverse of `mouse_to_log_pos`) and unit tests for no-wrap / wrap edges
- [ ] 1.3 Implement `ensure_caret_visible` for vertical scroll and horizontal `col_offset` when Soft-Wrap is off
- [ ] 1.4 Add word-boundary and logical-line expand helpers on formatted line text (`[A-Za-z0-9_]` rule)

## 2. Keyboard navigation and selection

- [ ] 2.1 Replace log-viewport arrow / Home / End / PageUp / PageDown handling with caret moves; remove `j`/`k` scroll bindings
- [ ] 2.2 Implement Shift+arrows / Shift+Home / Shift+End selection (anchor on first extend; clear selection on plain moves)
- [ ] 2.3 Call `ensure_caret_visible` after every keyboard caret move; do not call it from wheel or follow scroll alone
- [ ] 2.4 Preserve preferred column for Up/Down across short lines when practical

## 3. Mouse and clipboard

- [ ] 3.1 Remove mouse-up auto `copy_selection`; keep Ctrl/Cmd+C as sole clipboard write for selections
- [ ] 3.2 Single-click in log viewport sets caret and clears selection (drag select unchanged)
- [ ] 3.3 Add multi-click tracking (time + position); double-click word select; triple-click logical-line select; no clipboard side effects
- [ ] 3.4 Keep selection cleared on chrome click; sync caret with selection live end while dragging

## 4. Drawing and cursor ownership

- [ ] 4.1 When `Focus::Logs` owns the caret, set terminal `BlinkingBar` and `frame.set_cursor_position` at caret screen cell
- [ ] 4.2 Ensure Find/modal text inputs still own the hardware cursor when focused; hide or yield log caret when list empty or caret off-screen

## 5. Verification and docs

- [ ] 5.1 Add unit tests for word/line expand and caret movement edge cases (line boundaries, empty buffer)
- [ ] 5.2 Manually smoke: arrows, Shift select, Home/End, Page keys, double/triple click, wheel then arrow follow, Ctrl/Cmd+C only
- [ ] 5.3 Update README shortcut docs for caret navigation, selection, and copy; drop `j`/`k` and mouse-up copy claims

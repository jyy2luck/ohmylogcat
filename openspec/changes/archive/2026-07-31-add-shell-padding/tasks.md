## 1. Shell inset layout

- [x] 1.1 Add `shell_content_area(area: Rect) -> Rect` helper in `src/app.rs` with adaptive 1-cell inset when width ≥ 60 and height ≥ 15, else zero inset
- [x] 1.2 Wrap `draw()` vertical chunk split to use inset content area; keep `draw_modal(frame, area)` on full `frame.area()`

## 2. Verify chrome and interaction

- [x] 2.1 Confirm toolbar, filters, find bar, separators, and status render within inset bounds (dividers align with content width)
- [x] 2.2 Smoke-test mouse: toolbar/filter clicks, gutter clicks outside log area, log text drag selection at left/right/top/bottom edges of inset viewport
- [x] 2.3 Smoke-test soft-wrap width and follow-scroll after inset (viewport resize triggers re-follow as today)

## 3. Validation

- [x] 3.1 Run `cargo test` and `cargo build`
- [x] 3.2 Run `openspec validate add-shell-padding --strict`

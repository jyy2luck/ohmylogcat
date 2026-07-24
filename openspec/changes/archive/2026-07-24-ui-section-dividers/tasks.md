# Tasks: ui-section-dividers

## 1. Layout and separators

- [x] 1.1 Add `draw_separator()` helper rendering full-width `─` in DarkGray
- [x] 1.2 Extend `draw()` vertical layout with three `Constraint::Length(1)` separator rows (toolbar↔filters, filters↔logs, logs↔status); preserve find bar placement when open
- [x] 1.3 Wire separator areas to `draw_separator()` calls

## 2. Filter row labels

- [x] 2.1 Change `draw_filters()` labels to `[t]Tag[value]`, `[m]Message[value]`, `[l]Level[value]` with bold shortcut spans
- [x] 2.2 Use empty brackets for unset Tag/Message; always show current level for Level
- [x] 2.3 Remove `(t/m edit · l level · …)` trailing hint; keep `(click Tag/Message)`
- [x] 2.4 Recalculate filter row hit-map rects for new label widths

## 3. Verification and docs

- [x] 3.1 Manual smoke: dividers visible; filter labels and mouse clicks on Tag/Message still open modals; Level focus styling unchanged
- [x] 3.2 Update README smoke checklist if it references undivided layout wording
- [x] 3.3 Sync main `openspec/specs/tui-shell` from change delta on archive

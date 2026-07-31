## Why

The log viewport already supports mouse drag selection and Ctrl/Cmd+C copy, but keyboard navigation still only scrolls the viewport—there is no visible caret, no Shift-based range selection, and no editor-like click gestures. Users inspecting logs need to move, select, and copy text with the same muscle memory as a read-only editor. Mouse-up auto-copy also fights that model by writing the clipboard as a side effect of selecting.

## What Changes

- Add a visible **caret** in the log viewport (terminal native blinking bar) that marks the current character position.
- **BREAKING**: Arrow keys move the caret (with viewport follow after keyboard moves) instead of scrolling the viewport directly; remove `j`/`k` log-viewport scroll bindings.
- **BREAKING**: `Home`/`End` move the caret to the current logical line start/end (not buffer top/bottom); `Shift+Home`/`Shift+End` extend selection to line start/end.
- Support `Shift+←↑↓→` for keyboard range selection anchored at the caret.
- Keep `PageUp`/`PageDown`; they move the caret by one page and ensure the caret is visible.
- **BREAKING**: Stop copying on mouse-up; clipboard writes only via Ctrl+C (Windows) / Cmd+C (macOS).
- Add double-click word select (identifier rule `[A-Za-z0-9_]`) and triple-click logical-line select; neither writes the clipboard.
- Allow the caret to leave the viewport during mouse wheel scroll or log refresh; the next keyboard caret move re-runs ensure-visible.

## Capabilities

### New Capabilities

- (none)

### Modified Capabilities

- `log-display`: caret visibility, viewport follow rules, Home/End semantics, selection/copy behavior for the log list
- `tui-shell`: log-viewport key bindings, keyboard selection, multi-click mouse gestures, remove mouse-up auto-copy and `j`/`k` scroll shortcuts

## Impact

- Primary: `src/app.rs` (key/mouse handling, caret state, ensure-visible, draw cursor), `src/ui/selection.rs` (word/line expand, caret vs selection), possibly `src/ui/display.rs` for wrap-aware caret mapping
- Specs: `openspec/specs/log-display`, `openspec/specs/tui-shell`
- No new dependencies; reuses existing crossterm cursor positioning used by text inputs
- Out of scope this change: `Ctrl+Home`/`Ctrl+End` buffer jump; path-friendly word boundaries; optional auto-copy preference

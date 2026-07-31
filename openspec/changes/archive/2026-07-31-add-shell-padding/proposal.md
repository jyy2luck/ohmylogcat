## Why

The TUI main shell renders edge-to-edge in the terminal with no inset on any side. Content, hit targets, and log text selection all start at column 0 and extend to the last column, which makes mouse drag selection awkward at the edges, toolbar and filter controls feel cramped, and the I-beam cursor hard to see against the terminal border. A modest uniform inset improves usability for both text selection and clickable chrome without changing log or filter semantics.

## What Changes

- Add a uniform outer inset (padding) around the entire main shell layout so toolbar, filter row, dividers, log viewport, and status bar render inside a smaller content area rather than flush against the terminal edges.
- Use adaptive inset: 1 column and 1 row on each side when the terminal is large enough; reduce or disable inset on very small terminals to preserve minimum usable log height and width.
- Constrain horizontal dividers to the inset content width (not full terminal width) so separators align with the padded shell.
- Keep modal overlays centered against the full terminal area (not the inset shell) so dialogs remain visually balanced.
- Ensure mouse hit regions, log viewport dimensions, and soft-wrap width all derive from the inset content area so text selection and click targets stay accurate.

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

- `tui-shell`: main layout shell inset (padding) around chrome and log viewport; divider width within inset; adaptive inset on small terminals; mouse hit regions and selection mapping aligned to inset log area

## Impact

- `src/app.rs` — `draw()` outer inset layout, separator width, hit-map rects, log viewport width/height for wrap and selection
- `openspec/specs/tui-shell/spec.md` — requirement delta for shell viewport inset
- No engine, filter, streaming, or dependency changes

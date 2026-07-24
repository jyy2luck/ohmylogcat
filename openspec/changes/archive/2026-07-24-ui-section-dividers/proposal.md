# Proposal: UI section dividers and filter shortcut labels

## Why

The TUI main shell stacks toolbar, filter row, log viewport, and status bar as adjacent single-line rows with no visual separation. Filter shortcuts (`t`, `m`, `l`) are hidden in a trailing hint at the end of the filter row, unlike the toolbar which prefixes each control with `[key]`. This makes the layout harder to scan and discoverability of filter shortcuts inconsistent.

## What Changes

- Add full-width horizontal divider lines between: toolbar, filter row, log viewport, and status bar (dedicated 1-row separators using `─`, DarkGray).
- Relabel filter controls to match toolbar style: `[t]Tag[value]`, `[m]Message[value]`, `[l]Level[value]`.
- Use empty brackets when Tag or Message filter is unset (e.g. `[t]Tag[]`); Level always shows the current minimum level (default Verbose).
- Remove the trailing `(t/m edit · l level · click Tag/Message)` hint; keep only `(click Tag/Message)` for mouse users.
- Update mouse hit regions in the filter row to match new label widths.

## Capabilities

### New Capabilities

_(none — visual and labeling changes extend existing TUI shell behavior)_

### Modified Capabilities

- `tui-shell`: main layout visual structure (section dividers); filter row label format and shortcut discoverability

## Impact

- `src/app.rs` — `draw()` layout constraints, new `draw_separator()`, `draw_filters()` label/spans/hit-map updates
- `README.md` — smoke checklist wording (optional); keyboard table unchanged (shortcuts already documented)
- No engine, filter logic, or dependency changes

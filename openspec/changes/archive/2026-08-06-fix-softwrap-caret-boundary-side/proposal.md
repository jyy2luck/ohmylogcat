## Why

When Soft-Wrap splits a log entry across display rows, the logical gap at a
wrap boundary can be displayed either after the previous row's last character
or before the next row's first character. The current implementation assigns
that gap only to the next row, so `End` on a non-final wrapped row stops to the
left of its last character and `Right` immediately jumps to the next row's
first-character gap. This does not match editor-style caret behavior.

## What Changes

- Track the caret's visual side at an internal Soft-Wrap boundary separately
  from its logical `LogPos`.
- Make `End` on a non-final wrap chunk place the caret after that chunk's last
  character, and make `Home` on a continuation chunk place it before the
  chunk's first character.
- Make `Right` and `Left` cross a wrap boundary in two stages: switch the
  visual side first, then advance or retreat the logical gap.
- Keep `Up` and `Down` aligned to the actual display column while honoring the
  boundary side and existing clamp behavior.
- Make screen-coordinate and visibility mapping boundary-side aware without
  changing logical selection, copy, or log contents.
- Preserve existing behavior for final chunks and Soft-Wrap-off lines.

## Capabilities

### New Capabilities

<!-- No new capability; this modifies log-display behavior. -->

### Modified Capabilities

- `log-display`: Define visual boundary-side semantics for Soft-Wrap caret
  movement and require editor-style End, Home, Left, Right, and vertical
  movement behavior at wrap boundaries.

## Impact

- `src/app.rs`: caret state and keyboard movement transitions.
- `src/ui/display.rs`: Soft-Wrap chunk and display-coordinate helpers.
- `src/ui/selection.rs`: caret screen mapping and related regression tests.
- `src/ui/mod.rs`: exports for the boundary-side type or helpers.
- No new dependencies and no changes to logical selection or clipboard data.

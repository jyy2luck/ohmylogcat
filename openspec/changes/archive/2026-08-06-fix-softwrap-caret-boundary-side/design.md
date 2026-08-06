## Context

The caret is stored as a logical `LogPos { row, col }`, where `col` is a gap
index. Soft-Wrap renders one logical line as multiple chunks, but the current
mapping assigns an internal chunk boundary gap only to the following display
row. This makes the same logical gap unable to represent both the end of the
previous row and the start of the next row.

The existing `caret_preferred_col` is already a display-column value used by
vertical movement. The change must preserve that meaning while adding only
the missing visual-side state. Selection ranges and clipboard extraction must
remain logical and must not acquire display-row affinity.

## Goals / Non-Goals

**Goals:**

- Represent both visual placements of an internal Soft-Wrap boundary gap.
- Implement the confirmed two-step Right/Left transitions across a boundary.
- Keep End/Home on the current display chunk.
- Preserve display-column-based Up/Down movement and clamp behavior.
- Keep Soft-Wrap-off and final-chunk behavior unchanged.
- Add deterministic unit coverage for movement and screen mapping.

**Non-Goals:**

- Do not change the logical `LogPos` ordering or selection model.
- Do not change copied text, find ranges, log formatting, or hanging-indent
  layout.
- Do not add support for literal newline characters inside a log message.
- Do not introduce a new dependency or a general text-editor cursor model.

## Decisions

### 1. Keep visual boundary state separate from `LogPos`

Add a small enum representing the visual side of an internal wrap boundary:

- `PreviousRow`: render the gap after the previous chunk's last character.
- `NextRow`: render the gap before the next chunk's first character.

Store this state on the application caret, not in `LogPos`. `LogPos` is used
by selection ordering, range extraction, mouse placement, and logical
cross-entry movement; adding visual state there would make those operations
depend on rendering details. The side is meaningful only when the logical
column is an internal chunk boundary; all other positions normalize to the
default `NextRow` behavior.

**Alternative rejected:** extending `LogPos` with a side field. This would
couple logical selection and clipboard semantics to soft-wrap rendering and
would require changing ordering and equality behavior.

### 2. Make wrap geometry side-aware while retaining existing defaults

Add side-aware variants around the existing display helpers:

- Find the chunk containing a column using the requested boundary side.
- Resolve a logical column to its display row and display column using that
  side.
- Convert a preferred display column on a target chunk back to a logical
  column and, when it lands on an internal edge, the corresponding side.

The existing side-less helpers retain their current downstream-boundary
semantics for generic display and mouse/selection operations. The log caret
uses the side-aware variants explicitly.

For `PreviousRow` at an internal boundary, screen mapping returns the previous
chunk at its right edge (`area.x + viewport_width`). For `NextRow`, it returns
the following chunk at its first content position, including its hanging
indent. This mirrors the existing line-end-gap handling in non-wrapped mode.

**Alternative rejected:** changing the global boundary ownership from
`NextRow` to `PreviousRow`. That would fix End but make continuation Home,
mouse placement, and existing vertical mapping inconsistent because they need
the following-row placement.

### 3. Implement explicit horizontal boundary transitions

`End` on a non-final chunk sets the logical column to `chunk_start +
chunk_len` and the side to `PreviousRow`. `Home` on a continuation chunk sets
the logical column to `chunk_start` and the side to `NextRow`.

Horizontal movement uses these transitions:

1. `Right` from `PreviousRow` at an internal boundary switches to `NextRow`
   without changing the logical column.
2. A subsequent `Right` advances the logical column by one.
3. `Left` from `NextRow` at an internal boundary switches to `PreviousRow`
   without changing the logical column.
4. A subsequent `Left` retreats the logical column by one.

Any logical horizontal move, mouse placement, find placement, filter reset, or
Soft-Wrap toggle normalizes the side unless the operation explicitly creates a
boundary placement. Visual-only transitions keep selection endpoints logical;
they do not add or remove selected characters.

### 4. Thread the side through vertical movement and visibility

Vertical movement first resolves the current display chunk using the current
boundary side. It then uses the existing preferred display column to resolve a
target position. If the target display column is exactly an internal chunk
edge, the result uses `PreviousRow`; if it is a continuation start, it uses
`NextRow`. `caret_preferred_col` is updated from the side-aware screen column
after the move, preserving the existing clamp rule.

The same side-aware position must be used by caret screen mapping,
`ensure_caret_visible`, and above/below viewport checks. Otherwise an End
position could be drawn on one row while scrolling logic treats it as being on
the other row.

### 5. Keep logical interaction APIs unchanged

Mouse hit testing continues to return logical `LogPos` values and initializes
the caret on the default following-row side. Selection highlighting, find
matching, and clipboard extraction continue to consume only logical ranges.
Only the terminal caret's rendering and keyboard navigation use the added side.

## Risks / Trade-offs

- [The terminal cursor may be positioned one cell past a full non-final chunk]
  → Reuse the existing line-end-gap behavior already supported for nowrap and
  add screen-mapping tests for the right-edge coordinate.
- [A stale side could survive a resize or re-wrap and refer to a non-boundary]
  → Normalize the side whenever the caret is clamped, the viewport changes, or
  Soft-Wrap is toggled.
- [Shift plus a visual-only Left/Right transition could accidentally alter a
  logical selection] → Keep selection endpoints unchanged when only the side
  changes and test the zero-logical-distance transition.
- [Existing tests encode downstream-only boundary ownership] → Update those
  expectations only where the new user-visible contract changes them, while
  retaining generic mouse/selection mapping tests.

## Migration Plan

This is an in-memory behavior change with no persisted data migration. Implement
the side state and mapping, update the delta tests, then run the full Rust test
suite. Rollback consists of removing the side state and restoring the previous
downstream-only boundary mapping.

## Open Questions

None. Boundary behavior, horizontal transition order, and vertical display
column semantics were confirmed before implementation.

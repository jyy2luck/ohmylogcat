## Context

See `proposal.md` — Why. `OhmylogcatApp::draw()` in `src/app.rs` currently splits `frame.area()` directly into vertical chunks with no outer margin. All chrome and the log viewport render from `(0,0)` through the last terminal column and row. Hit regions (`HitMap`), `viewport_width` / `viewport_height`, and soft-wrap width all derive from the log chunk `Rect` passed to `draw_logs()`. Mouse text selection maps coordinates via `hit_map.log_viewport` and `ViewportMap` in `src/ui/selection.rs`.

## Goals / Non-Goals

**Goals:**

- Add a uniform outer inset around the entire main shell on terminals large enough to afford it.
- Keep dividers, toolbar, filters, logs, and status aligned within the inset content area.
- Preserve accurate mouse hit regions and log selection mapping after inset.
- Reduce or disable inset on very small terminals.

**Non-Goals:**

- Per-region padding (e.g. logs-only inset) — uniform shell inset only.
- Block borders around individual sections — keep existing dedicated separator rows.
- Changing modal visual style beyond using full terminal area for centering.
- User-configurable padding values or settings toggle.
- Changing log filtering, streaming, or selection semantics.

## Decisions

### 1. Uniform outer inset via nested Layout (方案 A)

Apply inset once at `draw()` entry by splitting `frame.area()` into gutter + content:

```
Vertical:   [top gutter 0–1] [content Min] [bottom gutter 0–1]
Horizontal: [left 0–1] [content] [right 0–1]
```

All existing vertical chunk splits (toolbar, separators, filters, find, logs, status) operate on the inner content `Rect`.

**Alternative considered:** horizontal-only inset — rejected; user asked for spacing on all four sides, especially for selection at top/bottom edges.

**Alternative considered:** logs-only inset — rejected; inconsistent alignment with toolbar/filters and weaker chrome click ergonomics.

### 2. Default inset: 1 column, 1 row per side

| Terminal condition | Horizontal inset (each side) | Vertical inset (each side) |
|--------------------|------------------------------|----------------------------|
| width ≥ 60 and height ≥ 15 | 1 | 1 |
| otherwise | 0 | 0 |

Thresholds chosen so 80×24 keeps inset (content ~78×22); very cramped terminals fall back to current edge-to-edge behavior.

**Alternative considered:** always 1 inset regardless of size — rejected; 40×12 would lose too much log area.

### 3. Dividers span inset content width

`draw_separator()` already receives the content-area `Rect`; no full-terminal repeat. Dividers visually align with toolbar/logs left and right edges.

### 4. Modals use full `frame.area()`

`draw_modal()` continues to receive the full terminal `area`, not the inset shell. Popups stay visually centered in the terminal; no change to `centered_rect()` math beyond passing full area as today.

### 5. Hit map and viewport derive from inset rects

No manual coordinate offset patches — each `draw_*` function receives already-inset `Rect`s, so `hit_map` entries and `log_viewport` bounds stay consistent. `viewport_width` / `viewport_height` in `draw_logs()` automatically reflect reduced dimensions; soft-wrap and selection mapping follow.

Gutter clicks: outside `log_viewport` → existing `try_start_log_selection` / clear-selection paths; no new hit targets in gutter.

### 6. Helper function

```rust
fn shell_content_area(area: Rect) -> (Rect, u16, u16) {
    // returns (content_rect, pad_x, pad_y) with adaptive pad
}
```

Used only in `draw()`; pad values available if needed for tests.

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| 2 rows + 2 columns lost on inset terminals | Adaptive disable below threshold; acceptable on 80×24+ |
| Separator rows no longer full terminal width | Intentional; aligns with inset shell panel look |
| Regression in mouse selection at edges | Manual smoke: drag select first/last column and top/bottom log rows |
| `Constraint::Min(3)` logs chunk too small after inset | Threshold disables inset when height < 15 |

## Migration Plan

Single change; no settings migration. Users see inset on next launch when terminal is large enough. Rollback: revert `draw()` inset wrapper only.

## Open Questions

_None — explore session confirmed uniform 1-cell inset with adaptive fallback._

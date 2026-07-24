# Design: UI section dividers and filter shortcut labels

## Context

`OhmylogcatApp::draw()` in `src/app.rs` splits the terminal into five vertical chunks: toolbar (1), filters (1), optional find bar (0–1), logs (Min), status (1). Each region renders as a plain `Paragraph` or `List` with no borders or separators. The toolbar uses `[key]Label` spans with bold shortcut keys; the filter row uses `Tag:[value]` labels and a trailing prose hint for shortcuts.

## Goals / Non-Goals

**Goals:**

- Visually separate the four main chrome zones (toolbar, filters, logs, status) with horizontal rules.
- Align filter row labeling with toolbar convention (`[t]Tag[]`, `[m]Message[]`, `[l]Level[Verbose]`).
- Preserve all existing keyboard and mouse behavior; only presentation changes.

**Non-Goals:**

- Restyling the find bar (`draw_find`) — out of scope unless a divider is needed above logs when find is open.
- Block-based borders instead of dedicated separator rows.
- Dim/gray styling for empty Tag/Message brackets (user chose plain empty `[]`).
- Changing filter semantics or Level focus behavior.

## Decisions

### 1. Dedicated separator rows (方案 A)

Insert `Constraint::Length(1)` rows between major sections and render a full-width `─` line in `DarkGray`.

**Layout (find closed):**

```
toolbar     (1)
separator   (1)
filters     (1)
separator   (1)
logs        (Min)
separator   (1)
status      (1)
```

**Layout (find open):** find bar sits between the filters separator and logs; no extra divider around find (same as today — find is a transient overlay row).

**Alternative considered:** `Block` top borders on logs/status — rejected because toolbar/filter boundaries would lack lines and dividers would be inconsistent.

### 2. Filter label format

| Control | Empty / default | With value |
|---------|-----------------|------------|
| Tag | `[t]Tag[]` | `[t]Tag[myapp]` |
| Message | `[m]Message[]` | `[m]Message[error]` |
| Level | `[l]Level[Verbose]` | `[l]Level[Warn]` |

- Shortcut keys `[t]`, `[m]`, `[l]` rendered **bold** (match toolbar).
- Level segment uses existing `field_style` when `Focus::Level`.
- Trailing hint reduced to `(click Tag/Message)` only.

**Empty brackets:** no placeholder text inside `[]` when unset — avoids ambiguity with literal filter values such as `-` or `all`.

### 3. Hit map

Recalculate `filter_tag`, `filter_message`, and `filter_level` rect widths from the new label strings (prefix adds 4 chars per control: `[x]`). Click targets remain on the Tag and Message summary segments only.

### 4. Helper function

```rust
fn draw_separator(frame: &mut Frame, area: Rect) {
    let line = "─".repeat(area.width as usize);
    frame.render_widget(
        Paragraph::new(line).style(Style::default().fg(Color::DarkGray)),
        area,
    );
}
```

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| Three separator rows reduce log viewport height by 3 lines on small terminals | Acceptable trade-off for readability; no config toggle planned |
| Wider filter labels may truncate sooner on narrow terminals | Keep existing `truncate_input` limits on bracket contents |
| Hit regions shift after label change | Update width math in `draw_filters` in same change |

## Migration Plan

Single PR; no settings migration. Users see updated layout on next launch.

## Open Questions

_None — decisions confirmed in explore: 方案 A dividers, empty brackets, keep `(click Tag/Message)`._

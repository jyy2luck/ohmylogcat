# Design: Log viewport text selection

## Approach

Keep `EnableMouseCapture` so toolbar clicks and wheel scroll continue to work. Add application-level selection state mapped from screen coordinates to `(filtered_row, char_col)` in the formatted log line.

## Selection model

```rust
LogPos { row: usize, col: usize }  // filtered index + char offset in format_log_line()

TextSelection {
    anchor: Option<LogPos>,
    cursor: Option<LogPos>,
    dragging: bool,
}
```

Range is normalized (min/max). Selection is **inclusive** on both ends per character index within each formatted line. Multi-line copy joins lines with `\n`.

## Coordinate mapping

| Mode | Row | Column |
|------|-----|--------|
| No wrap | `scroll_offset + (mouse.y - viewport.y)` | `col_offset + (mouse.x - viewport.x)`, clamped to line length |
| Soft wrap | Walk visible wrapped display rows from `scroll_offset`/`wrap_skip` | chunk_start + in-chunk offset (logical line index) |

Soft-wrap selection follows **logical log entry** text, not isolated display rows.

## Mouse handling

| Event | Log viewport | Chrome (toolbar/filters/status) |
|-------|--------------|----------------------------------|
| Moved | SteadyBar cursor | DefaultUserShape cursor |
| Down | Start selection + focus Logs | Clear selection; existing click actions |
| Drag | Extend selection | — |
| Up | End drag | — |

Scroll wheel unchanged. Drag does not scroll.

## Copy

When selection is active, Cmd+C / Ctrl+C copies selected plain text via `arboard` and consumes the key event (does not quit). Without selection, Ctrl+C remains non-quit per existing spec.

## Rendering

Split visible spans at selection boundaries; selection style uses blue background (distinct from find yellow). Priority: selection > find current > find match > level color.

## Invalidation

Clear selection on: filter apply, buffer clear, dropped-front compaction, click outside log viewport.

## Files

- `src/ui/selection.rs` — state, mapping, span helper, text extraction
- `src/app.rs` — mouse/key integration, draw_logs highlight
- `src/main.rs` — restore default cursor on exit
- `Cargo.toml` — `arboard`

## Context

See `proposal.md` for motivation. Today:

- `format_log_line` emits unpadded `{ts} {pid} {tid} {L} {tag}: {message}`
- Soft-Wrap uses equal-width `WrapChunks` / `wrap_line_count` (`src/ui/display.rs`)
- Paint (`draw_logs`), scroll height, `wrap_skip`, mouse hit-test, and caret mapping all assume `chunk_index * viewport_width` logical offsets
- Export already pads PID/TID with `{:5}` but UI does not

Hang indent and column pad touch the same wrap math surface; they should ship together so paint and interaction stay coherent.

## Goals / Non-Goals

**Goals:**

- Single shared hang-indent wrap model used by paint, row-height, scroll, mouse, and caret
- Message-column indent derived from the same formatting as the visible line
- PID/TID width-5 padding in the formatted logical line (find/copy/selection see the pads)

**Non-Goals:**

- Hard `\n` inside messages as forced display breaks (stack traces stay one logical line; soft-wrap may still cut mid-line)
- Word-boundary / CJK-aware wrap (keep char-chunk wrap)
- True multi-widget column layout (still one formatted string + display pad)
- Changing Soft-Wrap default (remains off)

## Decisions

### 1. Pad in `format_log_line`, width 5

**Choice:** Change `format_log_line` to `"{ts} {:5} {:5} {L} {tag}: {message}"` (right-aligned decimal, space-padded), matching export’s PID/TID field width. Keep single-char level in the UI line.

**Rationale:** One formatted string keeps Find, selection, caret, and copy consistent. Width 5 matches `logcat -v threadtime` / current export.

**Alternatives:** Pad only at paint time — rejected (find/copy would diverge). Width 6 for 6-digit PIDs — rejected for now; `{:5}` overflows naturally for larger values and still keeps typical devices aligned.

### 2. Message-column indent = prefix length through `": "`

**Choice:** Indent = character count of `"{ts} {:5} {:5} {L} {tag}: "` for that entry (everything before `message`). Continuations visually start under the first message character.

**Rationale:** Matches Android Studio-style “hang under message” (option C from exploration). Tag length varies per entry; indent is per-line, not a global column.

**Alternatives:** Align under Level or Tag — closer to metadata, less like AS. Fixed global indent — wrong for long tags.

### 3. Shared hang-indent wrap API in `display.rs`

**Choice:** Replace or extend `WrapChunks` / `wrap_line_count` with a hang-indent-aware API, e.g.:

- Inputs: logical line `&str`, `viewport_width`, `indent`
- First chunk capacity = `W`; later chunk capacity = `W - indent` (when `indent < W`)
- Iterator yields `(logical_char_start, logical_chunk_text)` 
- Display string for continuation = `" ".repeat(indent) + chunk` (paint only)
- If `indent >= W` or `indent == 0`, behave like today’s equal-width wrap

**Rationale:** Today’s bug class is paint and hit-test drifting; one iterator kills that.

**Alternatives:** Only pad spaces in `draw_logs` — rejected (scroll height and mouse stay wrong).

### 4. Hit-test / caret: pad is not logical text

**Choice:**

- Logical `LogPos.col` indexes into the formatted line **without** hang spaces
- Click in continuation pad → map to `logical_char_start` of that chunk
- Click on content → `logical_char_start + offset_within_chunk`
- `log_pos_to_screen` adds `indent` to x for continuation chunks

**Rationale:** Spec requires display-only pad; selection/copy must not invent spaces.

### 5. Scroll and `wrap_skip` use new row counts

**Choice:** `entry_wrap_height` and wrap scroll/follow paths call the new `wrap_line_count(..., indent)` so `wrap_skip` indexes hang-aware display rows.

**Rationale:** Follow-to-bottom and page scroll already depend on accurate heights.

## Risks / Trade-offs

- **[Risk] Missed call site still uses old equal-width math** → Mitigation: centralize in `display.rs`; grep for `WrapChunks` / `wrap_line_count` / `abs_ci * width`; unit-test paint-equivalent offsets vs mouse mapping.
- **[Risk] Very long tags make indent ≈ viewport** → Mitigation: fallback to indent 0 when `indent >= W`.
- **[Risk] PID/TID > 5 digits break cross-row column alignment** → Accept; rare on typical devices; document as same behavior as `{:5}` export.
- **[Trade-off] Char-based width, not Unicode display width** → Keeps consistency with current wrap; wide CJK glyphs may still visually misalign (pre-existing).

## Migration Plan

- No settings migration; Soft-Wrap preference unchanged.
- Users see padded columns and hang indent on next run when Soft-Wrap is on.
- Rollback: revert format + display wrap helpers and call sites.

## Open Questions

None that block implementation. Hard-newline display breaks can be a follow-up change if stack-trace readability still hurts after hang indent.

## Context

See proposal.md for motivation. Today the log viewport uses `TextSelection { anchor, cursor }` for mouse drag ranges, `mouse_to_log_pos` / `ViewportMap` for coordinate mapping, and `copy_selection` via `arboard`. Mouse-up with a non-empty range calls `copy_selection` (behavior beyond the written specs). Arrow / `j`/`k` / Page / Home / End in `handle_logs_key` only adjust `scroll_offset` / `col_offset`. Terminal blinking-bar cursor positioning already exists for Tag/Message/Find via `frame.set_cursor_position` and `TEXT_INPUT_CURSOR_STYLE`.

## Goals / Non-Goals

**Goals:**

- One caret (`LogPos`) driving keyboard navigation, selection live end, and hardware cursor placement
- Wrap-aware caret movement and ensure-visible after keyboard moves only
- App-level multi-click detection for word/line select
- Remove mouse-up auto-copy; unify clipboard on Ctrl/Cmd+C
- Drop log-viewport `j`/`k` bindings (Settings modal `j`/`k` unchanged)

**Non-Goals:**

- `Ctrl+Home` / `Ctrl+End` buffer jump
- Path-friendly word boundaries (dots/slashes glue)
- Optional auto-copy preference
- Visual-line vs logical-line preference toggle
- Changing find-match navigation or filter text-input caret behavior

## Decisions

### 1. Caret state separate from selection endpoints

**Choice:** Store `caret: Option<LogPos>` (or always-valid with empty-buffer sentinel) on `App`, independent of `TextSelection`. Selection continues to use `anchor` + `cursor`; when a selection is active, `caret` tracks the live end (`cursor`). Click without drag sets caret and clears selection.

**Rationale:** Matches editor model; avoids overloading “no selection” with a missing cursor. Empty filtered list: hide hardware cursor / no-op moves.

**Alternative:** Reuse `TextSelection.cursor` as caret always — rejected; collapsed selection vs no selection becomes ambiguous for drawing and Shift-start.

### 2. Hardware cursor for log caret

**Choice:** When `Focus::Logs` and no text-input overlay owns the cursor, map caret `LogPos` → screen `(x,y)` (inverse of `mouse_to_log_pos`, wrap-aware) and call `frame.set_cursor_position`. Enable `BlinkingBar` while log focus owns the caret (extend `sync_terminal_cursor_style` / `text_input_focused` logic so log caret and text inputs do not fight).

**Rationale:** Same approach as text inputs; native blink.

**Alternative:** Paint a reverse-video cell — rejected for v1 (no native blink; more draw complexity).

### 3. Movement semantics

**Choice:**

| Key | Behavior |
|-----|----------|
| Left/Right | ±1 char on logical line; at line start Left → previous line end; at line end Right → next line start |
| Up/Down | Move by one **display row** (wrap chunk when Soft-Wrap on; one logical row when off), preserve preferred column when possible |
| Home/End | Logical line start / end |
| PageUp/Down | Move caret by `viewport_height` display rows, then ensure-visible |
| Shift+… | Before move: if no selection, `anchor = caret`; after move: `selection.cursor = caret` |
| Plain move | Clear selection |

**Rationale:** Display-row Up/Down matches editor wrap navigation; Page distance matches current page scroll magnitude but tied to caret.

**Alternative:** Up/Down always logical entries — simpler but feels wrong with Soft-Wrap on.

### 4. Ensure-visible policy

**Choice:** `ensure_caret_visible()` runs only after keyboard caret moves (arrows, Home/End, Page*, and Shift variants). Wheel, follow/auto-scroll, and filter list churn may leave caret off-screen. Next keyboard move pulls viewport to caret.

**Rationale:** User-approved; avoids fighting follow mode and wheel browsing.

### 5. Multi-click in `handle_mouse`

**Choice:** Track `last_click_time`, `last_click_pos`, `click_count` on left Down inside log viewport. Threshold ~500ms and small cell tolerance. Count 1→caret, 2→expand word, 3→expand line, 4→reset to 1. Word expand: scan `[A-Za-z0-9_]` around click col; non-word → single char. Line expand: col 0 .. line_len-1 (or exclusive end consistent with existing inclusive `TextSelection` — keep inclusive endpoints as today).

**Rationale:** crossterm has no DoubleClick event.

**Alternative:** Rely on terminal select — impossible under `EnableMouseCapture`.

### 6. Clipboard: explicit only

**Choice:** Delete mouse-up `copy_selection()` call. Keep Ctrl/Cmd+C path and status message.

**Rationale:** Aligns keyboard and mouse; matches proposal.

### 7. Initial caret placement

**Choice:** On first need (non-empty list, no caret yet): place caret at first visible logical line, column 0 (or last focused line if still valid after filter). After clear/empty→non-empty, re-seed similarly. Invalid caret after filter shrink: clamp row/col into range.

**Rationale:** Predictable; avoids surprising jump to buffer end while follow moves the viewport independently.

## Risks / Trade-offs

- **[Risk] Caret screen mapping bugs with Soft-Wrap + `wrap_skip`** → Mitigation: share math with `mouse_to_log_pos` (extract inverse helper); unit tests for wrap edges.
- **[Risk] Hardware cursor conflicts with Find/modal** → Mitigation: single owner in draw/sync path; logs yield when text input focused.
- **[Risk] Users relied on `j`/`k` and Home/End buffer jump** → Mitigation: document in changelog/status help if present; Ctrl+Home/End deferred.
- **[Risk] Inclusive selection end differs from some editors’ exclusive caret** → Mitigation: keep existing inclusive `extract_text` / `contains` semantics for consistency with mouse drag.
- **[Trade-off] Preferred-column Up/Down** adds state but avoids “staircase” when moving through short lines — worth the small field.

## Migration Plan

Ship in one release. No persisted preference changes. Behavior breaks (`j`/`k`, Home/End meaning, mouse-up copy) are intentional; no rollback flag.

## Open Questions

None that block implementation. Page distance is defined as `viewport_height` display rows (same magnitude as today’s page scroll).

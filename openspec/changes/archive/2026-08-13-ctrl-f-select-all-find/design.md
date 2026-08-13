## Context

See proposal.md for motivation. Ctrl/Cmd+F is handled only in `handle_logs_key`; `Focus::Find` and `Focus::Level` ignore it. `FindState::open_bar` always places the insertion cursor at the end of the query. `TextInput` has `text` + `cursor` and no selection. `/` and Ctrl+F both call `open_find()` today.

## Goals / Non-Goals

**Goals:**

- Intercept Ctrl/Cmd+F once, above per-focus dispatch, whenever no modal is open
- Select-all on that shortcut when the Find query is non-empty, including when Find already has focus
- Replace-on-type / clear-on-Backspace while selected; Enter does not collapse
- Visible selection of the query inside `Find:[query]` only
- Keep `/` on the log viewport as open-or-refocus with cursor at end and `select_all` cleared

**Non-Goals:**

- Full in-field selection ranges (Shift+arrows, Ctrl+A) on Tag / Message / Export / Settings
- Remembering the last query after Esc closes Find
- Changing `/` while Find is focused (it stays a literal character)

## Decisions

### 1. Global Ctrl+F intercept, not a `handle_find_key` special case

**Choice:** In `handle_key`, after the copy-shortcut and quit checks, if `self.modal.is_none()` and the event is Ctrl+F or Cmd+F, call a dedicated activate path (`open` if needed, `focus = Find`, `select_all`) and return. Do this before the `Focus` match so Logs, Find, and Level all hit it. If a modal is open, fall through to existing modal handling.

**Rationale:** One path; Find-already-focused is the same as Logs/Level. Matches copy-shortcut style.

**Alternative:** Duplicate Ctrl+F in `handle_find_key` and `handle_level_key` — rejected as easy to miss.

### 2. Boolean `select_all` on `TextInput`, used only by Find

**Choice:** Add `select_all: bool` to `TextInput`. `select_all()` sets the flag (no-op visually when `text` is empty). `handle_key` then:

- printable (no Ctrl/Cmd) + `select_all` → clear text, insert the char, `select_all = false`
- Backspace/Delete + `select_all` → clear text, cursor 0, `select_all = false`
- Left/Home + `select_all` → cursor 0, `select_all = false`
- Right/End + `select_all` → cursor at end, `select_all = false`
- otherwise existing insert/move behavior, and any of those keys also clear the flag if it was set

Filter modals keep using `TextInput` but never set the flag.

**Rationale:** Find only needs “all or nothing.” A real `(anchor, cursor)` range is unused scope.

**Alternative:** `FindState`-only flag with duplicate key logic in `handle_find_key` — rejected; `handle_key` already owns editing.

### 3. Do not fold select-all into `open_bar`

**Choice:** `open_bar` stays “open + cursor at end + `select_all = false`” (`/` and any other open path). Ctrl+F calls open-if-needed then `input.select_all()`.

**Rationale:** Keeps `/` behavior identical without a second open API.

### 4. Draw selected query with existing selection colors

**Choice:** In `draw_find`, split the line into prefix / query / suffix spans. When `focus == Find` and `select_all` and query is non-empty, style the query span with `theme.selection_fg` / `selection_bg`. Hardware cursor stays at the end of the query while selected.

**Rationale:** Reuses the selection palette; only the text inside the brackets inverts.

**Alternative:** Invert the whole Find row — rejected; chrome (counter, help) must stay unselected.

### 5. Mouse click already in Find input collapses

**Choice:** Existing click-to-cursor in `find_input` also sets `select_all = false`.

## Risks / Trade-offs

- **[Risk] Ctrl+F while Find is focused currently no-ops, so tests may not cover it** → Mitigation: unit-test the activate path for Logs, Find, and Level; test `TextInput` select-all replace/collapse.
- **[Risk] Whole-row Find focus style hides the inner selection** → Mitigation: selection colors only on the query span; keep prefix/suffix on the existing field style.
- **[Risk] Some terminals swallow Ctrl+F** → Accepted (existing); Cmd+F / `/` remain.

## Migration Plan

No data migration. Behavior change only for Ctrl/Cmd+F. `/` and Esc-close-clears-query stay as they are.

## Context

The TUI currently treats Ctrl+C as a global quit (because raw mode swallows SIGINT) and documents `q` / Ctrl+C together in README. Tag and Message filters are edited inline in the filter row via `Focus::Tag` / `Focus::Message`, which forces every letter shortcut (`q`, `c`, `f`, …) to be suppressed or special-cased while typing. Windows users especially associate Ctrl+C with copy, not quit.

Existing modal patterns (Devices, Export, Settings) already isolate keyboard input from the main shell. Find remains an inline overlay row and stays unchanged in this change.

## Goals / Non-Goals

**Goals:**

- Top-layer-only quit via `q`; remove Ctrl+C quit binding.
- Consistent overlay rule: when any modal is open or Find is open, `q` never quits the application.
- Tag/Message filter editing via centered modal overlay with live filter apply on each keystroke; Esc only dismisses the modal (apply logic is independent).
- Toolbar shows `[q]Quit` alongside existing labeled shortcuts.
- Level filter stays inline (unchanged).

**Non-Goals:**

- Converting Find to a modal (remains inline).
- Esc-to-revert / snapshot-on-open for filter edits.
- Process-level SIGINT handler as a separate quit path.
- Changing Level, Find, Export, or Settings behavior beyond shared overlay quit rules.

## Decisions

### 1. Layer model for quit routing

**Choice:** Define `is_top_layer = modal.is_none() && !find.open`. Only when `is_top_layer`, unmodified `q` / `Q` sets `should_quit`.

**Overlay `q` behavior:**

| Overlay | `q` behavior |
|---------|----------------|
| Filter Tag/Message modal (text) | Insert character `q` |
| Find bar open (text) | Insert character `q` (unchanged) |
| Export path / Settings text fields (text) | Insert character `q` (unchanged) |
| Devices, Export menu, Settings preset field (non-text) | No effect |
| Any overlay | Never quit |

**Esc behavior:** Close the active overlay and return focus to log viewport. Does not revert filter values already applied live.

**Alternatives considered:**

- `is_text_input_active()` exception list on inline fields — rejected; modal layer is cleaner.
- `q` closes non-text modals — rejected; user chose Esc-only for consistency.

### 2. Remove Ctrl+C quit

**Choice:** Delete the global Ctrl+C → `should_quit` handler in `handle_key`. Document quit as top-layer `q` only.

**Rationale:** Windows copy muscle memory; toolbar already uses `[c]Clear`. Raw-mode SIGINT swallowing is not worth a conflicting binding.

**Alternatives considered:** Keep Ctrl+C on Unix only — rejected for cross-platform consistency.

### 3. Tag/Message filter modal

**Choice:** Add `ModalKind::FilterEdit { field: FilterField::Tag | Message }`. Opening triggers:

- Keyboard: `t` / `m` from log viewport (top layer).
- Mouse: click Tag or Message summary in filter row.

Modal UI (centered popup, same pattern as Export path):

```
┌─ Tag filter ─────────────────────┐
│ Tag contains: [_______________]  │
│ Live filter · Esc done           │
└──────────────────────────────────┘
```

Each character keystroke updates `filter_tag` or `filter_message` and calls existing `mark_filter_dirty()` debounce → `apply_filter()`. Esc sets `modal = None`, `focus = Logs`.

Filter row becomes read-only summary (`Tag:[value]`, truncated for display); no `Focus::Tag` / `Focus::Message`. Remove `handle_text_field_key` paths for filters and Tab-cycle through Tag/Message inline focus (Tab may still cycle Level or return to Logs per simplified model).

**Alternatives considered:** Inline editing with typing-context exceptions — rejected in exploration.

### 4. Toolbar quit hint

**Choice:** Append `[q]Quit` as the last toolbar label (display only; not a separate hit target required).

### 5. Focus enum simplification

**Choice:** Remove `Focus::Tag` and `Focus::Message`. Focus targets: `Logs`, `Level`, `Find`, `Modal`. Filter editing always goes through `Focus::Modal`.

## Risks / Trade-offs

- **[Risk] Users habituated to Ctrl+C quit** → Mitigation: README and toolbar `[q]Quit`; Windows smoke checklist already references `q`.
- **[Risk] Devices modal requires Esc then `q` to exit app** → Mitigation: Document overlay rule; Esc hint already on modals.
- **[Risk] Extra step for filter edit (open modal, Esc close)** → Mitigation: Live preview behind modal; filter editing is not high-frequency streaming input.
- **[Risk] Tab key behavior change when Tag/Message inline removed** → Mitigation: Update filter row hint text; keep Level inline behavior.

## Migration Plan

Single release; no data migration. README keyboard table updated. Breaking: Ctrl+C no longer quits; Tag/Message editing UX changes from inline to modal.

## Open Questions

None — overlay quit rules and Esc semantics confirmed in exploration.

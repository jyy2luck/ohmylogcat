## Context

Settings modal in `src/app.rs` uses `focus_field: usize` (0–3) with Tab/BackTab navigation and `[` `]` / Left/Right to cycle preset and theme. Devices modal already uses Up/Down list navigation. The Custom capacity row is conditionally rendered when preset is Custom, but focus index 2 remains reachable when the row is hidden—a latent UX bug.

See proposal.md for motivation.

## Goals / Non-Goals

**Goals:**

- ↑/↓ and j/k move focus among **visible** settings rows (ADB, Preset, Custom when shown, Theme)
- ←/→ and h/l cycle preset and theme when those rows are focused
- Text rows (ADB, Custom): append-only typing and Backspace; Left/Right no-op
- Single help line at modal top listing move · adjust · type · Enter · Esc
- Re-anchor focus when preset change hides Custom row

**Non-Goals:**

- Wiring Settings text fields to `TextInput` insertion cursor (separate change)
- Mouse click-to-focus on settings rows
- Tab/BackTab or `[` `]` compatibility aliases (removed to simplify the model)

## Decisions

### 1. Field identity enum instead of raw index

**Choice:** Introduce `SettingsField { Adb, Preset, Custom, Theme }` (or equivalent) plus helpers `visible_fields(preset) -> &[SettingsField]` and `move_focus(delta)`.

**Rationale:** Custom row visibility is dynamic; enum + visible list prevents focus on hidden rows and simplifies render/focus marking.

**Alternative:** Keep `usize` with skip logic — works but easy to desync render and focus.

### 2. Horizontal keys only adjust cycle fields

**Choice:** Left/Right/h/l call `cycle_preset` / `cycle_theme` only when Preset or Theme is focused; ignored on text fields.

**Rationale:** Matches user mental model ("左右调整设置项"); text fields stay append-only per existing design.

### 3. Help line format

**Choice:** One line, English, consistent with other modals:

`↑/↓ move · ←/→ adjust · type text · Enter save · Esc cancel`

Include j/k and h/l in the same line if space allows, e.g. `(j/k h/l)`.

**Alternative:** Localized Chinese — rejected; rest of TUI chrome uses English shortcut hints.

### 4. Preset change re-anchors focus

**Choice:** After `cycle_preset`, if focus was Custom and preset is no longer Custom, set focus to Preset (the row that was just adjusted).

**Rationale:** Avoids invisible focus; user context stays on the control they were editing.

### 5. Remove Tab and bracket keys

**Choice:** Drop Tab/BackTab and `[` `]` handlers from Settings modal entirely.

**Rationale:** Single navigation model; top help line is the source of truth.

## Risks / Trade-offs

- **[Risk] Users muscle-memorized Tab in Settings** → Mitigation: help line on every open; change is low-frequency surface.
- **[Risk] vim users expect h/l to move focus vertically in some TUIs** → Mitigation: help line states h/l = adjust; j/k = move (matches log viewport).
- **[Risk] Custom row show/hide edge cases** → Mitigation: centralize visibility in `visible_fields`; add manual test matrix in tasks.

## Migration Plan

No data or settings file migration. Ship in one release. Behavior change is Settings-modal-only.

## Open Questions

_(none)_

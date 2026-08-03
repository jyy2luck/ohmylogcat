## Context

See `proposal.md` — Why. Current code already live-applies Tag/Message filters via debounced `apply_filter`, but filter Esc closes immediately and Enter does nothing. Settings cycles update `settings_panel` only; `save_settings_panel` runs on Enter and Esc discards panel state. Language and theme do not update runtime until save.

## Goals / Non-Goals

**Goals:**

- Progressive Esc and Enter dismiss for Tag/Message filter modals.
- Settings horizontal adjust and text edits apply to runtime + persist immediately.
- Enter and Esc both dismiss Settings without save/cancel semantics.
- Update i18n help strings in all three locales.

**Non-Goals:**

- Change Find bar, Devices, Export, or Export path modal keyboard behavior.
- Auto-reconnect adb stream when adb path changes mid-session.
- Undo/revert for accidental settings changes.

## Decisions

### 1. Filter modal Esc: progressive dismiss

**Choice:** Non-empty → clear input + `mark_filter_dirty`; empty → `close_modal`.

**Alternatives:** Esc always clear (no one-key exit); Esc always close (current). Progressive matches Chrome/IDE search UX and preserves one-key exit when already empty.

### 2. Filter modal Enter: close only

**Choice:** `KeyCode::Enter => close_modal()` — filter already live.

**Alternatives:** Enter submits then closes (redundant).

### 3. Settings apply: shared `commit_settings_field` helper

**Choice:** Extract logic from `save_settings_panel` into a helper that builds `Settings` from panel + runtime prefs (`auto_scroll`, `soft_wrap`), applies side effects, and calls `save_settings`:

| Field change | Runtime effect | Persist |
|--------------|----------------|---------|
| Theme cycle | `self.theme = Theme::resolve(...)` | immediate |
| Language cycle | `self.locale`, `self.ui = UiStrings::for_locale(...)` | immediate |
| Preset / custom capacity | `engine.set_capacity(...)` | immediate |
| Adb path text | update `settings.adb_path`, optional validation hint in panel status | immediate on each text change |

**Alternatives:** Persist only on modal close — rejected; user chose immediate effect including persist.

### 4. Settings text fields: persist on every text change

**Choice:** After Backspace/char edit in adb or custom capacity, call commit helper. No debounce initially — settings text edits are low frequency.

**Alternatives:** Debounce 300ms — defer unless typing feels laggy.

### 5. Settings Enter/Esc: both `close_modal` only

**Choice:** Remove `save_settings_panel` from Enter; Esc no longer discards. `open_settings` still resets panel from `self.settings` on each open.

**Alternatives:** Keep Enter as explicit save — rejected per user decision.

### 6. Explicit focus model exception

Filter modals are the only overlay where first Esc may not close. Update help text; no change to Find/Devices/Export Esc behavior.

## Risks / Trade-offs

- **[Accidental preset/capacity change]** → User must cycle back; acceptable for live-preview model aligned with Level control.
- **[Buffer shrink drops logs immediately]** → Existing `set_capacity` behavior; status message optional, not blocking.
- **[Adb path change while streaming]** → Path saved but stream uses prior resolved path until reconnect; document in code comment, no auto-reconnect in this change.
- **[Esc overlay spec generality]** → Delta spec narrows exception to filter modals only.

## Migration Plan

Single release; no data migration. Existing settings file format unchanged. Users learn new Esc/Enter behavior via updated help lines.

## Open Questions

_(none — user confirmed progressive Esc and immediate settings apply)_

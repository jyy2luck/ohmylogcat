## Context

See proposal.md for motivation. Today `SettingsPanelState` always treats ADB path as a free-typing field: focus defaults to `SettingsField::Adb`, and every `Char`/`Backspace` immediately calls `commit_settings_from_panel`. Empty `adb_path` already means Auto via `resolve_adb_path(None)`; the panel shows an auto hint when empty. No file/schema migration is required.

## Goals / Non-Goals

**Goals:**

- Add a locked/edit mode flag for ADB path only inside Settings panel state
- Route `e` / `r` / Esc / typing according to that flag
- Surface Auto vs Custom and the effective path clearly in the ADB row
- Update the Settings help line / i18n strings so bindings match behavior

**Non-Goals:**

- File picker / browse dialog for adb
- Changing `resolve_adb_path` semantics or settings JSON shape
- Guarding Custom capacity the same way
- Auto-reconnect of an active stream when path changes (existing behavior remains)

## Decisions

### 1. Boolean edit flag on panel state

Store `adb_editing: bool` on `SettingsPanelState`, default `false` when opening/rebuilding the panel from settings.

**Why:** Smallest state that matches the UX. Alternatives considered: separate modal for path edit (too heavy); always-editable with confirm-to-save (changes live-persist model for one field only and still allows accidental focus typing).

### 2. Key routing while Adb focused

| Mode | Key | Behavior |
|------|-----|----------|
| locked | `e` | set `adb_editing = true` |
| locked | `r` | clear path → Auto, commit, stay locked |
| locked | other Char / Backspace | ignore |
| editing | Char / Backspace | mutate path + commit (`e`/`r` are literals) |
| editing | Esc | `adb_editing = false`, keep value, modal stays open |
| any | Enter | dismiss modal (existing) |
| locked | Esc | dismiss modal (existing) |

Leaving the Adb row with Up/Down while editing: clear `adb_editing` (keep value). Avoids “invisible edit mode” after focus moves.

**Why Esc-exits-edit over Esc-cancels:** User chose keep-on-exit; live persist already wrote edits, so cancel would need a draft buffer we explicitly avoided.

### 3. Display

When locked, show the effective path (custom text if non-empty, else `auto_adb` / not-found) plus an Auto/Custom marker and short `e`/`r` hints on the focused row or help line. When editing, show the editable buffer in `[...]` as today so the user sees what they type.

### 4. Help / i18n

Extend `modal_settings_help` (and any Adb-specific labels) so the top help line documents Adb lock bindings without claiming all text fields are always direct-type. Custom capacity remains “type when focused”.

## Risks / Trade-offs

- **[Users who relied on immediate typing]** → One extra `e` keystroke; documented in help line.
- **[Esc semantics split]** → Esc means two things depending on edit flag; mitigate with help text and by clearing edit flag on focus leave.
- **[Accidental `r`]** → Cleared path is recoverable by `e` + retype, but painful; acceptable because `r` only works while locked and focused on Adb, not globally.

## Migration Plan

No settings file migration. Existing custom `adbPath` values load as today and appear as Custom/locked; `r` clears them. Rollback is a code revert only.

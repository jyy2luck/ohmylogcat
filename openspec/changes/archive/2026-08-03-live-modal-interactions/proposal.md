## Why

Tag/Message filter modals and the Settings modal currently use inconsistent keyboard semantics: filter Esc closes while Settings Enter saves and Esc cancels, even though filters already apply live. Users must remember different confirm/cancel models per overlay. Aligning interactions with each modal's purpose (live text edit vs live option panel) reduces friction and matches patterns already used by the Level control.

## What Changes

- **Tag/Message filter modals**: Esc uses progressive dismiss — clear non-empty input (filter updates live), close when already empty; Enter closes the modal without a separate confirm step.
- **Settings modal**: Horizontal adjust and text edits apply immediately to runtime state and persist to settings storage; Enter and Esc both only close the modal (no save vs cancel distinction).
- **Help strings**: Update filter live hint and settings modal help line in all locales to document the new bindings.
- **BREAKING**: Settings Esc no longer discards unsaved changes (changes are applied as they are made). Filter Esc no longer closes on first press when input has content.

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

- `tui-shell`: Tag/Message filter modal Esc/Enter behavior; Settings modal live-apply and dismiss-only Enter/Esc; help line text; Esc overlay scenario caveats for filter vs settings.
- `ui-i18n`: Language preference applies to chrome immediately when adjusted in Settings, not only on explicit save.
- `app-settings`: Settings fields persist as they are changed in the Settings modal rather than on Enter save.

## Impact

- `src/app.rs`: `handle_filter_edit_modal_key`, `handle_settings_modal_key`, `cycle_*` helpers, new `apply_settings_*` / persist-on-change paths; remove save-only-on-Enter for settings.
- `src/ui/i18n.rs`: Updated help strings (`filter_live_hint`, `modal_settings_help`) in en / zh-CN / zh-TW.
- OpenSpec main specs synced on archive: `tui-shell`, `ui-i18n`, `app-settings`.

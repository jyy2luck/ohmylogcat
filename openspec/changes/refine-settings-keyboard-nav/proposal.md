## Why

The Settings modal uses Tab to move between fields and `[` `]` to cycle preset/theme, which is inconsistent with other modals (e.g. Devices uses ↑/↓) and splits adjustment across multiple key bindings. The help line at the top does not document the full interaction model, and focus can land on the hidden Custom capacity row when preset is not Custom—making navigation feel broken.

## What Changes

- Replace Tab-based field navigation with ↑/↓ (and j/k) to move the focus cursor between visible settings rows.
- Use ←/→ (and h/l) to adjust cycle-type fields (buffer preset, theme); text fields (ADB path, custom capacity) remain direct typing with Backspace.
- Display a single help line at the top of the Settings modal documenting move, adjust, save, and cancel keys (including vim aliases where applicable).
- Remove Tab/BackTab and `[` `]` as primary navigation/adjustment bindings for Settings (they may be dropped entirely).
- Fix focus tracking so navigation skips the Custom capacity row when it is not visible, and re-anchor focus when preset changes hide or show that row.

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

- `tui-shell`: Settings modal keyboard navigation, value adjustment bindings, on-screen help text, and visible-row focus behavior

## Impact

- `src/app.rs` — `handle_settings_modal_key`, Settings modal render, `SettingsPanelState` focus logic
- `openspec/specs/tui-shell/spec.md` — requirement delta for Settings modal interaction

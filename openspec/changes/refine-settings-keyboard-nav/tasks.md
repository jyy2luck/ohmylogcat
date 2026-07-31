## 1. Settings field model

- [ ] 1.1 Add `SettingsField` enum (Adb, Preset, Custom, Theme) and replace raw `focus_field: usize` in `SettingsPanelState`
- [ ] 1.2 Implement `visible_fields(preset) -> &[SettingsField]` and `move_focus(&mut self, delta: isize)` that wraps within visible rows only

## 2. Keyboard handling

- [ ] 2.1 Route Up/Down/j/k to `move_focus(-1/+1)` in `handle_settings_modal_key`
- [ ] 2.2 Route Left/Right/h/l to `cycle_preset` / `cycle_theme` when Preset or Theme is focused; no-op on text fields
- [ ] 2.3 Remove Tab/BackTab and `[` `]` handlers from Settings modal
- [ ] 2.4 After `cycle_preset`, re-anchor focus to Preset when Custom row becomes hidden while Custom was focused

## 3. Rendering

- [ ] 3.1 Update Settings modal help line to document ↑/↓ move, ←/→ adjust, type text, Enter save, Esc cancel (include j/k h/l aliases)
- [ ] 3.2 Drive row `>` markers and conditional Custom row from `visible_fields` + current focus field

## 4. Verification

- [ ] 4.1 Manual: navigate all rows with ↑/↓; cycle preset/theme with ←/→; type in ADB and Custom text fields
- [ ] 4.2 Manual: with Custom preset focused, cycle to Normal — focus stays visible and Custom row disappears from navigation
- [ ] 4.3 Run `cargo build` and smoke-test Settings save still persists to JSON

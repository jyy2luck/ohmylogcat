## 1. Panel state and key handling

- [x] 1.1 Add `adb_editing: bool` to `SettingsPanelState`, default `false` in `from_settings` / open path
- [x] 1.2 In `handle_settings_modal_key`, when Adb focused and locked: `e` → enter edit; `r` → clear path + `commit_settings_from_panel`; ignore other Char/Backspace for Adb
- [x] 1.3 When Adb focused and editing: Char/Backspace mutate path + commit (`e`/`r` as literals); Esc clears `adb_editing` and does not dismiss modal
- [x] 1.4 On Up/Down focus move away from Adb, clear `adb_editing`; Enter still dismisses; Esc dismisses only when not editing Adb

## 2. Display and i18n

- [x] 2.1 Render ADB row with effective path + Auto/Custom marker; show editable `[buffer]` while `adb_editing`
- [x] 2.2 Update `modal_settings_help` (en/zh-CN/zh-TW) for `e` edit, `r` restore Auto, Esc exits edit vs dismiss
- [x] 2.3 Add any short labels needed for Auto/Custom / edit hints on the Adb row

## 3. Verify

- [x] 3.1 Manually verify: locked typing ignored; `e` then type persists; Esc exits edit keeps value/modal; `r` restores Auto; Custom capacity still direct-type; Enter/`Esc` (locked) still close Settings

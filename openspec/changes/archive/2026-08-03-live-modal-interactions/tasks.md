## 1. Filter modal keyboard behavior

- [x] 1.1 In `handle_filter_edit_modal_key`, handle `Enter` by closing the modal without changing filter state
- [x] 1.2 Implement progressive Esc: if input non-empty, clear text, reset cursor, and `mark_filter_dirty`; if empty, `close_modal`
- [x] 1.3 Update `filter_live_hint` in `src/ui/i18n.rs` (en / zh-CN / zh-TW) to document Esc clear/close and Enter done

## 2. Settings immediate apply and persist

- [x] 2.1 Extract `commit_settings_from_panel` (or equivalent) from `save_settings_panel` that builds `Settings`, applies runtime side effects (`theme`, `locale`/`ui`, `engine.set_capacity`), and calls `save_settings`
- [x] 2.2 Call commit helper from `cycle_preset`, `cycle_theme`, and `cycle_language` after panel state updates
- [x] 2.3 Call commit helper after adb path and custom capacity text edits in `handle_settings_modal_key`
- [x] 2.4 Change Settings `Enter` and `Esc` to only `close_modal` (remove save-on-Enter and discard-on-Esc semantics)
- [x] 2.5 Remove or slim down `save_settings_panel` if fully superseded; keep error surfacing via `settings_panel.status` on persist failure

## 3. Settings help strings

- [x] 3.1 Update `modal_settings_help` in `src/ui/i18n.rs` (en / zh-CN / zh-TW) to document immediate apply and Enter/Esc dismiss (remove save/cancel wording)

## 4. Verification

- [x] 4.1 Manual smoke: Tag filter Esc clears then closes; Enter closes with filter intact
- [x] 4.2 Manual smoke: Settings theme/language/buffer change applies visually and persists after modal close and restart
- [x] 4.3 Run `cargo test` and fix any regressions

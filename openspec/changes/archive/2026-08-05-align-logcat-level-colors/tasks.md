## 1. Restore AS dual Logcat palettes

- [x] 1.1 In `src/ui/theme.rs`, replace `Theme::android()` with `dark_accents()` / `light_accents()` using the AS Logcat table (Verbose/Debug/Info/Warn/Error dark+light hex); keep `interaction_accents()` unchanged.
- [x] 1.2 Restore `Theme::resolve()`, `detect_light_background()`, `detect_light_background_from()`, and `$COLORFGBG` parsing (bg ≥ 7 ⇒ light; else dark); make `Default` delegate to `resolve()`.
- [x] 1.3 Update the `Theme` doc comment to describe dual Android Studio Logcat palettes + silent background detection.

## 2. Update call sites and tests

- [x] 2.1 Ensure `src/app.rs` constructs theme via `Theme::default()` or `Theme::resolve()`.
- [x] 2.2 In `src/ui/theme.rs` tests, assert both palettes' level RGB; restore detection tests; map Error/Fatal via `dark_accents()`.
- [x] 2.3 In `src/ui/selection.rs` tests, use `Theme::dark_accents()`.

## 3. Verify

- [x] 3.1 Run `cargo test` and confirm all tests pass.
- [x] 3.2 Confirm no new clippy warnings in changed files.
- [x] 3.3 Smoke-check: dark Info is olive `#ABC023` (not neon green); light Info `#59A869` when `COLORFGBG` selects light; find remains black-on-yellow.

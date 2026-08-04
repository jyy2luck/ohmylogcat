## 1. Collapse theme model

- [x] 1.1 Remove `ThemePreference`, Auto/`COLORFGBG` detection, and dual `dark()`/`light()` palettes from `src/ui/theme.rs`; keep a single static accent palette for levels, focus, selection, and find
- [x] 1.2 Drop `theme` from `Settings` / panel state / serde; ensure legacy `theme` in settings.json is ignored and omitted on next save
- [x] 1.3 Update `Theme::resolve` call sites (or replace with `Theme::default` / `Theme::accents`) in `app.rs` so runtime no longer branches on preference

## 2. Settings UI and i18n

- [x] 2.1 Remove `SettingsField::Theme`, Theme row rendering, and `cycle_theme` / horizontal-adjust routing from Settings modal
- [x] 2.2 Remove `settings_theme` (and unused related strings) from `UiStrings` / locale catalogs
- [x] 2.3 Fix focus re-anchor paths that listed theme as a neighbor when Custom hides

## 3. Shell chrome follows terminal

- [x] 3.1 Stop forcing `shell_fg` / `shell_muted` / `shell_divider` / `shell_hint` on toolbar, separators, filter row labels, status, and empty hints — use default style like modals (keep focus/selection/find accents)
- [x] 3.2 Keep log level coloring and selection/find/focus accents on the fixed palette; remove unused chrome color fields if nothing references them

## 4. Tests and smoke

- [x] 4.1 Replace Auto/Windows theme unit tests with coverage for the single accent palette and absence of ThemePreference
- [x] 4.2 Run `cargo test` and manually smoke Settings (no Theme row) plus main shell on a dark terminal (and light if available)

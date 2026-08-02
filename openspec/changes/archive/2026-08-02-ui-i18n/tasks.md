## 1. Locale model and persistence

- [x] 1.1 Add `sys-locale` dependency to `Cargo.toml`
- [x] 1.2 Create `src/ui/i18n.rs` with `LanguagePreference` (Auto/English/ZhHans/ZhHant), `Locale` (En/ZhHans/ZhHant), `cycle`, serde tags (`auto`/`en`/`zh-Hans`/`zh-Hant`), and `label()` fixed option strings
- [x] 1.3 Implement `Locale::resolve(pref)` using `sys-locale` + Auto rule table (Hans/CN/SG → ZhHans; Hant/TW/HK/MO → ZhHant; bare `zh` → ZhHans; `en*` → En; else En); unit-test resolver against fixture locale strings
- [x] 1.4 Add `language: LanguagePreference` to `Settings` with `#[serde(default)]`, wire into Default / load / save
- [x] 1.5 Export i18n types from `src/ui/mod.rs`

## 2. String catalog

- [x] 2.1 Define `UiStrings` (or equivalent) covering toolbar, filter row, find bar, empty state, status messages, and all modal chrome for En / ZhHans / ZhHant
- [x] 2.2 Provide `UiStrings::for_locale(Locale)` builders so each locale fills every field (compile-time exhaustiveness)

## 3. Settings modal Language field

- [x] 3.1 Add `SettingsField::Language` and include it in `visible_fields` after Theme
- [x] 3.2 Hydrate/save `settings_panel.language` from/to `Settings`; cycle with ←/→ / h/l like Theme
- [x] 3.3 Render Language row with localized row title + fixed option labels; on successful save, resolve locale and refresh active `UiStrings` immediately

## 4. Wire chrome to catalog

- [x] 4.1 Hold resolved `Locale` + `UiStrings` on app state; initialize from loaded settings at startup
- [x] 4.2 Replace toolbar / filter / find / empty-state hardcoded strings with catalog lookups (keep Latin shortcut letters)
- [x] 4.3 Replace modal titles, help lines, and prompts (Devices, Export, Settings, Filter edit, path entry) with catalog lookups
- [x] 4.4 Replace ephemeral status strings (settings saved, exported, copied, tips) with catalog lookups
- [x] 4.5 Spot-check narrow terminal toolbar width with zh labels; shorten labels only if overflow is observed

## 5. Verification

- [x] 5.1 Run unit tests for Auto locale resolution edge cases (zh-CN, zh-TW, zh-HK, bare zh, en-US, ja-JP, missing)
- [x] 5.2 Manually verify Settings: cycle Language, save, confirm chrome switches without restart; restart confirms persistence
- [x] 5.3 Manually verify Auto on a Chinese and a non-Chinese system locale (or mocked resolve path) matches spec fallback rules

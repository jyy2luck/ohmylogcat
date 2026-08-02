## Why

The TUI is English-only today. Chinese-speaking users (both Simplified and Traditional) should be able to use the shell in their language, with a sensible Auto default that follows the OS locale and falls back to English when the locale is unsupported.

## What Changes

- Add a persisted **language** preference: `Auto` / `English` / `简体中文` / `繁體中文` (default `Auto`)
- Resolve `Auto` from the system locale: Hans/CN/SG → Simplified; Hant/TW/HK/MO → Traditional; bare `zh` → Simplified; `en*` → English; everything else → English
- Localize TUI chrome strings (toolbar, filters, modals, status hints, empty states) for en / zh-Hans / zh-Hant
- Expose language as a cycleable field in the Settings modal (same interaction pattern as Theme)
- Apply the resolved locale immediately when settings are saved

## Capabilities

### New Capabilities
- `ui-i18n`: Locale preference model, system-locale Auto resolution, and localized UI string catalog for the TUI shell

### Modified Capabilities
- `app-settings`: Persist and restore the language preference alongside existing settings
- `tui-shell`: Settings modal gains a Language field; shell chrome renders localized labels while keyboard shortcuts stay Latin

## Impact

- `src/settings.rs` — new `language` field on `Settings`
- New locale/i18n module (e.g. `src/ui/i18n.rs` or `src/ui/locale.rs`) for preference enum, resolve, and string table
- `src/app.rs` — Settings panel field, save/apply path, replace hardcoded chrome strings with catalog lookups
- Optional light dependency for system locale detection (e.g. `sys-locale`); otherwise env/`GetUserDefault*` heuristics
- No change to log line content, adb protocol, or buffer/filter semantics
- Existing `settings.json` without `language` defaults to `Auto` (backward compatible)

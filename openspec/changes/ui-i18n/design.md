## Context

See proposal.md for motivation. The TUI is a single-process ratatui/crossterm app; almost all chrome strings live inline in `src/app.rs`. Settings already persist `theme: ThemePreference` with an `Auto` resolve path — language should mirror that pattern. No i18n crate or catalog exists today.

## Goals / Non-Goals

**Goals:**

- Hand-rolled locale preference + string catalog matching existing Theme ergonomics
- Reliable Auto detection on Windows and Unix without heavy OS glue
- Localized chrome only; keep shortcut letters Latin

**Non-Goals:**

- Translating log line content or adb/device error payloads from external tools
- Runtime language packs, plural rules engines, or Fluent/gettext tooling
- Additional languages beyond en / zh-Hans / zh-Hant
- Instant preview of language while cycling inside the Settings modal before save (apply on save, like theme)

## Decisions

### 1. Hand-rolled catalog over i18n frameworks

**Choice:** `LanguagePreference` + resolved `Locale` + static `UiText` / key lookups in a new `src/ui/i18n.rs` (or `locale.rs`).

**Why:** String surface is small and owned by one crate; zero new framework concepts; matches project’s minimal-dependency style (`Cargo.toml` is already lean).

**Alternatives:** `rust-i18n` / Fluent — deferred until more locales or external translators appear.

### 2. Mirror `ThemePreference` for persistence and UX

**Choice:**

```
LanguagePreference { Auto, English, ZhHans, ZhHant }  // serde: auto|en|zh-Hans|zh-Hant
Locale { En, ZhHans, ZhHant }                         // resolved active locale
```

- `Settings.language: LanguagePreference` with `#[serde(default)]` → Auto
- Settings modal adds `SettingsField::Language` after Theme
- ←/→ / h/l cycles preference; Enter saves and calls `Locale::resolve(pref)`

**Why:** Users already know this interaction; archive/delta specs stay consistent with theme.

### 3. System locale via `sys-locale` + rule table

**Choice:** Add `sys-locale` to read the OS locale string; normalize to lowercase; apply the Auto rules from the ui-i18n spec (Hans/CN/SG → ZhHans; Hant/TW/HK/MO → ZhHant; bare `zh` → ZhHans; `en*` → En; else En).

**Why:** Cross-platform without `#cfg` WinAPI / `LANG` parsing in app code. Fallback if detection fails: English.

**Alternatives:** Pure env-var heuristics — weaker on Windows where `LANG` is often unset.

### 4. Fixed bilingual option labels

**Choice:** Language row always shows `Auto` / `English` / `简体中文` / `繁體中文`, independent of active locale. Other chrome (row title “Language” / “语言” / “語言”, help line, etc.) follows the active locale.

**Why:** Users can always recognize their target language while exploring options; avoids “everything looks Chinese except I wanted English” confusion when Auto is wrong.

### 5. Catalog shape

**Choice:** Central `fn t(locale, key) -> &'static str` or a `UiStrings` struct filled once per resolve, keyed by stable identifiers (`toolbar.settings`, `status.settings_saved`, …). Prefer a struct of fields for compile-time exhaustiveness when wiring `app.rs`.

**Why:** Missing translations fail at compile time if every locale builds a full `UiStrings`.

### 6. Width / layout

**Choice:** Keep existing layout; rely on ratatui unicode width. Spot-check toolbar overflow on narrow terminals after Chinese labels land; truncate only if a concrete overflow shows up in testing (no speculative truncation layer).

## Risks / Trade-offs

- **[Risk] Chinese labels widen the toolbar and clip on narrow terminals** → Mitigation: manual width check; shorten zh labels if needed (e.g. 设置 vs 设定项).
- **[Risk] Incomplete string migration leaves mixed EN/ZH chrome** → Mitigation: tasks enumerate surfaces (toolbar, filters, modals, status); greppable English leftovers as a checklist.
- **[Risk] `sys-locale` returns unexpected formats** → Mitigation: normalize + rule table with English fallback; unit-test Auto resolver against fixture locale strings.
- **[Trade-off] Apply-on-save (not live preview)** → Same as theme; simpler state; user confirmed via explore defaults.

## Migration Plan

1. Ship with `#[serde(default)]` on `language` so old `settings.json` keeps working (Auto).
2. No settings file rewrite required on first launch.
3. Rollback: remove field + catalog; old installs ignore unknown JSON keys if we ever reverse — but forward path is additive only.

## Open Questions

None deferred — bare `zh` → Simplified was decided before proposal.

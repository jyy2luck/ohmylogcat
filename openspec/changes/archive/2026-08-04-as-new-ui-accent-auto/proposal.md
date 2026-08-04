## Why

Info-level logs use ANSI `Gray`, which is hard to read on light terminals. Android Studio already defines distinct light and dark Logcat level colors; we should match those accents while still following the host terminal automatically—without bringing back a Settings Theme control.

## What Changes

- Replace the single fixed ANSI accent palette with two RGB palettes aligned to Android Studio / IntelliJ console `LOG_*` colors:
  - **Light**: IntelliJ Default / IntelliJ Light (Info green `#00cd00`)
  - **Dark**: Android Studio New UI / Islands Dark (Info gold `#e0bb65`)
- At startup (and only for accents), silently detect whether the host terminal background is light or dark; apply the matching palette.
- When detection fails or is inconclusive, fall back to the **dark** palette.
- Keep shell chrome on terminal defaults; keep Settings free of any Theme / Auto / Light / Dark preference.
- Focus, selection, and find accents remain on the chosen palette’s interaction colors (or stay as today’s fixed interaction accents if they already contrast on both hosts—design decides).

## Capabilities

### New Capabilities

<!-- none -->

### Modified Capabilities

- `ui-colors`: Dual Android Studio–aligned accent palettes with silent background detection and dark fallback; still no user-selectable theme mode; chrome still follows terminal defaults.

## Impact

- Code: `src/ui/theme.rs` (restore light/dark accent constructors + background detection; no `ThemePreference` / settings field), call sites in `app.rs` that build `Theme`.
- Specs: `ui-colors` (replace “single fixed palette / no Auto detection” with silent dual accents).
- Tests: coverage for light/dark palette RGB values, detection success paths, and dark fallback when detection fails.
- No Settings UI, i18n, or settings.json schema changes (legacy `theme` remains ignored).

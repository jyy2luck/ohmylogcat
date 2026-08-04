## Why

The Settings Theme preference (Auto / Dark / Light) fights the host terminal instead of following it. Auto detection is brittle (especially on Windows without `$COLORFGBG`), and users do not want an app-level light theme inside a dark terminal. Modals already look correct by inheriting terminal defaults — the main shell should do the same.

## What Changes

- **BREAKING**: Remove the persisted Theme preference (`auto` / `dark` / `light`) and the Theme row from the Settings modal.
- Stop forcing shell chrome colors from a Dark/Light palette; main shell text/dividers follow the terminal default like Tag/Message/Settings modals.
- Keep a single fixed accent palette for semantic UI only: log levels, focus highlight, selection, and find matches (ANSI/named colors that ride the terminal's own color table).
- Ignore a legacy `theme` field in existing `settings.json` (no migration UI; value is discarded on next save).

## Capabilities

### New Capabilities

- `ui-colors`: Terminal-native shell chrome plus a fixed semantic accent palette (levels, focus, selection, find) with no user-selectable theme mode.

### Modified Capabilities

- `app-settings`: Settings no longer expose or persist a theme preference; settings entry and live-persist rules drop theme.
- `tui-shell`: Settings modal navigation/adjust scenarios no longer include a Theme row; open-settings scenario no longer lists theme controls.

## Impact

- Code: `src/ui/theme.rs` (drop `ThemePreference` / Auto detect; simplify `Theme`), `src/settings.rs`, `src/app.rs` (Settings field, cycle/commit, draw paths), i18n strings for settings theme label if unused afterward.
- Specs: `app-settings`, `tui-shell`; new `ui-colors`.
- Tests: replace Auto/Windows fallback tests with chrome-default / accent coverage.
- No new dependencies.

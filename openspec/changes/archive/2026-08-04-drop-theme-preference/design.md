## Context

See proposal.md — Why. Today `ThemePreference::{Auto,Dark,Light}` persists in settings; `Theme::resolve` picks `dark()` / `light()` RGB palettes (Auto via `$COLORFGBG`, else dark after the recent Windows fallback fix). Main shell always paints `self.theme.*`. Settings / Tag / Message modals use unstyled `Style::default()` and already match the terminal. Specs currently mention theme in Settings entry, live persist, and keyboard adjust scenarios.

## Goals / Non-Goals

**Goals:**

- One color model: terminal-default chrome + fixed semantic accents.
- Remove Theme from Settings UI, settings model, and Auto detection code.
- Keep log level / focus / selection / find visually distinct.

**Non-Goals:**

- Runtime OSC / Win32 background querying.
- Per-user custom color pickers.
- Restyling modal chrome beyond keeping current terminal-default behavior.
- Changing find/selection interaction behavior — only their color source.

## Decisions

### 1. Drop `ThemePreference`; keep a single `Theme` (or rename to accents)

**Choice:** Delete `ThemePreference`, `cycle`, serde theme field, Settings `Theme` row, and `detect_light_background`. Keep one static accent struct used by log levels, focus, selection, and find. Shell chrome styles omit forced foreground (use `Style::default()` / no `.fg(...)`), matching modals.

**Alternatives considered:**

- Keep Auto-only with better detection → still maintains a parallel palette and detection surface; rejected.
- ANSI-only everywhere including levels without a struct → harder to keep selection/find contrast consistent; a small fixed struct is fine.

### 2. Accent colors: named/ANSI-style colors, not dual RGB palettes

**Choice:** Use one palette (prefer ratatui named / standard ANSI colors where contrast is acceptable on both light and dark hosts; keep RGB only if a named color is clearly worse for Error/Warn). No light/dark branch.

**Why:** Terminal palettes already adapt; dual RGB themes were the reason Auto existed.

### 3. Legacy `theme` in settings.json

**Choice:** Remove `theme` from `Settings`. Serde will ignore unknown fields by default if we simply delete the field (confirm `Settings` deserialize behavior — if deny_unknown is not set, old files load; next save omits `theme`).

**Why:** No migration UI; preference is intentionally gone.

### 4. i18n

**Choice:** Remove `settings_theme` from `UiStrings` and catalogs once the Settings row is gone.

## Risks / Trade-offs

- **[Risk] Fixed accents wash out on unusual terminal palettes** → Mitigation: stick to common ANSI roles (red/yellow/blue/gray); manual smoke on dark + light terminals.
- **[Trade-off] Users who forced Light on a dark terminal lose that override** → Accepted; product decision is follow-the-host.
- **[Risk] Separators become less visible without `shell_divider` gray** → Mitigation: keep a subtle named DarkGray/Gray only if default looks flat in smoke; otherwise plain default like modals.

## Migration Plan

1. Ship code that stops reading/writing theme and drops the Settings row.
2. Existing installs: old `theme` key ignored; no prompt.
3. Rollback: revert commit; old settings files still valid if they retained other fields.

## Open Questions

None — product chose terminal-native chrome over theme modes.

## Context

See proposal.md — Why. Today `Theme::accents()` is one ANSI palette (`Info = Gray`, `Verbose = DarkGray`). Shell chrome already uses terminal defaults; Settings has no Theme row; legacy `settings.json` `theme` is ignored. Older code had `detect_light_background()` via `$COLORFGBG` (bg index ≥ 7 ⇒ light; unset ⇒ dark)—that heuristic returns for accents only.

Source of truth for level colors: IntelliJ `DefaultColorSchemesManager.xml` / New UI schemes (`LOG_*` console attributes), which Android Studio Logcat inherits.

## Goals / Non-Goals

**Goals:**

- Dual RGB level palettes matching AS Light + New UI Dark (gold Info).
- Silent resolve at startup from host background; dark when unknown.
- Keep chrome terminal-default; keep no Theme UI / no persisted preference.

**Non-Goals:**

- Restoring `ThemePreference`, Settings Theme row, or settings.json `theme`.
- OSC / Win32 API terminal background queries (beyond existing env heuristics).
- Matching AS tag-hash colors or per-process coloring—only priority/level accents.
- Redesigning focus / selection / find interaction colors (keep current shared accents).
- Truecolor fallbacks for terminals without RGB support beyond ratatui’s existing behavior.

## Decisions

### 1. Dual level palettes; shared interaction accents

**Choice:** `Theme::light_accents()` / `Theme::dark_accents()` (or equivalent) hold AS-aligned **level** RGB. Focus / selection / find stay the current named accents (Black/Yellow, White/Blue, Black/Yellow) shared by both.

**Why:** The readability bug and AS alignment are about log levels; interaction accents already contrast on both hosts and are not defined by AS Logcat `LOG_*`.

**Alternatives considered:** Duplicate interaction colors per palette — unnecessary churn.

### 2. Exact level RGB (from IntelliJ / AS New UI)

| Level | Light (Default / IntelliJ Light) | Dark (New UI / Islands Dark) |
|-------|----------------------------------|------------------------------|
| Verbose | `#0000ee` | `#56a8f5` |
| Debug | `#00cccc` | `#299999` |
| Info | `#00cd00` | `#e0bb65` |
| Warn | `#a66f00` | `#a66f00` (AS dark leaves FG unset; use Default Warn) |
| Error / Fatal | `#cd0000` | `#f75464` |

**Why:** Matches product choice (New UI gold Info on dark). Warn on dark has no explicit FG in scheme XML; `#a66f00` is the inherited Default value and remains readable on dark backgrounds.

### 3. Silent resolve API (no preference enum)

**Choice:** `Theme::resolve()` / `Theme::default()` calls background detection and returns light or dark accents. No `ThemePreference`, no serde, no Settings wiring.

**Alternatives considered:** Restore Auto/Dark/Light preference — rejected by product; user wants adaptation without a switch.

### 4. Detection heuristic and dark fallback

**Choice:** Reuse `$COLORFGBG` (`fg;bg…`, last segment as bg index): `bg >= 7` ⇒ light, else dark. If unset / unparsable ⇒ **dark** (explicit product decision; matches prior Windows fallback).

**Why:** Same practical signal as before; dark-default avoids painting light-palette blues/greens assuming a white chrome on typical dark terminals when Windows Terminal omits `COLORFGBG`.

**Alternatives considered:** Always dark (no detection) — fails the light-terminal Info case. OSC query — out of scope for this change.

### 5. When to resolve

**Choice:** Resolve once at app startup when building `App` / `Theme` (same as today’s `Theme::accents()` call site). Do not re-detect on every frame.

**Why:** Terminal background does not change mid-session for our purposes; keeps behavior simple.

## Risks / Trade-offs

- **[Risk] Mis-detection** (e.g. light terminal without `COLORFGBG` gets dark gold Info) → Mitigation: dark fallback still beats Gray on light; document that setting `COLORFGBG` improves accuracy; optional later OSC work stays non-goal.
- **[Risk] Truecolor RGB ignored on ancient terminals** → Mitigation: accept ratatui/terminal degradation; levels still attempt distinct colors.
- **[Trade-off] Warn dark not a unique AS New UI swatch** → Accepted; use Default `#a66f00`.
- **[Trade-off] Main `ui-colors` Purpose still says “single fixed palette” until archive/sync** → Task: update Purpose when archiving or via sync.

## Migration Plan

1. Ship dual accents + silent resolve; no settings migration.
2. Existing installs: ignore legacy `theme` as today.
3. Rollback: revert to single `Theme::accents()` ANSI palette.

## Open Questions

None — dark fallback and New UI gold Info decided with the user.

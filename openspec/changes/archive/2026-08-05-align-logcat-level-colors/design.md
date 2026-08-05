## Context

See proposal.md — Why. Today `Theme::android()` applies one neon `logprint.c` palette with no background detection. Interaction accents are shared. Call site is `Theme::default()` in `app.rs`. Source of truth for the new values: Android Studio → Editor → Color Scheme → Android Logcat (dark / light).

## Goals / Non-Goals

**Goals:**
- Dual AS Logcat level palettes matching the user-provided dark/light hex table.
- Silent host-background selection via `$COLORFGBG` (dark fallback).
- Keep interaction accents unchanged.

**Non-Goals:**
- Redesigning focus / selection / find colors.
- User-selectable theme mode or Settings Theme row.
- Per-tag/per-process colors.

## Decisions

### 1. Dual constructors + resolve

**Choice:** `Theme::dark_accents()` / `Theme::light_accents()` hold level RGB; `Theme::resolve()` / `Default` pick via `$COLORFGBG` (`bg >= 7` ⇒ light; unset/unparsable ⇒ dark). Remove `Theme::android()`.

**Why:** AS exposes two schemes; a single mid-tone palette failed visual match. Detection was removed only when palettes were identical — they are not anymore.

**Alternatives considered:** Always-dark only — drops light-terminal match the user supplied.

### 2. Exact AS Logcat RGB

| Level | Dark | Light |
|-------|------|-------|
| Verbose | `#BBBBBB` | `#000000` |
| Debug | `#299999` | `#389FD6` |
| Info | `#ABC023` | `#59A869` |
| Warn | `#BBB529` | `#645607` |
| Error / Fatal (Assert) | `#FF6B68` | `#CD0000` |

**Why:** User-verified from AS. Assert shares Error in AS; we map Fatal → error accent (no separate Assert level).

### 3. Interaction accents unchanged

**Choice:** `interaction_accents()` keeps focus Black/Yellow, selection White/Blue, find Black/Yellow.

**Why:** Already contrast on both hosts; dark Info `#ABC023` stays distinct from find yellow.

### 4. Tests

**Choice:** Assert both palettes' level RGB; restore detection helper tests; `level_color_maps_error_and_fatal` on `dark_accents()`; selection tests use `dark_accents()`.

## Risks / Trade-offs

- **[Risk] Mis-detection on Windows without `COLORFGBG`** → Mitigation: dark fallback (typical host).
- **[Trade-off] Verbose is tinted again** (not terminal default) → Accepted; matches AS.

## Migration Plan

1. Swap palette + restore resolve in `theme.rs`.
2. Point `app.rs` at `Theme::default()` / `resolve()`; update tests.
3. No settings migration. Rollback: revert the commit.

## Open Questions

None.

## 1. Dual accent palettes

- [x] 1.1 Replace `Theme::accents()` ANSI levels with `light_accents()` / `dark_accents()` RGB values from design.md (Verbose/Debug/Info/Warn/Error); keep shared focus/selection/find named accents
- [x] 1.2 Add `Theme::resolve()` (or `Default`) that picks light vs dark accents via `$COLORFGBG` (`bg >= 7` ⇒ light; unset/unparsable ⇒ dark); call it once from the existing `App` theme construction site
- [x] 1.3 Ensure Fatal still maps to the active Error accent; do not reintroduce `ThemePreference`, Settings Theme UI, or settings.json `theme`

## 2. Tests and verification

- [x] 2.1 Unit-test light vs dark Info (and key level) RGB values; assert dark ≠ light Info; assert resolve without `COLORFGBG` yields dark accents
- [x] 2.2 Add or adjust a test that a light `COLORFGBG` background selects light accents (isolate env safely in-process or via a testable detect helper)
- [x] 2.3 Run `cargo test`; manually smoke Info on a dark terminal and, if available, a light terminal (or forced `COLORFGBG`)

## 3. Spec hygiene

- [x] 3.1 When archiving or syncing this change, update main `openspec/specs/ui-colors/spec.md` Purpose so it no longer claims a single fixed palette with no Auto detection

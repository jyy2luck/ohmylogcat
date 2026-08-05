## Why

The neon single palette from `adb logcat -v color` (`#00D700` Info) is too harsh and does not match Android Studio's Logcat scheme. Users compare against AS Editor → Color Scheme → Android Logcat, which has distinct dark/light swatches (Info `#ABC023` / `#59A869`).

## What Changes

- Replace the single `Theme::android()` / `logprint.c` palette with dual Android Studio Logcat palettes (dark / light), values taken from AS scheme:
  - Verbose: `#BBBBBB` / `#000000`
  - Debug: `#299999` / `#389FD6`
  - Info: `#ABC023` / `#59A869`
  - Warn: `#BBB529` / `#645607`
  - Error / Fatal / Assert: `#FF6B68` / `#CD0000`
- Restore silent `$COLORFGBG` background detection (`Theme::resolve()`); dark when unset/unparsable.
- Focus, selection, and find interaction accents stay unchanged (shared Black/Yellow, White/Blue, Black/Yellow).
- No user-selectable theme mode; shell chrome still follows terminal defaults.

## Capabilities

### New Capabilities

<!-- none -->

### Modified Capabilities

- `ui-colors`: replaces the single Android Logcat 4-color / no-detection requirement with dual Android Studio Logcat level palettes chosen by silent host-background detection (no user theme preference).

## Impact

- Code: `src/ui/theme.rs` (dual constructors + detection), `src/app.rs` (resolve/default), tests in `theme.rs` / `selection.rs`.
- Specs: `ui-colors` delta rewrites the palette requirement and scenarios.
- No settings schema or i18n changes.

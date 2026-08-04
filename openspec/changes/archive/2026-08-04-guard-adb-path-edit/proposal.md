## Why

Settings modal focuses ADB path first and accepts every keystroke with immediate persistence, so accidental typing easily corrupts a hard-to-restore custom path. Users need an escape hatch that still allows intentional overrides without making the path a free-typing landmine.

## What Changes

- ADB path row in Settings becomes read-only by default (no direct typing/backspace while locked)
- Explicit unlock with `e` enters edit mode; printable keys and Backspace then work as today with immediate persist
- `r` while locked clears the custom path and restores Auto resolution, persisted immediately
- Esc while editing exits edit mode and keeps the current value (does not dismiss Settings); Esc while locked still dismisses Settings
- Enter continues to dismiss Settings (not used to enter edit)
- Custom capacity and other settings rows are unchanged

## Capabilities

### New Capabilities

<!-- none -->

### Modified Capabilities

- `app-settings`: Custom ADB path remains supported, but editing requires an explicit unlock; restore-to-Auto becomes a first-class action
- `tui-shell`: Settings keyboard model for ADB path changes from always-direct text entry to locked/edit modes with documented bindings

## Impact

- `src/app.rs`: Settings panel state (edit lock), key handling, and Settings modal rendering/help text
- i18n strings for help/status/Auto vs Custom labels as needed
- Specs: `openspec/specs/app-settings`, `openspec/specs/tui-shell`
- No storage format change: empty `adbPath` continues to mean Auto; non-empty remains a custom absolute path

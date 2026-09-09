## Why

Currently ohmylogcat requires manual device selection after plugging in a phone or emulator, even when it is the only available device. This breaks the expected plug-and-play workflow for log viewing and forces an extra step on every connect/disconnect cycle.

## What Changes

- Automatically select and start log streaming when the first valid device appears and nothing is connected yet (startup with device present, or device plugged in later).
- When streaming on one device and a second device is plugged in, keep the current device selected (no automatic switch).
- When the active device is unplugged, enter a disconnected state that preserves the existing log buffer; do not clear logs on disconnect.
- When the active device disappears but other valid devices remain in the list, automatically switch to the first remaining valid device and resume streaming.
- When disconnected and a new valid device appears in the list (including replugging the same or a different device), automatically connect to that newly appeared device.

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

- `device-connection`: Add automatic device selection and reconnection rules for startup, hot-plug, disconnect, multi-device, and fallback scenarios.

## Impact

- **Code**: `src/app.rs` (`refresh_devices`, new auto-connect helper), possibly unit tests in `app.rs` test module.
- **Specs**: Delta update to `device-connection` requirement set.
- **UX**: Device toolbar and session status will reflect auto-connected state without user pressing `d`.
- **Non-goals**: No OS-level USB hotplug listener; still relies on existing ~5s device list polling. No persistence of last-selected device across app restarts. No change to manual device picker behavior beyond automatic triggers.

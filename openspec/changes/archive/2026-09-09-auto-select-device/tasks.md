## 1. State and helpers

- [x] 1.1 Add `prev_devices: Vec<Device>` field to `OhmylogcatApp`, initialized empty in `new()` and test helpers
- [x] 1.2 Add helper `is_valid_device(d: &Device) -> bool` (state equals `device`, case-insensitive)
- [x] 1.3 Add helper to compute newly appeared valid serials by diffing `prev_devices` against the refreshed list
- [x] 1.4 Add helper to find the first valid device in list order

## 2. Auto-connect logic

- [x] 2.1 Implement `maybe_auto_connect(&mut self, previous_devices: &[Device])` following design decision order
- [x] 2.2 Wire `maybe_auto_connect` at end of successful `refresh_devices()`; snapshot `prev_devices` before replacing `self.devices`
- [x] 2.3 Ensure streaming path: active device removed with others remaining triggers fallback to first remaining valid device
- [x] 2.4 Ensure non-streaming path: newly appeared valid device triggers connect; idle + valid list triggers first-device connect
- [x] 2.5 Ensure streaming path: newly appeared device while active device still valid does not change selection

## 3. Tests

- [x] 3.1 Unit test: startup with empty list → no selection
- [x] 3.2 Unit test: first valid device appears while idle → auto-select and start stream
- [x] 3.3 Unit test: second device appears while streaming → selection unchanged
- [x] 3.4 Unit test: active device removed, another valid remains → fallback to remaining device
- [x] 3.5 Unit test: disconnected state, newly appeared device → reconnect to new device
- [x] 3.6 Unit test: disconnect preserves buffer (stream error / stop does not clear logs)

## 4. Validation

- [x] 4.1 Run `cargo test` and fix any failures
- [x] 4.2 Run `openspec validate auto-select-device --strict`

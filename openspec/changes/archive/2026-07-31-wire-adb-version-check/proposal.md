## Why

`cargo run` surfaces eight `dead_code` warnings. Most are test-only helpers left outside `#[cfg(test)]`, plus a few unused APIs. Separately, `check_adb_version` already exists and matches the device-connection requirement to verify adb before discovery/streaming, but nothing calls it—so a path that exists but is not a working adb binary fails later with a less clear error.

## What Changes

- Call `check_adb_version` after successful `resolve_adb_path` and before device discovery or log streaming; surface failures via the existing error path.
- Remove unused non-test dead code: `Engine::stats`, and production-facing helpers that are only used by unit tests (move test-only helpers under `#[cfg(test)]` or delete if fully redundant).
- Delete unused public helpers that are not needed for the wired path: keep `check_adb_version`; remove or cfg-gate the rest (`allocated_capacity`, `apply_filter`, `scroll_bottom_position`, `TextInput::from_text`, both `make_entry` helpers) per design (prefer delete when tests can use local helpers; prefer `#[cfg(test)]` when tests still need them).

## Capabilities

### New Capabilities

<!-- none -->

### Modified Capabilities

- `device-connection`: Clarify that adb availability is confirmed by a successful `adb --version` after path resolution, before device discovery and streaming.

## Impact

- `src/adb/mod.rs` — version check remains; call sites in `src/app.rs` (`refresh_devices`, `start_selected_device`) gain a check.
- `src/engine.rs`, `src/buffer/mod.rs`, `src/filter/mod.rs`, `src/ui/display.rs`, `src/ui/text_input.rs` — remove or cfg-gate unused symbols; adjust unit tests as needed.
- No new dependencies. User-visible change: clearer error when adb path resolves but the binary cannot run.

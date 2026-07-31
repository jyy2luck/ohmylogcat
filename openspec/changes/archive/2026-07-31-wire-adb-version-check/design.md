## Context

See proposal.md for motivation. Today `resolve_adb_path` only checks filesystem existence; `check_adb_version` is unused. Several symbols exist only for unit tests but sit outside `#[cfg(test)]`, which produces `dead_code` warnings on `cargo run`. `App::scroll_to_bottom` already inlines the logic of `scroll_bottom_position`.

## Goals / Non-Goals

**Goals:**

- Gate discovery and streaming on a successful version check after path resolve.
- Clear all eight current `dead_code` warnings without `#![allow(dead_code)]`.
- Keep existing unit tests green.

**Non-Goals:**

- Validating device connectivity, adb server health, or USB drivers.
- Refactoring `App::scroll_to_bottom` to call a shared helper (helper is deleted; production already has the logic).
- Changing settings UI or error display chrome beyond reusing `last_error`.

## Decisions

### 1. Where to call `check_adb_version`

Call it in `App::refresh_devices` and `App::start_selected_device` immediately after `resolve_adb_path` succeeds, before `list_devices` / `start_stream`.

**Why:** These are the two entry points that need a working adb. Periodic refresh already uses `refresh_devices`, so a broken binary surfaces without waiting for stream start.

**Alternative considered:** Fold version check into `resolve_adb_path`. Rejected — keeps path resolution cheap/pure and matches the existing split of functions.

**Alternative considered:** Check only on stream start. Rejected — discovery would still call a broken binary via `list_devices` with a worse error.

### 2. Dead code removal strategy

| Symbol | Action |
|--------|--------|
| `check_adb_version` | Keep and wire |
| `Engine::stats` | Delete (callers use events / `buffer_stats_self`) |
| `apply_filter` | Delete; keep `FilterCriteria::matches` tests; drop or rewrite `test_apply_filter` inline |
| `scroll_bottom_position` | Delete; move its unit tests to assert via the same algorithm inlined in the test module, or delete those tests if they only covered the helper (prefer keep coverage by inlining a private test helper under `#[cfg(test)]` in `display.rs`) |
| `TextInput::from_text` | Delete from production API; recreate as a local helper inside `text_input` tests |
| `RingBuffer::allocated_capacity` | Delete from production API; recreate as a local test helper or assert via other means in buffer tests |
| `make_entry` (buffer / filter) | Move into each module's `#[cfg(test)]` tests block |

**Why delete rather than `#[cfg(test)]` on public APIs:** User asked to remove the unused surface; production binaries should not carry unused `pub` methods. Test-local helpers stay private inside test modules.

### 3. Error handling

On version check failure, set `last_error` the same way path resolve / `list_devices` failures already do. Do not start stream; do not update device list on failed refresh.

## Risks / Trade-offs

- [Version check adds ~50–200ms spawn cost on each `refresh_devices` (every 5s)] → Acceptable for a TUI; if noisy later, cache "adb ok" until settings path changes.
- [Deleting `scroll_bottom_position` loses a named unit under test] → Mitigate by keeping equivalent assertions with a private `#[cfg(test)]` helper in `display.rs`.
- [Broken adb now fails earlier and more often in the status/error line] → Desired; clearer than stream spawn failures.

## Migration Plan

No data migration. Deploy by shipping the binary; behavior is additive (stricter gate). Rollback = previous binary.

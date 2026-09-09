## Context

Device discovery runs on a ~5 second poll via `refresh_devices()` in `OhmylogcatApp` (`src/app.rs`). It updates `self.devices` but does not change `selected_serial` or call `start_selected_device()`. Manual selection via the devices modal (`d` key) is the only connect path today.

Stream lifecycle: `start_stream()` clears the log buffer; `stop_stream()` and stream errors set `is_streaming = false` without clearing the buffer. See proposal.md for the user-facing motivation.

## Goals / Non-Goals

**Goals:**

- Implement auto-connect rules from the delta spec after each successful device list refresh.
- Detect newly appeared devices by diffing the previous and current device snapshots.
- Keep manual device picker behavior unchanged.

**Non-Goals:**

- OS-level USB hotplug events or faster polling intervals.
- Persisting last-selected device across restarts.
- Auto-connect suppression after the user explicitly chooses "none" in the device modal (deferred; not in spec).
- Preserving the log buffer when switching to a different device (existing `start_stream()` clear behavior remains).

## Decisions

### 1. Hook point: after `refresh_devices()` updates the list

**Choice:** Add a `maybe_auto_connect(previous_devices)` call at the end of `refresh_devices()` when listing succeeds.

**Rationale:** Single choke point covers startup, periodic poll, and manual refresh. Avoids duplicating logic in `open_devices()` or `tick()`.

**Alternative considered:** Separate watcher task — rejected as over-engineering for poll-based discovery.

### 2. Track previous device snapshot on `OhmylogcatApp`

**Choice:** Add `prev_devices: Vec<Device>` (or serial set) updated at the start of each successful refresh, before replacing `self.devices`.

**Rationale:** "Newly appeared" is defined as a serial present in the new list with state `device` that was absent from the previous snapshot (regardless of prior state). This covers first plug, replug, and swap-after-disconnect.

### 3. Decision order in `maybe_auto_connect`

```
if engine.is_streaming():
    if selected_serial missing from new list OR not valid:
        if other valid devices exist:
            select first remaining valid → start_stream   # Case 5
        else:
            leave disconnected (stream will end via error)  # Case 3 partial
    else:
        no-op                                              # Case 4 guard
else:  # not streaming — disconnected or never connected
    if newly_appeared_valid_devices non-empty:
        select first newly appeared → start_stream         # Case 3 replug, Case 1 plug
    elif selected_serial is None and valid devices exist:
        select first valid → start_stream                  # Case 2 startup (first refresh)
    else:
        no-op
```

**Note on Case 2:** On first refresh at startup with devices present, all devices are "newly appeared" relative to an empty `prev_devices`, so the newly-appeared branch covers both Case 1 (plug later) and Case 2 (already connected).

**Note on Case 5 vs Case 3:** When streaming and the active device vanishes but others remain, fallback runs immediately (Case 5) without waiting for a replug. When streaming and the active device vanishes with no others, the stream errors into disconnected state with logs preserved (Case 3).

### 4. Valid device filter

**Choice:** Only auto-select devices with `state == "device"` (case-insensitive).

**Rationale:** Avoids connecting to `unauthorized` or `offline` entries; aligns with spec scenarios.

### 5. Reuse `start_selected_device()`

**Choice:** Set `selected_serial` then call existing `start_selected_device()`.

**Rationale:** Reuses ADB version check and stream startup; consistent with manual selection path.

### 6. Streaming guard for Case 4

**Choice:** While `is_streaming()` is true, never auto-switch due to newly appeared devices; only handle active-device disappearance fallback.

**Rationale:** Matches "second device plugged in does not change selection."

## Risks / Trade-offs

- **[5s discovery delay]** → Device may not auto-connect until the next poll. Acceptable per non-goals; manual `d` + `r` still works.
- **[adb devices order]** → "First valid device" follows adb output order, not user preference. Documented; manual picker unchanged.
- **[Race: stream error vs refresh]** → Stream may set `is_streaming=false` before refresh detects removal. Disconnected + newly-appeared path still handles replug; fallback path handles multi-device unplug.
- **[Unauthorized then authorized]** → Device appears first as `unauthorized`, then as `device`; second appearance counts as newly valid if serial was not valid before. Auto-connects after user grants debugging.

## Migration Plan

No migration. Behavior change is forward-only on next release. Users who relied on manual-only connect gain automatic connect; manual override via `d` unchanged.

## Open Questions

_(none — user confirmed all multi-device cases)_

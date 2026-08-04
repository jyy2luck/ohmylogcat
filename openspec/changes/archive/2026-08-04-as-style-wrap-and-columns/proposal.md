## Why

Soft-Wrap currently chops long log lines from column 0, so continuation rows collide with the timestamp/pid/tid metadata and are hard to scan. Android Studio Logcat keeps metadata visually clear by hanging continuations under the message column and padding PID/TID into fixed columns. Matching that layout makes Ohmylogcat's wrap mode readable without changing filter or stream semantics.

## What Changes

- Pad PID and TID to a fixed display width in the formatted log line so Level/Tag/Message columns align across rows (same spirit as export's `{:5}` padding).
- When Soft-Wrap is on, wrap long lines with a hanging indent so continuation display rows start under the **message** column (after `{tag}: `), not at column 0.
- Keep Soft-Wrap off behavior as single-row + horizontal pan; only the visible layout of wrapped rows changes.
- Update wrap-height math, scroll/`wrap_skip`, mouse hit-testing, caret screen mapping, and selection highlights so they share one hang-indent wrap model (no drift between paint and interaction).

## Capabilities

### New Capabilities

<!-- none -->

### Modified Capabilities

- `log-display`: Soft-Wrap MUST hang-indent under the message column; formatted lines MUST use fixed-width PID/TID so columns align across entries.

## Impact

- `src/ui/format.rs` — PID/TID padding and a helper to compute message-column indent from a formatted line / entry
- `src/ui/display.rs` — hang-indent-aware wrap chunking and row-count
- `src/app.rs` — `draw_logs`, `entry_wrap_height`, follow/scroll wrap math
- `src/ui/selection.rs` — wrap-mode mouse ↔ `LogPos` and caret screen mapping
- Find / copy continue to use the formatted logical line (padded PID/TID included; hang-indent spaces are display-only and not part of copied text)
- Export already pads PID/TID; align UI width with that convention where practical
- Out of scope: treating hard `\n` inside messages (stack-trace continuations) as forced display breaks; word-boundary wrapping

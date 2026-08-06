## Why

Status bar copy is misleading and under-explained: "Live / 实时" reads like a generic real-time badge, but the flag only means the logcat stream is active (`is_streaming`). Metric numbers (`filtered/stored/capacity`, lines/s, memory) also lack short labels, and the throughput unit is hard-coded English.

## What Changes

- Rename the streaming indicator to **Streaming / 拉流中** (and Traditional Chinese **拉流中**), with idle as **Idle / 空闲 / 空閒**
- Append brief parenthetical labels after each numeric cluster: `(filtered/stored/max)`, rate, and memory — localized per UI language
- Localize the throughput unit (e.g. `lines/s` → `行/秒`) instead of leaving it English-only
- No change to the underlying metrics or how they are computed

## Capabilities

### New Capabilities

- (none)

### Modified Capabilities

- `log-buffer`: Clarify status-bar wording for the streaming indicator and require localized labels/units for filtered count, stored count, capacity, throughput, and memory estimate

## Impact

- `src/ui/i18n.rs` — status strings and new label/unit fragments for en / zh-Hans / zh-Hant
- `src/app.rs` (`draw_status`) — format string assembly
- `openspec/specs/log-buffer/spec.md` — status bar display requirement wording
- No engine, buffer capacity, or filter semantics changes

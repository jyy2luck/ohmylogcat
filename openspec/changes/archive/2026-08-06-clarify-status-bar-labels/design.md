## Context

See proposal.md for motivation. Today `draw_status` assembles a single format string with hard-coded `lines/s` / `MB` and i18n only for the live/idle, focus, and wrap hints (`status_live` ≈ "Live"/"实时"). Metrics themselves (`filtered_len`, `stats.count`, `stats.capacity`, `lines_per_sec`, `memory_estimate_mb`) are unchanged.

## Goals / Non-Goals

**Goals:**

- Align indicator copy with `is_streaming` semantics across en / zh-Hans / zh-Hant
- Add localized parenthetical labels after each numeric cluster
- Localize the throughput unit string

**Non-Goals:**

- Changing how rates or memory are estimated
- Relayouting the status bar into multiple widgets or truncation strategy beyond existing single-line Paragraph
- Renaming focus/wrap hints (already localized and clear enough)

## Decisions

### 1. Copy table (locked in explore)

| Role | en | zh-Hans | zh-Hant |
|------|----|---------|---------|
| Streaming on | `● Streaming` | `● 拉流中` | `● 拉流中` |
| Streaming off | `○ Idle` | `○ 空闲` | `○ 空閒` |
| Count labels | `(filtered/stored/max)` | `(筛选/已存/上限)` | `(篩選/已存/上限)` |
| Throughput | `{n:.0} lines/s(rate)` | `{n:.0} 行/秒(速率)` | `{n:.0} 行/秒(速率)` |
| Memory | `~{mb:.1}MB(mem)` | `~{mb:.1}MB(内存)` | `~{mb:.1}MB(記憶體)` |

**Alternatives considered:** "Active/已激活" (too vague vs pause); per-number parentheses `n(filtered)/…` (wider, more truncate risk) — rejected in favor of one cluster-level parenthesis.

### 2. i18n shape

Add small fragments on `UiStrings` rather than one mega template:

- Update `status_live` / `status_idle`
- Add `status_counts_hint`, `status_rate_unit` (includes unit + `(rate)` / `(速率)`), `status_mem_hint`

`draw_status` keeps assembling pieces so number formatting stays in Rust (`{:.0}`, `{:.1}`).

**Alternatives considered:** Single `format!` template with `{filtered}` placeholders in i18n — heavier and awkward with `:.0` formatting; rejected for this small change.

### 3. Scope of "MB"

Keep the `MB` unit literal in all locales (common in status UIs); only the parenthetical mem label is localized.

## Risks / Trade-offs

- **[Risk] Longer status line truncates on narrow terminals** → Mitigation: keep labels short (`mem` / `内存` / `記憶體`); accept same single-line Paragraph behavior as today
- **[Trade-off] Cluster-level vs per-number parentheses** → Cluster is denser; order is documented in the parenthetical so users can map positions

## Migration Plan

Pure UI string change; no settings migration. Rollback = revert the change.

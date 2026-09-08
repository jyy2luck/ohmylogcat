## Context

See proposal.md for motivation. Today `draw_status` is two slots: left counts, optional right `last_error` / `status_message`. Latest GitHub Release discovery already exists in `cli.rs` (`releases/latest` Location / effective URL, strip leading `v`) and is private, blocking, and used only after `ohmylogcat update`. The TUI event loop is a 50ms poll plus `app.tick()`; adb work already uses a dedicated Tokio runtime and `mpsc`.

Constraints that shape the approach: no `api.github.com` on this path; no new HTTP crate; shell chrome stays default-colored (theme accents are for levels / selection / find); Windows self-update file lock remains a CLI concern only.

## Goals / Non-Goals

**Goals:**

- Three-slot status row: counts | message | version cluster, with the cluster pinned right.
- Reuse the existing non-REST latest-tag discovery from a background worker so the UI thread never waits on curl / PowerShell.
- Compare numeric semver triples for the TUI hint only; leave CLI post-update equality messaging unchanged.

**Non-Goals:**

- In-TUI update, click-to-update, or opening a browser.
- Recurring checks, dismiss/snooze, or a settings toggle.
- Changing `ohmylogcat update` / `--version`.
- Adding `reqwest`, `semver`, or other network/version crates.

## Decisions

### 1. Background one-shot check via channel

On TUI construction, spawn a worker (`std::thread` or `spawn_blocking` on the existing runtime) that calls the shared latest-tag resolver with a hard timeout (about 5s). Send `Option<String>` (parsed version or none) on a `std::sync::mpsc` channel. `tick()` try-recv once; on `Ok(Some(v))` store `latest_remote` and never check again this session. Ignore hang-up / empty as "no update".

**Alternatives considered:** inline check before first draw (blocks startup); periodic re-check (nags, extra network); GitHub REST API (already rejected for rate-limit 403).

### 2. Share discovery, keep CLI comparison

Extract latest-tag resolution (and URL parse) so both CLI and TUI call one function. Do not change `select_update_outcome`'s string equality. TUI uses a small local triple compare: split on `.`, take three `u64`s, ignore `+` / `-` suffixes on the last numeric component. Unparsable either side → no suffix.

**Alternatives considered:** pull in the `semver` crate (heavier than this hint needs); reuse string inequality (false positives when local is ahead, e.g. `0.7.0-dev` vs `0.6.0`).

### 3. Status row layout and truncation

`draw_status` measures three pieces with `str_display_width`. Pad so the version cluster's last column aligns with the status row's right edge. If `left + gap + message + cluster > width`: omit update suffix; if still over, truncate / omit message; if still over, keep counts and as much of `版本-x.y.z` / `v x.y.z` as fits (counts win over cluster if the row is tiny).

Update suffix uses `Modifier::BOLD` only (no new theme color), so it reads as a hint without violating chrome color rules.

**Alternatives considered:** overlay chip in the log viewport (covers logs, collides with the new-logs chip); toolbar badge (fights shortcuts, dropped on narrow widths).

### 4. i18n fragments

Add catalog pieces rather than one mega template, e.g. current-version pattern and update-suffix pattern with a `{}` for the version string. Locale table (locked in explore):

| Locale | Current only | With update |
|--------|----------------|-------------|
| en | `v{ver}` | `v{ver} (update {latest})` |
| zh-Hans | `版本-{ver}` | `版本-{ver}（有更新{latest}）` |
| zh-Hant | `版本-{ver}` | `版本-{ver}（有更新{latest}）` |

`draw_status` fills `{ver}` / `{latest}` from Rust so number formatting stays out of the catalog.

### 5. Timeout and process spawn

The existing resolver shells out to `curl` (Unix) or PowerShell (Windows). The worker MUST bound wait time so a stalled DNS / proxy cannot leak a child for the whole TUI session: wrap the command with a timeout (Tokio `timeout` + `Command`, or kill the child after 5s). Failed spawn / non-zero / empty Location → silent none.

## Risks / Trade-offs

- **[Risk] PowerShell / curl hang without timeout** → Mitigation: decision 5; treat timeout as silent failure.
- **[Risk] GitHub changes `releases/latest` Location shape** → Mitigation: same parser as CLI; TUI degrades to current-version only.
- **[Risk] Narrow terminals clip the cluster** → Mitigation: drop suffix first; still show current version when possible.
- **[Trade-off] One check per session** → Stale if a release is published while the TUI stays open for hours; acceptable for v1.
- **[Trade-off] Bold suffix instead of color** → Weaker than a colored badge; keeps shell chrome default-colored.

## Migration Plan

No settings or install migration. Rollback is a revert. Cargo-installed and script-installed binaries both show the cluster; only the latter can `ohmylogcat update` after quitting.

## Open Questions

None that block implementation.

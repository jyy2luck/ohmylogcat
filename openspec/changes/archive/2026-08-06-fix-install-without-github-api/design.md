## Context

See proposal.md — Why. Today both install scripts resolve the asset via `GET https://api.github.com/repos/<repo>/releases/latest` (unauthenticated, ~60 req/hour/IP). `ohmylogcat update` on Unix runs `curl …/main/install.sh | sh`, so fixing `main` immediately helps already-shipped binaries. `src/cli.rs` also calls the same API after a successful install to compare versions. Windows deferred-replace behavior from the prior change must remain intact.

## Goals / Non-Goals

**Goals:**

- Install and update succeed when `api.github.com` is rate-limited or unreachable.
- Keep asset names, install dirs, PATH hints, and Windows lock/deferred flow unchanged.
- Keep “always fetch install script from default branch” so a main fix rescues old clients.
- Version comparison after update still works without the REST API when possible.

**Non-Goals:**

- Pinning install scripts to release tags (would break the “fix main → old update works” property).
- Authenticated GitHub API / embedding tokens in install scripts.
- Rewriting update as a fully in-binary downloader (no shell/PowerShell).
- Changing Linux “builds not published yet” exit behavior.

## Decisions

### 1. Primary download URL: `releases/latest/download/<asset>`

Construct:

`https://github.com/<owner>/<repo>/releases/latest/download/<asset-name>`

GitHub responds with redirects to the current latest tag asset, then the CDN. This does **not** consume REST API quota. Verified working for published darwin/windows assets.

**Alternatives considered:**

| Approach | Pros | Cons |
|----------|------|------|
| Keep API + User-Agent | Minimal diff | UA does not raise unauthenticated limit; still 403 when remaining=0 |
| API with optional `GITHUB_TOKEN` | Higher limit | Users shouldn’t need tokens for a public install |
| Scrape HTML of releases page | No API | Fragile markup |
| **latest/download (chosen)** | Simple, official, no quota | Needs known asset filename (already known per OS/arch) |

### 2. Drop API from the install hot path (no required fallback)

`install.sh` / `install.ps1` SHALL download via the latest/download URL only. Do not call `api.github.com` during normal install. Optional later: API fallback is unnecessary if the direct URL is the contract.

Retain existing local overrides where present (`OHMYLOGCAT_INSTALL_SOURCE` on Windows) and add a matching override on Unix if useful for tests, so CI never needs live GitHub.

### 3. Latest version discovery without REST API

After an immediate successful update, resolve latest tag by requesting (no follow, or use effective URL):

`https://github.com/<owner>/<repo>/releases/latest`

Parse the redirect `Location` (e.g. `.../releases/tag/v0.4.0`) and strip the leading `v` for comparison with `CARGO_PKG_VERSION`.

**Alternatives:** Keep API only for version check (still fails under the same rate limit that broke install); omit version messaging (worse UX).

### 4. Error messaging

If download fails, surface the HTTP status and the attempted URL. Do not claim “no release published” when the failure is network/403 from an unused API path.

### 5. Compatibility

Old binaries already invoke `main` scripts → merge to `main` is the migration for existing installs. No new release is *required* for 0.3.0→latest update to work once `install.sh` is fixed on `main`, though shipping a release that includes the `cli.rs` version-check fix is still desirable.

## Risks / Trade-offs

- **[Risk]** `latest/download` 404 if asset name wrong or release has no assets → **Mitigation:** keep existing OS/arch→asset mapping; fail with clear missing-asset message; CI/release workflow already publishes named assets.
- **[Risk]** Version redirect parsing breaks if GitHub changes Location shape → **Mitigation:** narrow parse (`/releases/tag/v…`); on failure keep today’s `InstalledUnverified` warning path.
- **[Risk]** Developers continue hammering API in ad-hoc curl during release testing → **Mitigation:** README/release note: prefer latest/download; optional checklist item.
- **[Trade-off]** Removing API means we no longer read `browser_download_url` from JSON — acceptable because asset names are deterministic.

## Migration Plan

1. Land script + `cli.rs` changes on `main`.
2. Users on ≤0.3.0 run `ohmylogcat update` (or re-run install curl|sh) and get the fixed script immediately.
3. Next tagged release ships the improved version-check binary.
4. Rollback: revert scripts on `main` (clients fetch main again); no client-side migration state.

## Open Questions

None that block implementation. Optional docs polish (README rate-limit note) can follow tasks without changing specs.

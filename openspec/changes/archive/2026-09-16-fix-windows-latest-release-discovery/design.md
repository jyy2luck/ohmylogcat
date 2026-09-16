## Context

See proposal.md for motivation. Latest-tag discovery already lives in `src/cli.rs` (`fetch_latest_release_version` → `resolve_latest_release_location` → `extract_version_from_release_url`) and is shared by the TUI hint and post-`update` verification. Unix uses `curl -fsSI` then `%{url_effective}`. Windows currently shells `powershell.exe` with `Invoke-WebRequest -MaximumRedirection 0` and tries to read `Location` from the thrown exception.

On Windows PowerShell 5.1 that exception is `InvalidOperationException` with no `Response` and no inner exception, so the catch rethrows, stdout is empty, and both call sites treat it as discovery failure. Windows 10+ ships `curl.exe`; the same HEAD request that Unix uses already returns `Location: .../releases/tag/vX.Y.Z` on the affected machine. Constraints: no `api.github.com` on this path; no new HTTP crate; keep the existing 5s child timeout; do not change install zip download.

## Goals / Non-Goals

**Goals:**

- Make Windows latest-tag discovery succeed when `releases/latest` is reachable, using a path that does not treat the first 302 as a hard error.
- Keep one shared resolver so the TUI suffix and CLI verification are fixed together.
- Preserve silent TUI failure and non-fatal CLI unverified messaging for genuine network / timeout / parse errors.

**Non-Goals:**

- Changing `install.ps1` / `install.sh` asset download URLs.
- In-TUI self-update, status-row layout, or version-triple comparison.
- Fixing PowerShell 5.1 GBK vs UTF-8 error-text mojibake (it goes away if the primary path is not PowerShell).
- Widening the 5s timeout, or adding a recurring TUI re-check.

## Decisions

### 1. Windows primary path: system `curl.exe`, same hops as Unix

Use `curl.exe` (explicit `.exe` so PowerShell’s `curl` alias never applies) with the existing Unix flags: first `-fsSI` and parse `Location`; if that header is missing, `-fsSL -o NUL -w %{url_effective}`. Feed the result through the existing tag parser. Keep `User-Agent: ohmylogcat` and `DISCOVERY_TIMEOUT` (5s).

**Why this over fixing `-MaximumRedirection 0`:** the 5.1 exception object has nothing to read; the catch block cannot be patched into correctness. **Why not `Invoke-WebRequest -Method Head` with follow:** it works, but still cold-starts `powershell.exe`, still fights encoding, and diverges from Unix. **Why not download the release HTML:** a followed GET is ~200KB for a version string; HEAD / first-hop Location stays tiny.

### 2. Fallback only when `curl.exe` cannot be spawned

If creating the curl process fails with program-not-found, fall back once to PowerShell: `Invoke-WebRequest -Method Head -UseBasicParsing` **with default / positive redirection**, then read `BaseResponse.ResponseUri.AbsoluteUri`. Never use `-MaximumRedirection 0`. If curl exists but the request fails or times out, do **not** spend a second 5s on PowerShell.

**Alternatives considered:** curl-only with no fallback (simpler; breaks pre-`curl.exe` Windows); always PowerShell HEAD-follow (works, slower, encoding-noisy).

### 3. Do not split TUI vs CLI resolvers

Both already call `fetch_latest_release_version`. Changing `resolve_latest_release_location` is sufficient. Do not change `select_update_outcome` or TUI `remote_is_newer`.

### 4. Tests stay local; no required live GitHub in unit tests

Keep and reuse `extract_version_from_release_url` / `location_header_value` tests. After unifying, compile `location_header_value` on Windows too (drop `cfg(not(windows))` around it). Add a regression guard that the Windows PowerShell fallback script, if present, does not pass `-MaximumRedirection 0`. Live `releases/latest` remains a manual / exploratory check, not a unit-test dependency.

## Risks / Trade-offs

- **[Risk] `curl.exe` missing on very old Windows** → Mitigation: decision 2 Head-follow fallback; if that also fails, existing silent / unverified behavior.
- **[Risk] GitHub stops sending a `Location` on HEAD** → Mitigation: existing effective-URL fallback; parser still requires `/releases/tag/`.
- **[Risk] Two tools on PATH named curl** → Mitigation: invoke `curl.exe` on Windows.
- **[Trade-off] Already-shipped 0.7.x / 0.8.0 binaries keep the bug until the user installs a build that contains this fix** → Mitigation: CLI `update` already delivers the binary (download path is fine); after this ships, the next newer release can surface in the TUI on Windows.
- **[Trade-off] Narrow-row layout may still drop the suffix first** → Out of scope; not why Windows currently never shows it.

## Migration Plan

No settings or install-script migration. Rollback is a revert of the Windows resolver. Users on a Release install pick up the fix with `ohmylogcat update` (ignore the current unverified warning) or by installing the next published asset.

## Open Questions

None.

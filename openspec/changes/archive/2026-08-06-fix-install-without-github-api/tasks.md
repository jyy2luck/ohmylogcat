## 1. Install scripts (non-API download)

- [x] 1.1 Change `install.sh` to download via `https://github.com/<repo>/releases/latest/download/<asset>` (remove required `api.github.com` call)
- [x] 1.2 Add optional `OHMYLOGCAT_INSTALL_SOURCE` (or equivalent) local override on Unix for tests, mirroring Windows
- [x] 1.3 Change `install.ps1` to use the same latest/download URL as the primary path; keep deferred-replace / result markers / existing local source override
- [x] 1.4 Improve download failure messages to show attempted URL / HTTP failure clearly (no false “no release” when the issue is network)

## 2. CLI latest-version discovery

- [x] 2.1 Replace `fetch_latest_release_version` REST API usage with public `releases/latest` redirect/Location tag parsing
- [x] 2.2 Keep `InstalledUnverified` behavior when discovery fails; add unit tests for tag extraction from Location / effective URL
- [x] 2.3 Remove or stop using `RELEASES_API` for the update hot path if unused

## 3. Verification & docs

- [x] 3.1 Smoke-check: with API rate-limited or blocked, install script still resolves and can fetch (or local-override install succeeds)
- [x] 3.2 Run relevant `cargo test` for `cli` lifecycle / version parsing
- [x] 3.3 Brief README note: install/update uses latest/download; API rate-limit 403 is not required for install
- [x] 3.4 Confirm old-client path: default-branch script change is sufficient for `ohmylogcat update` from ≤0.3.0 without a new binary first

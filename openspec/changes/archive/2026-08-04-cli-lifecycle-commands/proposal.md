## Why

Users who install via the published scripts can run the TUI but cannot check the binary version, refresh to the latest GitHub Release, or uninstall without manually hunting paths and PATH entries. Adding small CLI lifecycle commands closes that gap without changing the in-TUI experience.

## What Changes

- Add `ohmylogcat --version` / `-V` to print the package version and exit (before entering the TUI).
- Add `ohmylogcat --help` / `-h` with usage, lifecycle commands, and a short list of common TUI shortcuts (full table remains in README).
- Add `ohmylogcat update` that re-runs the existing platform install script (`install.sh` / `install.ps1` from the repo’s `main` branch) and reports when already at the latest release version.
- Add `ohmylogcat uninstall` that removes the Release-installed binary (and Windows user PATH entry when applicable), interactively asks whether to keep `settings.json` (default: keep), with `--yes`, `--keep-data`, and `--purge` for non-interactive use.
- Document the new commands in README; cargo-install users get a short message pointing them to `cargo` instead of silent wrong-path updates/uninstalls.

## Capabilities

### New Capabilities
- `cli-lifecycle`: CLI entry routing for version, help, update (via install scripts), and uninstall (binary + optional config) before TUI startup.

### Modified Capabilities
- (none)

## Impact

- `src/main.rs` (and likely a small new module such as `src/cli.rs`): parse args before `enable_raw_mode` / alternate screen.
- Reuse existing `install.sh` / `install.ps1` URLs; no change to release packaging required for MVP.
- Settings path via existing `settings::config_path` / config directory for uninstall purge/keep.
- README Install / usage sections; no new crate dependencies required if using `std::env::args` (optional: `clap` only if desired later).

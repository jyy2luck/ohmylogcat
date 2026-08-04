## 1. CLI scaffolding

- [x] 1.1 Add `src/cli.rs` with parse (`CliAction`) and wire `mod cli` from `main.rs`
- [x] 1.2 In `main`, handle non-TUI actions before `enable_raw_mode`; keep no-arg path launching TUI
- [x] 1.3 Implement `--version` / `-V` using `env!("CARGO_PKG_VERSION")`
- [x] 1.4 Implement `--help` / `-h` with usage, lifecycle commands, and abbreviated shortcuts
- [x] 1.5 Reject unknown top-level args with non-zero exit and pointer to `--help`

## 2. Update via install scripts

- [x] 2.1 Detect Release install vs other (`current_exe` vs default/`INSTALL_DIR` paths); print cargo guidance when not Release-installed
- [x] 2.2 Invoke platform install script (`install.sh` via curl|sh; `install.ps1` via PowerShell), preserving `INSTALL_DIR` when set
- [x] 2.3 After successful script run, compare package version to latest GitHub Release tag and print `Already up to date (x.y.z)` when equal

## 3. Uninstall

- [x] 3.1 Parse `uninstall` flags: `--yes`/`-y`, `--keep-data`, `--purge`
- [x] 3.2 Confirm binary removal (interactive or `--yes`); non-TTY requires `--yes`
- [x] 3.3 Prompt to keep settings when TTY and neither keep/purge flag; default keep; apply `--keep-data` / `--purge`
- [x] 3.4 Remove Release binary (Windows self-delete workaround if needed) and clean Windows user PATH entry when applicable
- [x] 3.5 On non-Release install path, print `cargo uninstall` guidance and exit non-zero

## 4. Docs and verification

- [x] 4.1 Document `--version`, `--help`, `update`, `uninstall` in README (Install / usage)
- [x] 4.2 Add unit/integration-style tests for arg parsing and help/version output (no TUI)
- [x] 4.3 Run `cargo test` and smoke `--version` / `--help` / unknown-arg locally

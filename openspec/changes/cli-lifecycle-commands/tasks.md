## 1. CLI scaffolding

- [ ] 1.1 Add `src/cli.rs` with parse (`CliAction`) and wire `mod cli` from `main.rs`
- [ ] 1.2 In `main`, handle non-TUI actions before `enable_raw_mode`; keep no-arg path launching TUI
- [ ] 1.3 Implement `--version` / `-V` using `env!("CARGO_PKG_VERSION")`
- [ ] 1.4 Implement `--help` / `-h` with usage, lifecycle commands, and abbreviated shortcuts
- [ ] 1.5 Reject unknown top-level args with non-zero exit and pointer to `--help`

## 2. Update via install scripts

- [ ] 2.1 Detect Release install vs other (`current_exe` vs default/`INSTALL_DIR` paths); print cargo guidance when not Release-installed
- [ ] 2.2 Invoke platform install script (`install.sh` via curl|sh; `install.ps1` via PowerShell), preserving `INSTALL_DIR` when set
- [ ] 2.3 After successful script run, compare package version to latest GitHub Release tag and print `Already up to date (x.y.z)` when equal

## 3. Uninstall

- [ ] 3.1 Parse `uninstall` flags: `--yes`/`-y`, `--keep-data`, `--purge`
- [ ] 3.2 Confirm binary removal (interactive or `--yes`); non-TTY requires `--yes`
- [ ] 3.3 Prompt to keep settings when TTY and neither keep/purge flag; default keep; apply `--keep-data` / `--purge`
- [ ] 3.4 Remove Release binary (Windows self-delete workaround if needed) and clean Windows user PATH entry when applicable
- [ ] 3.5 On non-Release install path, print `cargo uninstall` guidance and exit non-zero

## 4. Docs and verification

- [ ] 4.1 Document `--version`, `--help`, `update`, `uninstall` in README (Install / usage)
- [ ] 4.2 Add unit/integration-style tests for arg parsing and help/version output (no TUI)
- [ ] 4.3 Run `cargo test` and smoke `--version` / `--help` / unknown-arg locally

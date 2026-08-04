## Purpose

Provides out-of-TUI CLI commands so users can check the installed version, refresh via the published install scripts, uninstall the Release-installed binary, and see brief usage help without launching the terminal UI.

## ADDED Requirements

### Requirement: Version flag exits before TUI

The system SHALL accept `--version` and `-V` as top-level arguments. When either is present, the process SHALL print the application version derived from the package version to stdout and exit with status 0 without entering the TUI (no alternate screen, no raw mode).

#### Scenario: Print version with --version

- **WHEN** the user runs `ohmylogcat --version`
- **THEN** stdout contains the version string and the process exits 0 without starting the TUI

#### Scenario: Print version with -V

- **WHEN** the user runs `ohmylogcat -V`
- **THEN** the same version output and exit behavior as `--version` occurs

### Requirement: Help flag shows usage and common shortcuts

The system SHALL accept `--help` and `-h` as top-level arguments. When either is present, the process SHALL print usage for lifecycle commands (`--version`, `update`, `uninstall`, `--help`), a short list of common in-TUI keyboard shortcuts, and a pointer to README for the full shortcut table, then exit 0 without entering the TUI.

#### Scenario: Help lists lifecycle commands

- **WHEN** the user runs `ohmylogcat --help`
- **THEN** the output includes how to start the TUI with no args and documents `update`, `uninstall`, `--version`, and `--help`

#### Scenario: Help includes abbreviated shortcuts

- **WHEN** the user runs `ohmylogcat -h`
- **THEN** the output includes a short common-shortcut list and does not require launching the TUI

### Requirement: Update re-runs the platform install script

The system SHALL treat `update` as a top-level subcommand. On supported platforms, `ohmylogcat update` SHALL invoke the same install mechanism documented for fresh installs (`install.sh` on Unix-like hosts via curl|sh, `install.ps1` on Windows via PowerShell), targeting the repository’s published scripts on the default branch. After a successful update path, if the installed binary version matches the latest GitHub Release version, the system SHALL report that the installation is already up to date (including the version). The command SHALL NOT enter the TUI.

#### Scenario: Update when already latest

- **WHEN** the user runs `ohmylogcat update` and the latest release version equals the running package version after the install script completes (or the script installs the same version)
- **THEN** the process reports that it is already up to date with that version and exits successfully without starting the TUI

#### Scenario: Update installs newer release

- **WHEN** the user runs `ohmylogcat update` and a newer release is available
- **THEN** the install script runs and the binary is refreshed from the latest GitHub Release as for a fresh install

#### Scenario: Update for cargo-install users

- **WHEN** the user runs `ohmylogcat update` and the running binary is not under the Release install locations used by `install.sh` / `install.ps1`
- **THEN** the system prints guidance to use `cargo install` (or equivalent) instead of silently writing a second binary elsewhere

### Requirement: Uninstall removes Release binary and optionally config

The system SHALL treat `uninstall` as a top-level subcommand. For Release-script installs, it SHALL remove the installed `ohmylogcat` binary from the documented install location and, on Windows, remove the install directory from the user PATH when it was added by the installer. When a TTY is available and neither `--purge` nor `--keep-data` is set, the system SHALL ask whether to keep application settings (`settings.json` under the platform config directory), defaulting to keep. `--keep-data` SHALL keep settings without asking; `--purge` SHALL delete the settings file (and empty config directory when applicable) without asking to keep. `--yes` / `-y` SHALL skip the “proceed with uninstall” confirmation. The command SHALL NOT enter the TUI.

#### Scenario: Interactive uninstall keeps settings by default

- **WHEN** the user runs `ohmylogcat uninstall` in a TTY, confirms uninstall, and accepts the default when asked about settings
- **THEN** the Release-installed binary is removed and settings remain on disk

#### Scenario: Purge removes settings

- **WHEN** the user runs `ohmylogcat uninstall --purge` (with confirmation satisfied via `--yes` or interactive yes)
- **THEN** the binary is removed and the application settings file is deleted

#### Scenario: Keep-data skips the settings prompt

- **WHEN** the user runs `ohmylogcat uninstall --yes --keep-data`
- **THEN** the binary is removed, settings are retained, and no interactive settings prompt is shown

#### Scenario: Uninstall for cargo-install users

- **WHEN** the user runs `ohmylogcat uninstall` and the running binary is not under the Release install locations
- **THEN** the system prints guidance to use `cargo uninstall ohmylogcat` (or equivalent) instead of deleting an unrelated path

### Requirement: Default invocation still starts the TUI

When no lifecycle flag or subcommand is supplied, the system SHALL start the TUI as today. Unknown top-level arguments SHALL be rejected with a non-zero exit and a short error that points to `--help`, without entering the TUI.

#### Scenario: No arguments launches TUI

- **WHEN** the user runs `ohmylogcat` with no arguments
- **THEN** the TUI starts normally

#### Scenario: Unknown argument fails fast

- **WHEN** the user runs `ohmylogcat` with an unrecognized top-level argument
- **THEN** the process exits non-zero with an error referencing `--help` and does not enter the TUI

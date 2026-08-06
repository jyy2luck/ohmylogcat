## MODIFIED Requirements

### Requirement: Update re-runs the platform install script

The system SHALL treat `update` as a top-level subcommand. On supported platforms, `ohmylogcat update` SHALL invoke the same install mechanism documented for fresh installs (`install.sh` on Unix-like hosts via curl|sh, `install.ps1` on Windows via PowerShell), targeting the repository’s published scripts on the default branch. On Windows, when the target executable is locked by the currently running application or another process, the update path SHALL stage the downloaded executable and defer replacement until the lock is released instead of failing solely because the target is locked. The command SHALL clearly report whether the replacement completed immediately or was scheduled for deferred completion, and SHALL NOT report a deferred update as already installed. After an immediate successful update path, if the installed binary version matches the latest GitHub Release version, the system SHALL report that the installation is already up to date (including the version). The command SHALL NOT enter the TUI.

#### Scenario: Update when already latest

- **WHEN** the user runs `ohmylogcat update` and the latest release version equals the running package version after the install script completes (or the script installs the same version)
- **THEN** the process reports that it is already up to date with that version and exits successfully without starting the TUI

#### Scenario: Update installs newer release

- **WHEN** the user runs `ohmylogcat update` and a newer release is available
- **THEN** the install script runs and the binary is refreshed from the latest GitHub Release as for a fresh install

#### Scenario: Windows update target is locked by the running application

- **WHEN** the user runs `ohmylogcat update` on Windows and the installed executable is locked by the current `ohmylogcat` process
- **THEN** the new executable is staged outside the installation directory, a detached updater is scheduled to replace it after the current process exits, the command reports that the update was scheduled, and the command exits successfully without starting the TUI

#### Scenario: Windows deferred update completes

- **WHEN** the detached updater can observe that the locking process has exited and the staged executable is valid
- **THEN** it replaces the installed executable, removes its temporary files, and reports successful completion without requiring a system restart

#### Scenario: Windows deferred update cannot complete

- **WHEN** the detached updater cannot start or cannot replace the target after bounded retries
- **THEN** it reports an actionable failure, preserves the existing installed executable, and removes or clearly identifies its temporary state

#### Scenario: Windows installer is run while another application holds the target lock

- **WHEN** the published `install.ps1` is run while an existing `ohmylogcat.exe` process holds the target executable open
- **THEN** the installer stages the new executable and schedules deferred replacement instead of failing solely with a file-in-use error

#### Scenario: Update for cargo-install users

- **WHEN** the user runs `ohmylogcat update` and the running binary is not under the Release install locations used by `install.sh` / `install.ps1`
- **THEN** the system prints guidance to use `cargo install` (or equivalent) instead of silently writing a second binary elsewhere

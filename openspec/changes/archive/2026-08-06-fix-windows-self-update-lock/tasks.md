## 1. Windows installer staging and deferred replacement

- [x] 1.1 Add temporary-path and status-result handling to `install.ps1`, including unique pending and backup paths outside the installation directory.
- [x] 1.2 Replace the direct Windows overwrite with a recoverable replacement routine that preserves the existing executable until the staged executable is ready.
- [x] 1.3 Detect a locked target, persist the staged executable, generate a temporary helper script, and launch it detached with explicit paths and a bounded retry deadline.
- [x] 1.4 Implement helper success, retry exhaustion, cleanup, and backup restoration behavior, including actionable status/log output.
- [x] 1.5 Keep the unlocked install path synchronous and verify that direct installer execution and older release binaries can use the deferred fallback.

## 2. Rust update command result handling

- [x] 2.1 Capture and relay Windows installer output while recognizing explicit `installed` and `scheduled` results.
- [x] 2.2 Prevent deferred updates from being reported as already installed or fully updated, while preserving Unix behavior and Cargo-install guidance.
- [x] 2.3 Add focused tests for installer-result parsing and update-message selection.

## 3. Verification and documentation

- [x] 3.1 Add or update Windows smoke coverage for an unlocked install, a self-update with the running executable locked, helper completion, bounded failure, and rollback preservation.
- [x] 3.2 Exercise temporary and installation paths containing spaces under the supported PowerShell environments.
- [x] 3.3 Document immediate versus scheduled Windows update output and recovery guidance in the lifecycle/install documentation.
- [x] 3.4 Run formatting, Rust tests, and release-build checks, then validate the completed OpenSpec change.

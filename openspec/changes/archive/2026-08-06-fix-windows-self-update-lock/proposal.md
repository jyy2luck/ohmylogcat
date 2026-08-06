## Why

On Windows, `ohmylogcat update` invokes `install.ps1` synchronously while the running `ohmylogcat.exe` still holds an exclusive image-file lock. The installer therefore fails when it tries to overwrite the executable, even though the download and extraction succeeded. The update command should retain its one-command experience while deferring the final replacement until the running process has exited.

## What Changes

- Make the Windows installer detect when the target executable is locked.
- Stage the downloaded executable outside the installation directory before attempting replacement.
- When the target is locked, launch a detached temporary helper that waits for the current process to exit, retries the replacement, reports the final outcome, and cleans up temporary files.
- Keep direct installation synchronous when the target is not locked.
- Distinguish between an update that was installed immediately and one that was scheduled for deferred replacement.
- Keep the helper out of the permanent installation directory so existing installs remain a single executable.
- Preserve compatibility with older released binaries that invoke the latest `install.ps1` during `ohmylogcat update`.

## Capabilities

### New Capabilities

### Modified Capabilities

- `cli-lifecycle`: Change the Windows `update` behavior from failing on a locked running executable to completing immediately or scheduling a deferred replacement, with accurate user-facing status.

## Impact

- `install.ps1`: Windows staging, lock detection, detached helper startup, retry, cleanup, and status reporting.
- `src/cli.rs`: Update result handling and messaging where needed so deferred installation is not reported as already completed.
- `openspec/specs/cli-lifecycle/spec.md`: Add requirements and scenarios for locked Windows self-update.
- Tests and documentation for successful immediate updates, deferred updates, failed retries, and cleanup.
- No new runtime dependency is required; the helper can use PowerShell and the existing temporary-directory mechanism.

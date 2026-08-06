## Context

See `proposal.md` for the motivation. The current Windows path runs the published `install.ps1` synchronously from `src/cli.rs`; the installer downloads into a temporary directory and then copies directly over the running executable. Windows keeps the executable image locked until the current process exits. Older released binaries also fetch the current `install.ps1`, so the installer is the only place that can provide a bootstrap-compatible workaround.

## Goals / Non-Goals

**Goals:**

- Keep `ohmylogcat update` as the user-facing command.
- Make the published Windows installer safe when the target executable is locked.
- Let older released binaries benefit from the installer-side workaround.
- Preserve a synchronous direct install when the target is available.
- Keep all helper state temporary and leave no updater executable in the installation directory.
- Give the Rust command a reliable distinction between immediate installation and deferred scheduling.

**Non-Goals:**

- Replacing the single-file installation layout with a permanent launcher and version directory.
- Adding a separate packaged updater executable.
- Changing Unix update behavior or Cargo-install update guidance.
- Guaranteeing that a deferred update completes after the user forcibly terminates the helper or deletes its temporary state.

## Decisions

### 1. Put the lock fallback in `install.ps1`

The installer remains the source of truth for downloading and unpacking release assets. If the destination is locked, it stages the complete new executable in a durable temporary path, starts a detached helper, and returns a scheduled result. This is preferred over putting all orchestration in the new Rust binary because an older binary already invokes the latest published installer and can therefore receive the fix without a manual bootstrap update.

The normal path remains synchronous: when the target can be replaced, the installer completes the replacement before returning.

### 2. Generate a temporary helper only when needed

The helper logic is maintained as part of the repository’s installer logic, but its executable script file is created under the user’s temporary directory only for a deferred update. It is not copied to `%LOCALAPPDATA%\ohmylogcat` and is removed after success or terminal failure.

The helper receives the staged source path, target path, temporary status path, and retry deadline. It runs independently of the installer and does not wait on the original PowerShell process. Its replacement attempts are condition-based: it retries while the target remains unavailable and stops after a bounded deadline.

### 3. Use a staged, recoverable replacement

The downloaded executable is never used directly from the extraction directory because the installer’s cleanup runs when the initial process exits. Before scheduling the helper, the installer copies the complete executable to a unique pending path that survives that cleanup.

After the target becomes available, the helper:

1. Moves the existing target to a unique backup path.
2. Moves the staged executable into the target path.
3. Removes the backup only after the new target is in place.
4. Restores the backup if the second move fails.

This preserves the old executable when replacement cannot be completed and avoids requiring a system restart.

### 4. Add an explicit installer result

The installer emits a machine-detectable result in addition to human-readable progress:

- `installed`: replacement completed before the installer exited.
- `scheduled`: replacement was delegated to the detached helper.

The Rust update path captures and relays the installer output, recognizes these results, and avoids printing “already up to date” or “updated” as if a scheduled replacement had already completed. Older binaries that do not parse the marker still receive a successful installer exit and the human-readable scheduled message.

The helper writes its final success or failure to the temporary status/log path and reports to the originating console when that console remains available. Failure includes the target path and the remaining temporary state so the user has an actionable recovery path.

### 5. Keep the existing remote-script update boundary

`src/cli.rs` continues to invoke the published `install.ps1` from the repository’s default branch. No permanent helper is downloaded into the application directory, and no new runtime dependency is introduced. The change only adds the deferred Windows replacement protocol around the existing installer.

## Risks / Trade-offs

- **[Risk] Another process keeps the executable locked indefinitely** → Use a bounded retry deadline, preserve the old executable, and report the pending or failed paths.
- **[Risk] The temporary staged executable is removed or modified before the helper runs** → Use a unique random temporary path, verify that the staged file exists and is a valid executable before replacement, and include the path in failure diagnostics.
- **[Risk] The Rust process reports success before deferred replacement finishes** → Emit and parse the `scheduled` result separately from `installed`; only the helper reports final replacement completion.
- **[Risk] Old binaries cannot understand the new result marker** → Keep the installer exit status successful after scheduling and retain human-readable output; the actual replacement still completes independently.
- **[Risk] Temporary helper quoting or process inheritance behaves differently across PowerShell versions** → Keep helper arguments explicit, avoid interpolating unescaped paths into commands, and test paths containing spaces under Windows PowerShell and PowerShell 7.
- **[Trade-off] The helper is generated at runtime instead of installed permanently** → The installation remains simple and self-contained, but deferred-update diagnostics depend on temporary files until the helper finishes.

## Migration Plan

1. Publish the updated `install.ps1`; this bootstraps deferred replacement for existing release-installed binaries.
2. Release the Rust changes that parse immediate versus scheduled installer results.
3. Test an update from an old release while the target executable is running, then test a normal direct installation with no lock.
4. Remove stale helper state only after successful replacement or a bounded failure; the old installed executable remains a valid rollback state.

Rollback consists of publishing the previous installer and Rust binary. Any already-running helper either completes its staged replacement or fails without deleting the valid old executable.

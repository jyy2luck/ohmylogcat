## Context

See `proposal.md` — Why. Today `main` always enables raw mode and the alternate screen; there is no argument parsing. Fresh installs use `install.sh` / `install.ps1` against GitHub Releases; settings live at `dirs::config_dir()/ohmylogcat/settings.json` via `settings::config_path`.

## Goals / Non-Goals

**Goals:**
- Route lifecycle args before any terminal mode change.
- Reuse existing install scripts for `update` (no in-binary Releases downloader).
- Safe uninstall for script-installed binaries with explicit settings keep/purge behavior.
- Minimal help text that stays maintainable.

**Non-Goals:**
- Full in-binary self-update / GitHub API asset download.
- First-class `cargo install` update/uninstall automation.
- In-TUI `?` help panel (separate change if desired).
- Linux Release assets (install.sh already exits for Linux; update inherits that).

## Decisions

### 1. Parse args with `std::env::args`, no clap

**Choice:** Small manual match on first arg (`--version`/`-V`, `--help`/`-h`, `update`, `uninstall`, none → TUI). Uninstall flags parsed only after `uninstall`.

**Why:** Zero new dependencies; surface area is tiny.

**Alternatives:** `clap` — better help generation, heavier for four commands.

### 2. `update` = re-invoke published install scripts

**Choice:**
- macOS/Unix: `curl -fsSL https://raw.githubusercontent.com/jyy2luck/ohmylogcat/main/install.sh | sh` (honor `INSTALL_DIR` if already set in the environment).
- Windows: PowerShell equivalent of `irm .../install.ps1 | iex` (honor `$env:INSTALL_DIR`).

**Already latest:** After the script succeeds, compare `env!("CARGO_PKG_VERSION")` to the latest release `tag_name` (strip leading `v`) via GitHub Releases API; if equal, print `Already up to date (x.y.z).` The script still runs (idempotent overwrite) so users always get a consistent on-disk binary; messaging covers the no-op case. If preferred later, a pre-check can skip the download — not required for MVP if post-check messaging is clear.

**Cargo / unknown path:** If `std::env::current_exe()` is not under the documented Release install dirs (`~/.local/bin` or `$INSTALL_DIR` on Unix; `%LOCALAPPDATA%\ohmylogcat` or `$INSTALL_DIR` on Windows), print guidance and exit non-zero without running the script.

**Alternatives considered:** Embed downloader in Rust (rejected for MVP); only print curl instructions (rejected — user asked for a real `update` command).

### 3. Uninstall path detection + Windows PATH cleanup

**Choice:** Delete the expected Release binary path (same defaults as install scripts). On Windows, if the user PATH contains the install dir, remove that entry. Do not delete arbitrary `current_exe()` outside those locations.

**Self-delete:** On Windows, if the running exe is the file being deleted, spawn a short delayed `cmd`/`powershell` cleanup then exit; on Unix, `unlink` after exec is usually fine while mapped.

**Settings prompt:** stdin is a TTY → ask `Keep settings (settings.json)? [Y/n]` default Y. `--keep-data` / `--purge` override. Non-TTY without those flags → keep data (safe default) and skip prompt; require `--yes` only for the destructive “remove binary” confirmation when non-interactive… **Decision:** non-TTY requires `--yes` to proceed with binary removal; settings default to keep unless `--purge`.

### 4. Help content

**Choice:** Static English string in the CLI module: usage + ~8–10 shortcuts (`q`, `d`, Space, `c`, `f`, `t`/`m`, `/`, `s`, `e`) + “see README”. Do not sync from i18n tables in MVP.

### 5. Module layout

**Choice:** `src/cli.rs` with `enum CliAction { RunTui, Version, Help, Update, Uninstall { .. } }` and `run()` for non-TUI actions; `main` matches and only then enters existing TUI setup.

## Risks / Trade-offs

- **[Risk] install script URL/branch drift** → Mitigation: single constant matching README; scripts remain source of truth for paths/assets.
- **[Risk] update replaces binary while process still running (esp. Windows)** → Mitigation: script overwrite then instruct user to relaunch; document that `update` should be run from a normal shell, not from inside a replaced locked file when possible (Windows may need restart of the shell).
- **[Risk] false “cargo” classification if user copied binary elsewhere** → Mitigation: message is advisory; they can set `INSTALL_DIR` and use scripts manually.
- **[Trade-off] Running install script even when already latest** → Extra network; simplest correctness. Optional pre-check later.

## Migration Plan

- Ship in next release; existing installs gain commands after one manual update or re-run of install script.
- No settings schema change.
- Rollback: remove CLI module; old binaries unchanged.

## Open Questions

- None blocking: post-script “already latest” vs pre-check skip can be tuned during implementation without changing specs.

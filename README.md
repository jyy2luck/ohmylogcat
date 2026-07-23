# Oh My Logcat

A lightweight Android Logcat viewer as a **terminal UI (TUI)** — ratatui + crossterm, no GPU window, no WebView.

**Why?** Android Studio's Logcat and GPU-backed GUI shells burn memory during long sessions. Oh My Logcat keeps a ring buffer (default 200k lines) in a single process whose UI tax is essentially the terminal.

## Features

- Real-time `adb logcat -v threadtime` streaming
- Virtualized log viewport (only visible rows rendered)
- Level-based ANSI coloring (Error/Warn/Info/Debug/Verbose)
- Filter by Tag (case-sensitive), Message (case-insensitive), and Level (minimum)
- Pause/Resume, Clear, Follow (tail), Soft-Wrap preference (MVP: no-wrap + horizontal pan)
- Find in logs (`/` or Ctrl/Cmd+F) with highlight and next/prev navigation
- Configurable buffer presets: Light (50k), Normal (200k), Heavy (500k), Marathon (1M)
- Export filtered or all logs via in-TUI path prompt (default `ohmylogcat.log`)
- Settings persist under the user config directory

## Memory expectations

- **Idle / empty buffer**: near-zero UI overhead beyond the Rust process itself (no wgpu/egui atlas)
- **Under load**: grows with the buffer preset and stored log lines (full capacity still costs memory by design)

## Install

Prebuilt binaries are published on [GitHub Releases](https://github.com/jyy2luck/ohmylogcat/releases). You still need **adb** (see below).

### macOS

```bash
curl -fsSL https://raw.githubusercontent.com/jyy2luck/ohmylogcat/main/install.sh | sh
```

Installs to `~/.local/bin` (override with `INSTALL_DIR=...`).

Or download the matching `.tar.gz` from the latest release and place `ohmylogcat` on your `PATH`.

### Windows

在 **PowerShell** 中（推荐 Windows Terminal）：

```powershell
irm https://raw.githubusercontent.com/jyy2luck/ohmylogcat/main/install.ps1 | iex
```

默认安装到 `%LOCALAPPDATA%\ohmylogcat` 并写入用户 PATH（新开终端后生效）。可用 `$env:INSTALL_DIR=...` 覆盖安装目录。

若浏览器下载被拦截，也可用同一脚本绕过；手动方式见 [Releases](https://github.com/jyy2luck/ohmylogcat/releases/latest) 中的 `ohmylogcat-x86_64-pc-windows-msvc.zip`。

### From source

```bash
cargo install --git https://github.com/jyy2luck/ohmylogcat
# or: clone, then cargo build --release
```

## Prerequisites

- **Android SDK platform-tools** (`adb`) — on PATH or configured in Settings
- A capable terminal: macOS Terminal / iTerm2, **Windows Terminal** (ConPTY)
- macOS or Windows (Linux not officially tested)
- **Rust** — only if building from source ([rustup](https://rustup.rs/))

### ADB Setup

**macOS:**
```bash
brew install android-platform-tools
```

**Windows:**
Default SDK path is `%LOCALAPPDATA%\Android\Sdk\platform-tools\adb.exe`.
Set a custom path in Settings if adb is elsewhere.

## Build & Run

```bash
# Development (run inside a real terminal)
cargo run

# Release binary
cargo build --release
```

- macOS / Linux: `target/release/ohmylogcat`
- Windows: `target/release/ohmylogcat.exe`

```bash
# Tests
cargo test
```

### Publishing a release

Maintainers: push a version tag to trigger [.github/workflows/release.yml](.github/workflows/release.yml):

```bash
git tag v0.1.0
git push github v0.1.0
```

### Smoke checklist

**macOS (Terminal / iTerm2)**
- [ ] Launch shows toolbar / filters / log viewport / status bar
- [ ] Device modal (`d`), stream starts, Pause / Clear / Follow work
- [ ] Tag / Message / Level filters; Find (`/`); Export path; Settings persist

**Windows Terminal**
- [ ] Same keyboard flows as macOS
- [ ] Mouse click on toolbar labels (optional; keyboard must remain complete)
- [ ] Colors and alternate-screen restore on quit (`q` on top layer)

## Architecture

```
┌──────────────────────────────────────────────────┐
│  TUI (ratatui + crossterm)                       │
│  Toolbar · Filter · Virtual Log Viewport · Status│
└──────────────────────┬───────────────────────────┘
                       │ in-process calls + channel
┌──────────────────────▼───────────────────────────┐
│  Engine (Rust)                                   │
│  ADB stream · Parser · Ring buffer · Filter      │
└──────────────────────────────────────────────────┘
```

## Keyboard Shortcuts

| Shortcut | Context | Action |
|----------|---------|--------|
| `q` | Top layer (no modal, find closed) | Quit |
| `Space` | Logs | Pause / Resume |
| `c` | Logs | Clear buffer |
| `f` | Logs | Toggle Follow (tail) |
| `d` | Logs | Device list |
| `e` | Logs | Export menu |
| `s` | Logs | Settings |
| `w` | Logs | Toggle Soft-Wrap preference |
| `t` / `m` | Logs (top layer) | Open Tag / Message filter modal |
| `l` / `Tab` | Logs / Level | Focus Level filter (inline) |
| Click Tag / Message | Filter row | Open Tag / Message filter modal |
| `Esc` | Filter modal / Find / Modal | Close overlay · return to log viewport |
| `/` or Ctrl/Cmd+F | Logs | Open Find |
| `n` / `N` | Logs (find open) | Next / previous match |
| Enter / Shift+Enter | Find | Next / previous match |
| ↑↓ / j k / PgUp PgDn | Logs | Scroll |
| ←→ / h | Logs (wrap off) | Horizontal pan |
| Mouse click / wheel | Anywhere | Toolbar hits + scroll (when terminal supports mouse) |

## Configuration

Settings are stored as JSON under the platform config directory, e.g.:

- macOS: `~/Library/Application Support/ohmylogcat/settings.json`
- Windows: `%APPDATA%\ohmylogcat\settings.json`

# Oh My Logcat

A lightweight, standalone Android Logcat viewer built with **egui / eframe** (pure Rust, no WebView).

**Why?** Android Studio's built-in Logcat consumes significant JVM memory during long debug sessions. Oh My Logcat is a single-process desktop app with a ring buffer (default 200k lines) that can handle hours of debugging without WebView platform tax.

## Features

- Real-time `adb logcat -v threadtime` streaming
- Virtualized log list (fixed row height when Soft-Wrap is off)
- Level-based row coloring (Error/Warn/Info/Debug/Verbose)
- Filter by Tag (case-sensitive), Message (case-insensitive), and Level (minimum)
- Pause/Resume, Clear, Scroll to End (tail-follow)
- Soft-Wrap toggle with horizontal scroll when wrap is off
- Find in logs (Cmd/Ctrl+F) with highlight and next/prev navigation
- Configurable buffer presets: Light (50k), Normal (200k), Heavy (500k), Marathon (1M)
- Export filtered or all logs to `.log` file
- Settings persist under the user config directory

## Memory expectations

- **Idle / empty buffer**: significantly lower than a Tauri + WKWebView shell (no WebContent helper process for the UI)
- **Under load**: grows with the buffer preset roughly with stored log lines (full capacity still costs memory by design)

## Prerequisites

- **Rust** (stable toolchain) — [rustup](https://rustup.rs/)
- **Android SDK platform-tools** (`adb`) — on PATH or configured in Settings
- macOS or Windows (Linux not officially tested)

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
# Development
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

## Architecture

```
┌──────────────────────────────────────────────────┐
│  UI (egui + eframe)                              │
│  Toolbar · Filter · Virtual Log List · Status    │
└──────────────────────┬───────────────────────────┘
                       │ in-process calls + channel
┌──────────────────────▼───────────────────────────┐
│  Engine (Rust)                                   │
│  ADB stream · Parser · Ring buffer · Filter      │
└──────────────────────────────────────────────────┘
```

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Cmd/Ctrl+F | Open Find bar |
| Enter / Shift+Enter | Next / previous find match |
| Esc | Close Find bar |

## Configuration

Settings are stored as JSON under the platform config directory, e.g.:

- macOS: `~/Library/Application Support/ohmylogcat/settings.json`
- Windows: `%APPDATA%\ohmylogcat\settings.json`

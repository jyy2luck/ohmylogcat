# Oh My Logcat

A lightweight, standalone Android Logcat viewer built with Tauri v2.

**Why?** Android Studio's built-in Logcat consumes significant JVM memory during long debug sessions. Oh My Logcat is an independent process with a ring buffer (default 200k lines) that can handle hours of debugging without memory bloat.

## Features

- Real-time `adb logcat -v threadtime` streaming
- Virtual-scrolled log list (react-virtuoso) — smooth even with 200k+ lines
- Level-based row coloring (Error/Warn/Info/Debug/Verbose)
- Filter by Tag (case-sensitive), Message (case-insensitive), and Level (minimum)
- Pause/Resume, Clear, Scroll to End
- Configurable buffer presets: Light (50k), Normal (200k), Heavy (500k), Marathon (1M)
- Export filtered or all logs to `.log` file
- Settings persist between sessions

## Prerequisites

- **Android SDK platform-tools** (`adb`) — must be installable via PATH or configured in settings
- macOS or Windows (Linux not officially tested)

### ADB Setup

**macOS:**
```bash
# Homebrew
brew install android-platform-tools

# Or manual SDK install sets up PATH
```

**Windows:**
The default SDK path is `%LOCALAPPDATA%\Android\Sdk\platform-tools\adb.exe`.
Set a custom path in Settings if adb is elsewhere.

## Build & Run

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri dev

# Build production binaries
npm run tauri build
```

The macOS build produces a `.dmg` file in `src-tauri/target/release/bundle/dmg/`.
Windows builds (cross-compiled via GitHub Actions) produce `.msi`/`.exe` in CI artifacts.

## Architecture

```
┌──────────────────────────────────────────────────┐
│  Frontend (React + TypeScript + Tailwind)        │
│  ┌──────────┬─────────────┬───────────────────┐  │
│  │ Toolbar  │ Filter Bar  │ Virtual Log List  │  │
│  │ Device │ │ Tag/Msg/Lvl │ (react-virtuoso)  │  │
│  │ Pause   │ │             │                   │  │
│  │ Clear   │ │             │                   │  │
│  │ Export  │ │             │                   │  │
│  └──────────┴─────────────┴───────────────────┘  │
│  Status Bar: [●] Count/Capacity  lines/s  ~MB    │
└──────────────────────┬───────────────────────────┘
                       │ Tauri IPC (events + commands)
┌──────────────────────▼───────────────────────────┐
│  Backend (Rust)                                   │
│  ┌──────────┬──────────┬──────────┬────────────┐  │
│  │ ADB      │ Parser   │ Ring     │ Filter     │  │
│  │ Module   │ (thread  │ Buffer   │ (Tag/Msg/  │  │
│  │          │  time)   │ (config  │  Level)    │  │
│  │          │          │  cap)    │            │  │
│  └──────────┴──────────┴──────────┴────────────┘  │
└──────────────────────────────────────────────────┘
```

## Keyboard Shortcuts

TBD — v1.1 feature.

## License

MIT

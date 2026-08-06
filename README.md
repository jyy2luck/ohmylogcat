# Oh My Logcat

A lightweight, single-process Android Logcat viewer for the terminal. Oh My Logcat is built with Rust, ratatui, and crossterm: it uses the terminal directly instead of opening a GPU-backed window or an embedded WebView.

The application is designed for long-running sessions. It keeps a bounded ring buffer, renders only the visible portion of the log, and leaves the UI overhead close to the cost of the terminal itself.

## Highlights

### Streaming and display

- Streams `adb logcat -v threadtime` from the selected Android device or emulator.
- Parses and displays timestamp, PID, TID, level, tag, and message fields.
- Uses a virtualized log viewport that remains responsive with large buffers.
- Keeps a fixed-capacity ring buffer and discards the oldest entries when the limit is reached.
- Supports Light (50k), Normal (200k), Heavy (500k), Marathon (1M), and Custom buffer capacities.
- Aligns PID and TID in fixed-width columns to match threadtime output.
- Applies Android Studio Logcat level colors, with light or dark accents selected from the host terminal background.
- Keeps shell chrome on the terminal's default colors and does not expose a user-selectable theme mode.

### Filtering and search

- Filters by Tag using case-sensitive substring matching.
- Filters by Message using case-insensitive substring matching.
- Filters by minimum Level: All, Verbose, Debug, Info, Warn, or Error.
- Combines Tag, Message, and Level filters with AND logic while the stream continues running.
- Searches the currently visible logs with `/` or Ctrl/Cmd+F without hiding non-matching entries.
- Highlights all find matches and supports next/previous navigation with a match counter.

### Interaction

- Pauses and resumes display without unnecessarily terminating the ADB process.
- Follows the newest entry automatically, with a persistent Follow preference.
- Provides a Soft-Wrap preference with message-column hanging indentation; when disabled, long lines remain horizontally accessible.
- Provides editor-like caret navigation with arrow keys, Home, End, PageUp, and PageDown.
- Supports keyboard selection, mouse drag selection, double-click word selection, and triple-click logical-line selection.
- Copies selected text only when Cmd+C or Ctrl+C is pressed.
- Supports mouse interaction when the terminal reports mouse events, while keeping all core workflows available from the keyboard.

### Devices, export, and settings

- Enumerates connected ADB devices and emulators, including serial and state.
- Validates `adb --version` before device discovery and log streaming.
- Switches the active device from an in-terminal device modal.
- Exports filtered logs or the complete buffered log in threadtime format through an in-terminal path prompt.
- Persists ADB path, buffer capacity, Follow, Soft-Wrap, and language settings.
- Localizes TUI chrome in English, Simplified Chinese, or Traditional Chinese. Log content from the device is never translated.

## Memory model

- **Empty buffer:** storage grows on demand instead of pre-allocating the full capacity.
- **Active buffer:** memory usage grows with the selected capacity and the number of stored log entries.
- **At capacity:** the ring buffer retains only the newest entries.
- **Status bar:** shows current entries, configured capacity, approximate lines per second, and estimated memory usage.

## Install

Prebuilt binaries are published on [GitHub Releases](https://github.com/jyy2luck/ohmylogcat/releases). You still need **adb**; see [Prerequisites](#prerequisites).

### macOS

The installer supports both Apple Silicon and Intel Macs:

```bash
curl -fsSL https://raw.githubusercontent.com/jyy2luck/ohmylogcat/main/install.sh | sh
```

The default install directory is `~/.local/bin`. Override it with `INSTALL_DIR=...`:

```bash
curl -fsSL https://raw.githubusercontent.com/jyy2luck/ohmylogcat/main/install.sh | INSTALL_DIR="$HOME/bin" sh
```

You can also download the matching `.tar.gz` from the latest release and place `ohmylogcat` on your `PATH`.

### Windows

Run the following command in **PowerShell**. Windows Terminal is recommended:

```powershell
irm https://raw.githubusercontent.com/jyy2luck/ohmylogcat/main/install.ps1 | iex
```

The default install directory is `%LOCALAPPDATA%\ohmylogcat`. The installer adds it to the user `PATH`; open a new terminal for the updated `PATH` to take effect. Override the directory with `$env:INSTALL_DIR`:

```powershell
$env:INSTALL_DIR = "$env:LOCALAPPDATA\Tools\ohmylogcat"
irm https://raw.githubusercontent.com/jyy2luck/ohmylogcat/main/install.ps1 | iex
```

When `ohmylogcat update` is run while the current Windows executable is still
running, the installer prints a `scheduled` result and exits successfully. A
detached helper waits for the executable to become available, then replaces it
without requiring a restart. Close all running Oh My Logcat processes to let
the helper finish; the installer prints the temporary status-log path for
diagnostics. If the retry deadline is reached, the existing executable is
preserved and the status log identifies the staged file so the update can be
retried after the lock is released.

If the script cannot be used, download `ohmylogcat-x86_64-pc-windows-msvc.zip` from the [latest release](https://github.com/jyy2luck/ohmylogcat/releases/latest) and put `ohmylogcat.exe` on your `PATH`.

### Linux

Linux binaries are not published yet. Build from source:

```bash
cargo install --git https://github.com/jyy2luck/ohmylogcat
```

### From source

```bash
cargo install --git https://github.com/jyy2luck/ohmylogcat
# or: clone, then cargo build --release
```

## Lifecycle commands

These commands are available outside the TUI:

```bash
ohmylogcat --version    # print the package version (-V)
ohmylogcat --help       # show usage and common shortcuts (-h)
ohmylogcat update       # re-run the platform installer for the latest release
ohmylogcat uninstall    # remove a Release-script installation
```

Uninstall options:

- `--yes` / `-y`: skip the uninstall confirmation.
- `--keep-data`: keep `settings.json` without prompting.
- `--purge`: delete `settings.json` without prompting to keep it.

`update` and `uninstall` operate on installations created by `install.sh` or `install.ps1` (`~/.local/bin`, `%LOCALAPPDATA%\ohmylogcat`, or `INSTALL_DIR`). For Cargo installations, use `cargo install --force` and `cargo uninstall ohmylogcat`.

On Windows, an unlocked update reports `installed` and completes before the
command exits. If the running executable is locked, `update` reports that the
replacement was scheduled; it does not claim that the new version is already
installed. The detached helper completes after the process exits, or writes a
failure status with the preserved executable and temporary staged path when
its bounded retries are exhausted.

## Prerequisites

- **Android SDK Platform-Tools:** `adb` must be on `PATH` or configured in Settings. Oh My Logcat verifies the executable with `adb --version`.
- **A capable terminal:** macOS Terminal, iTerm2, or **Windows Terminal** (ConPTY). A real terminal is required; redirected or non-interactive output is not a TUI.
- **Supported release platforms:** macOS Intel, macOS Apple Silicon, and Windows x64. Linux is currently source-build only.
- **Rust:** required only when building from source ([rustup](https://rustup.rs/)).

### ADB setup

**macOS:**

```bash
brew install android-platform-tools
```

**Windows:**

The default SDK path is `%LOCALAPPDATA%\Android\Sdk\platform-tools\adb.exe`. Configure a custom path in Settings if `adb` is installed elsewhere.


## Architecture

```text
┌──────────────────────────────────────────────────┐
│  TUI (ratatui + crossterm)                       │
│  Toolbar · Filters · Virtual Log Viewport · Status│
└──────────────────────┬───────────────────────────┘
                       │ in-process calls + channel
┌──────────────────────▼───────────────────────────┐
│  Engine (Rust)                                   │
│  ADB stream · Parser · Ring buffer · Filters     │
└──────────────────────────────────────────────────┘
```

The TUI and engine run in one process. The engine reads and parses ADB output, stores structured entries in the ring buffer, and applies filters. The TUI renders only the visible rows and handles keyboard, mouse, modal, and clipboard interactions.

## Keyboard shortcuts

### Main shell

| Shortcut | Context | Action |
|----------|---------|--------|
| `q` / `Q` | Top layer (no modal, Find closed) | Quit and restore the terminal |
| `Space` | Logs | Pause or resume display |
| `c` | Logs | Clear the buffer |
| `f` | Logs | Toggle Follow (tail) |
| `d` | Logs | Open the device list |
| `e` | Logs | Open the export menu |
| `s` | Logs | Open Settings |
| `w` | Logs | Toggle Soft-Wrap |
| `t` / `m` | Logs, top layer | Open the Tag or Message filter modal |
| `l` / `Tab` | Logs / Level | Focus the Level filter |
| `/` or Ctrl/Cmd+F | Logs | Open Find |
| `n` / `N` | Find open | Go to the next or previous match |
| `Enter` / `Shift+Enter` | Find open | Go to the next or previous match |
| `←` `↑` `↓` `→` | Logs | Move the caret and keep it visible |
| `Shift` + arrows | Logs | Extend the selection from the caret |
| `Home` / `End` | Logs | Move to the start or end of the current display row |
| `PageUp` / `PageDown` | Logs | Move the caret by approximately one page |
| `Cmd+C` / `Ctrl+C` | Logs, selection active | Copy the selected text |
| `Esc` | Find or modal open | Close the active overlay, subject to filter/settings edit rules |

### Modal controls

| Shortcut | Context | Action |
|----------|---------|--------|
| `↑` / `↓` | Devices modal | Select a device |
| `Enter` | Devices modal | Activate the selected device |
| `r` | Devices modal | Refresh the device list |
| `1` / `f` | Export menu | Export filtered logs |
| `2` / `a` | Export menu | Export all buffered logs |
| `Esc` | Devices or export modal | Close the modal |
| `↑` / `↓` | Settings | Move between visible rows |
| `←` / `→` | Settings | Adjust buffer preset or language |
| `e` | Settings, ADB row locked | Enter ADB path edit mode |
| `r` | Settings, ADB row locked | Restore automatic ADB resolution |
| Printable keys / `Backspace` | Settings, editable row | Edit the active text field |
| `Enter` | Settings | Dismiss the modal; changes are already persisted |
| `Esc` | Settings | Exit ADB edit mode or dismiss the modal |
| Printable keys / editing keys | Tag or Message modal | Edit the live filter |
| `Enter` | Tag or Message modal | Keep the current filter and dismiss the modal |
| `Esc` | Tag or Message modal | Clear a non-empty filter, then dismiss when empty |

### Mouse controls

| Action | Context | Result |
|--------|---------|--------|
| Click | Log viewport | Place the caret and clear the selection |
| Double-click | Log viewport | Select an ASCII word (`[A-Za-z0-9_]`) |
| Triple-click | Log viewport | Select the complete logical log line |
| Left-button drag | Log viewport | Select text across visible log lines |
| Mouse wheel | Log viewport | Scroll without moving the logical caret |
| Click Tag or Message | Filter row | Open the corresponding filter modal |
| Click or wheel | Toolbar and filters | Activate controls or scroll when supported |
| Mouse hover | Log viewport / shell chrome | Use the terminal's text or default pointer shape |

Selecting text never changes the clipboard by itself. Copy is explicit with Cmd+C on macOS or Ctrl+C on Windows.

## Configuration

Settings are stored as JSON under the platform configuration directory:

- macOS: `~/Library/Application Support/ohmylogcat/settings.json`
- Windows: `%APPDATA%\ohmylogcat\settings.json`
- Linux: `~/.config/ohmylogcat/settings.json`

The Settings modal persists changes immediately. Available settings include:

- **ADB path:** automatic resolution or a custom executable path. Custom editing is locked until explicitly enabled.
- **Buffer capacity:** Light, Normal, Heavy, Marathon, or a custom line count.
- **Follow:** whether the log viewport follows the newest visible entry.
- **Soft-Wrap:** whether long entries wrap under the message column.
- **Language:** Auto, English, Simplified Chinese, or Traditional Chinese.

The application does not persist or expose a theme preference. Level accents are selected automatically from the terminal environment, while interaction highlights remain fixed.


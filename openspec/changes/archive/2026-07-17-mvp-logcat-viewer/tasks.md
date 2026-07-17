## 1. Project Scaffolding

- [ ] 1.1 Initialize Tauri v2 app with React + TypeScript + Tailwind CSS template
- [ ] 1.2 Configure project structure: `src-tauri/src/` modules (adb, parser, buffer, filter) and `src/` frontend components
- [ ] 1.3 Add dependencies: tokio, react-virtuoso, serde; verify `tauri dev` runs on macOS

## 2. Rust Backend — ADB & Parsing

- [ ] 2.1 Implement adb path resolution (PATH + platform hints) and version check command
- [ ] 2.2 Implement `list_devices` Tauri command wrapping `adb devices`
- [ ] 2.3 Implement logcat subprocess spawn: `adb -s <serial> logcat -v threadtime`, async stdout reader
- [ ] 2.4 Implement threadtime line parser → `LogEntry { timestamp, pid, tid, level, tag, message }`
- [ ] 2.5 Handle parser edge cases: malformed lines, multi-line continuations, message truncation at 4 KB

## 3. Rust Backend — Buffer & Filter

- [ ] 3.1 Implement fixed-capacity ring buffer with configurable size (default 200,000 lines)
- [ ] 3.2 Implement filter engine: tag substring, message case-insensitive substring, level minimum, AND logic
- [ ] 3.3 Implement batch emit via Tauri events (`log-batch`) every 50–100 ms
- [ ] 3.4 Implement Tauri commands: `get_filtered_logs`, `clear_buffer`, pause/resume state
- [ ] 3.5 Track buffer stats: count, lines/sec, memory estimate for status bar events

## 4. Frontend — Layout & Toolbar

- [ ] 4.1 Build main layout aligned with AS Logcat: toolbar, filter bar, log list, status bar (light theme only)
- [ ] 4.2 Implement device dropdown wired to `list_devices`, auto-refresh on interval
- [ ] 4.3 Implement toolbar: Pause, Clear, Scroll to End, Export, Settings gear
- [ ] 4.4 Implement filter bar: Tag input, Message input, Level dropdown (All/V/D/I/W/E)

## 5. Frontend — Log Display

- [ ] 5.1 Integrate react-virtuoso virtual list subscribed to `log-batch` events
- [ ] 5.2 Implement level-based row coloring (Error/Fatal, Warn, Info, Debug, Verbose)
- [ ] 5.3 Implement auto-scroll to end with scroll-up detection to disable auto-scroll
- [ ] 5.4 Implement Scroll to End button to jump to bottom and re-enable auto-scroll
- [ ] 5.5 Implement status bar: live indicator, count/max, lines/sec, memory estimate

## 6. Settings & Export

- [ ] 6.1 Implement settings dialog: adb path input, buffer preset selector (Light/Normal/Heavy/Marathon/Custom)
- [ ] 6.2 Persist settings to disk (Tauri store or config file) and restore on launch
- [ ] 6.3 Implement export filtered logs to `.log` file via native save dialog
- [ ] 6.4 Implement export all buffered logs option

## 7. Integration & Polish

- [ ] 7.1 Wire device switch: stop old stream, clear buffer, start new stream
- [ ] 7.2 Show error states: adb missing, no devices, device unauthorized
- [ ] 7.3 Manual test on macOS with physical device or emulator (2+ hour session, verify buffer retention)
- [ ] 7.4 Add README: prerequisites (adb), Windows adb path, build/run instructions

## 8. CI & Windows Delivery

- [ ] 8.1 Add GitHub Actions workflow: build on `windows-latest` and `macos-latest`
- [ ] 8.2 Upload artifacts: Windows `.msi`/`.exe` and macOS `.dmg`
- [ ] 8.3 Verify Windows build on clean machine or VM; document SmartScreen unsigned warning

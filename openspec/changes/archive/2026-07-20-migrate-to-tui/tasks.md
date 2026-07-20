## 1. Dependencies and skeleton

- [x] 1.1 Add `ratatui` + `crossterm` (and optional textarea helper); remove `eframe` / `egui` / `rfd` from `Cargo.toml`
- [x] 1.2 Narrow `tokio` features to what adb streaming needs (drop `full` if unused)
- [x] 1.3 Replace `main.rs` with TUI entry: raw mode, alternate screen, restore terminal on exit
- [x] 1.4 Introduce TUI app state shell wired to existing `Engine` + tokio runtime (empty layout first)

## 2. Layout and focus

- [x] 2.1 Draw main layout: toolbar / filter row / log viewport / status bar
- [x] 2.2 Implement `Focus` state machine (Logs, Tag, Message, Level, Find, Modal)
- [x] 2.3 Focusable Tag and Message inputs; Esc/Tab return to log viewport; debounce refilter
- [x] 2.4 Level control (cycle or small popup) wired to `Engine::set_filter`

## 3. Toolbar and streaming controls

- [x] 3.1 Toolbar labels + shortcuts: device, Pause, Clear, Follow, Export, Settings
- [x] 3.2 Optional mouse click hit-testing on toolbar controls when mouse events are enabled
- [x] 3.3 Device list modal + start/stop stream via existing adb/engine APIs
- [x] 3.4 Wire Pause / Clear / Follow (incl. scroll-away disables follow; persist follow pref)

## 4. Log viewport and status

- [x] 4.1 Virtual viewport: render only visible filtered rows; level-based ANSI colors
- [x] 4.2 Scroll keys / mouse wheel; horizontal pan when Soft-Wrap off
- [x] 4.3 Status bar: live indicator, count/capacity, lines/s, memory estimate
- [x] 4.4 Soft-Wrap: keep preference field; MVP no-wrap + horizontal access (wrap best-effort optional)

## 5. Find, export, settings

- [x] 5.1 Find UI (`/` and Ctrl/Cmd+F when available): highlight, n/N or next/prev, Esc close, suspend follow while active
- [x] 5.2 Export filtered / export-all via in-TUI path modal (default `ohmylogcat.log`)
- [x] 5.3 Settings modal: adb path + buffer presets; persist via existing settings module

## 6. Remove egui UI and docs

- [x] 6.1 Delete egui-specific `src/ui` / `app` drawing code; keep or relocate shared helpers (format line, etc.)
- [x] 6.2 Update README: TUI run instructions, memory expectations, keyboard cheat sheet; drop egui window narrative
- [x] 6.3 Smoke-check macOS terminal + note Windows Terminal acceptance checklist

## 7. Verification

- [x] 7.1 Manual: device connect, stream, filter Tag/Message/Level, pause/clear/follow
- [x] 7.2 Manual: find navigate, export path, settings persist across restart
- [x] 7.3 Confirm release binary has no egui/wgpu link; idle RSS sanity check vs previous egui build

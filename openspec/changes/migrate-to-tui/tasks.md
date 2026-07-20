## 1. Dependencies and skeleton

- [ ] 1.1 Add `ratatui` + `crossterm` (and optional textarea helper); remove `eframe` / `egui` / `rfd` from `Cargo.toml`
- [ ] 1.2 Narrow `tokio` features to what adb streaming needs (drop `full` if unused)
- [ ] 1.3 Replace `main.rs` with TUI entry: raw mode, alternate screen, restore terminal on exit
- [ ] 1.4 Introduce TUI app state shell wired to existing `Engine` + tokio runtime (empty layout first)

## 2. Layout and focus

- [ ] 2.1 Draw main layout: toolbar / filter row / log viewport / status bar
- [ ] 2.2 Implement `Focus` state machine (Logs, Tag, Message, Level, Find, Modal)
- [ ] 2.3 Focusable Tag and Message inputs; Esc/Tab return to log viewport; debounce refilter
- [ ] 2.4 Level control (cycle or small popup) wired to `Engine::set_filter`

## 3. Toolbar and streaming controls

- [ ] 3.1 Toolbar labels + shortcuts: device, Pause, Clear, Follow, Export, Settings
- [ ] 3.2 Optional mouse click hit-testing on toolbar controls when mouse events are enabled
- [ ] 3.3 Device list modal + start/stop stream via existing adb/engine APIs
- [ ] 3.4 Wire Pause / Clear / Follow (incl. scroll-away disables follow; persist follow pref)

## 4. Log viewport and status

- [ ] 4.1 Virtual viewport: render only visible filtered rows; level-based ANSI colors
- [ ] 4.2 Scroll keys / mouse wheel; horizontal pan when Soft-Wrap off
- [ ] 4.3 Status bar: live indicator, count/capacity, lines/s, memory estimate
- [ ] 4.4 Soft-Wrap: keep preference field; MVP no-wrap + horizontal access (wrap best-effort optional)

## 5. Find, export, settings

- [ ] 5.1 Find UI (`/` and Ctrl/Cmd+F when available): highlight, n/N or next/prev, Esc close, suspend follow while active
- [ ] 5.2 Export filtered / export-all via in-TUI path modal (default `ohmylogcat.log`)
- [ ] 5.3 Settings modal: adb path + buffer presets; persist via existing settings module

## 6. Remove egui UI and docs

- [ ] 6.1 Delete egui-specific `src/ui` / `app` drawing code; keep or relocate shared helpers (format line, etc.)
- [ ] 6.2 Update README: TUI run instructions, memory expectations, keyboard cheat sheet; drop egui window narrative
- [ ] 6.3 Smoke-check macOS terminal + note Windows Terminal acceptance checklist

## 7. Verification

- [ ] 7.1 Manual: device connect, stream, filter Tag/Message/Level, pause/clear/follow
- [ ] 7.2 Manual: find navigate, export path, settings persist across restart
- [ ] 7.3 Confirm release binary has no egui/wgpu link; idle RSS sanity check vs previous egui build

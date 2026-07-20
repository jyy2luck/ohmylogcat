## 1. Project skeleton

- [ ] 1.1 Create root `Cargo.toml` binary crate (`ohmylogcat`) with `eframe`, `egui`, `tokio`, `serde`, `serde_json`, `rfd`, `dirs`
- [ ] 1.2 Add `src/main.rs` + empty `OhmylogcatApp` that opens an eframe window
- [ ] 1.3 Move `adb`, `parser`, `buffer`, `filter`, `engine` modules from `src-tauri/src` into root `src/`, stripping Tauri types (`AppHandle`, `Emitter`, `State`, commands)
- [ ] 1.4 Keep/adapt existing unit tests; ensure `cargo test` passes for parser/buffer/filter

## 2. Engine without IPC

- [ ] 2.1 Refactor `Engine` to expose direct methods (start/stop stream, pause, clear, set_filter, stats, export iterators) usable from the UI thread
- [ ] 2.2 Wire tokio runtime for adb ingest; deliver batches to the app via channel drained in `eframe::App::update`
- [ ] 2.3 Implement lazy-growth ring buffer (no full-capacity `None` prefill); update buffer tests for empty-compact + capacity wrap

## 3. Core UI (feature parity baseline)

- [ ] 3.1 Toolbar: device combo, Pause/Resume, Clear, Scroll to End toggle, Export, Settings entry
- [ ] 3.2 Filter bar: Tag, Message, Level controls; apply AND filter through Engine
- [ ] 3.3 Virtualized log list (fixed row height / Soft-Wrap off): only paint visible rows; level colors
- [ ] 3.4 Status bar: live indicator, count/capacity, lines/s, memory estimate
- [ ] 3.5 Device discovery + select device starts/stops logcat stream

## 4. Settings, export, persistence

- [ ] 4.1 Settings panel/window: adb path + buffer presets (Light/Normal/Heavy/Marathon/Custom)
- [ ] 4.2 Persist settings JSON under user config dir; restore on launch (new schema OK)
- [ ] 4.3 Export filtered / export-all via `rfd` save dialog in threadtime text format

## 5. Display polish (existing specs)

- [ ] 5.1 Soft-Wrap toggle with persistence; horizontal scroll when wrap off
- [ ] 5.2 Find bar (Cmd/Ctrl+F): case-insensitive highlight, next/prev, Esc close; suspends auto-scroll while active
- [ ] 5.3 Tail-following behavior per `log-display` (scroll-to-end toggle, break on scroll-up, survive clear/device switch)

## 6. Cut over docs and remove Tauri stack

- [ ] 6.1 Update README for egui build/run, memory expectations, drop npm/tauri instructions
- [ ] 6.2 Update CI to `cargo build --release` / `cargo test` on macOS + Windows (installers optional later)
- [ ] 6.3 Remove React `src/`, Vite, `src-tauri` Tauri scaffolding, and obsolete npm frontend deps once egui path is verified
- [ ] 6.4 Manual acceptance: launch without WebView helper processes; empty buffer compact; stream/filter/export/find/wrap smoke on macOS (and Windows if available)

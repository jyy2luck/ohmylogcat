## 1. Project skeleton

- [x] 1.1 Create root `Cargo.toml` binary crate (`ohmylogcat`) with `eframe`, `egui`, `tokio`, `serde`, `serde_json`, `rfd`, `dirs`
- [x] 1.2 Add `src/main.rs` + empty `OhmylogcatApp` that opens an eframe window
- [x] 1.3 Move `adb`, `parser`, `buffer`, `filter`, `engine` modules from `src-tauri/src` into root `src/`, stripping Tauri types (`AppHandle`, `Emitter`, `State`, commands)
- [x] 1.4 Keep/adapt existing unit tests; ensure `cargo test` passes for parser/buffer/filter

## 2. Engine without IPC

- [x] 2.1 Refactor `Engine` to expose direct methods (start/stop stream, pause, clear, set_filter, stats, export iterators) usable from the UI thread
- [x] 2.2 Wire tokio runtime for adb ingest; deliver batches to the app via channel drained in `eframe::App::update`
- [x] 2.3 Implement lazy-growth ring buffer (no full-capacity `None` prefill); update buffer tests for empty-compact + capacity wrap

## 3. Core UI (feature parity baseline)

- [x] 3.1 Toolbar: device combo, Pause/Resume, Clear, Scroll to End toggle, Export, Settings entry
- [x] 3.2 Filter bar: Tag, Message, Level controls; apply AND filter through Engine
- [x] 3.3 Virtualized log list (fixed row height / Soft-Wrap off): only paint visible rows; level colors
- [x] 3.4 Status bar: live indicator, count/capacity, lines/s, memory estimate
- [x] 3.5 Device discovery + select device starts/stops logcat stream

## 4. Settings, export, persistence

- [x] 4.1 Settings panel/window: adb path + buffer presets (Light/Normal/Heavy/Marathon/Custom)
- [x] 4.2 Persist settings JSON under user config dir; restore on launch (new schema OK)
- [x] 4.3 Export filtered / export-all via `rfd` save dialog in threadtime text format

## 5. Display polish (existing specs)

- [x] 5.1 Soft-Wrap toggle with persistence; horizontal scroll when wrap off
- [x] 5.2 Find bar (Cmd/Ctrl+F): case-insensitive highlight, next/prev, Esc close; suspends auto-scroll while active
- [x] 5.3 Tail-following behavior per `log-display` (scroll-to-end toggle, break on scroll-up, survive clear/device switch)

## 6. Cut over docs and remove Tauri stack

- [x] 6.1 Update README for egui build/run, memory expectations, drop npm/tauri instructions
- [x] 6.2 Update CI to `cargo build --release` / `cargo test` on macOS + Windows (installers optional later)
- [x] 6.3 Remove React `src/`, Vite, `src-tauri` Tauri scaffolding, and obsolete npm frontend deps once egui path is verified
- [x] 6.4 Manual acceptance: launch without WebView helper processes; empty buffer compact; stream/filter/export/find/wrap smoke on macOS (and Windows if available)

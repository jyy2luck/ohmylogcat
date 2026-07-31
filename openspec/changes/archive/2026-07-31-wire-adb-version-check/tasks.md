## 1. Wire adb version check

- [x] 1.1 In `App::refresh_devices`, after `resolve_adb_path` succeeds, call `check_adb_version`; on failure set `last_error` and skip `list_devices`
- [x] 1.2 In `App::start_selected_device`, after `resolve_adb_path` succeeds, call `check_adb_version`; on failure set `last_error` and do not `start_stream`

## 2. Remove unused production APIs

- [x] 2.1 Delete `Engine::stats` from `engine.rs`
- [x] 2.2 Delete `apply_filter` from `filter/mod.rs`; remove or rewrite `test_apply_filter` without that helper
- [x] 2.3 Delete `scroll_bottom_position` from `display.rs`; keep coverage via a private `#[cfg(test)]` helper used only by existing scroll-bottom tests
- [x] 2.4 Delete `TextInput::from_text`; add a local helper inside the `text_input` test module
- [x] 2.5 Delete `RingBuffer::allocated_capacity`; assert allocation behavior via a test-local helper or equivalent in buffer tests
- [x] 2.6 Move both `make_entry` helpers into their respective `#[cfg(test)]` modules

## 3. Verify

- [x] 3.1 `cargo test` passes
- [x] 3.2 `cargo build` / `cargo run` compiles with no `dead_code` warnings from the previous eight symbols

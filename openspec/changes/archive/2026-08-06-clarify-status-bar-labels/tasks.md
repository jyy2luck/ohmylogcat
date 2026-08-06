## 1. i18n strings

- [x] 1.1 Add `status_counts_hint`, `status_rate_unit`, and `status_mem_hint` fields to `UiStrings`
- [x] 1.2 Update en / zh-Hans / zh-Hant values for `status_live`, `status_idle`, and the three new fragments per design.md copy table
- [x] 1.3 Fix any compile/test fallout from the new `UiStrings` fields

## 2. Status bar rendering

- [x] 2.1 Update `draw_status` to assemble `indicator  filtered/stored/max{counts_hint}  {rate}{rate_unit}  ~{mb}MB{mem_hint}  focus  wrap  err` (no hard-coded `lines/s`)
- [x] 2.2 Manually verify English, Simplified Chinese, and Traditional Chinese status bar copy against the design table (streaming on/off)

## 3. Verification

- [x] 3.1 Run `cargo test` / `cargo check` and confirm no regressions

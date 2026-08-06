## Why

Soft-Wrap 模式下，光标在跨多显示行（wrap chunk）的日志条目上移动时有两处不符合"垂直/当前行"直觉的行为：`End` 会跨显示行跳到整条逻辑行末尾，`Home` 会跳回第一行行首；上下方向键移动时 `caret_preferred_col` 被无条件还原成原始值，导致从 level 列下移到续行行首后再上移时回到原 level 列而非正上方，破坏了"上下移动严格垂直"的预期。

## What Changes

- soft-wrap 下 `Home` 移动到当前 wrap chunk 的起点（`chunk_start`），而非整条逻辑行 col 0。
- soft-wrap 下 `End` 移动到当前 wrap chunk 的末尾：末 chunk 为 `chunk_start + chunk_len`（= `line_len`），非末 chunk 为 `chunk_start + chunk_len - 1`（当前行最后一个字符的左间隙，因该字符之后的间隙归属下一显示行），而非整条逻辑行末。
- 非 soft-wrap 模式 `Home`/`End` 行为不变（仍为 col 0 / `line_len`）。
- 上下方向键移动后，仅当实际到达的 display 列与 `caret_preferred_col` 不一致（发生 clamp）时才把 `caret_preferred_col` 更新为实际到达的 display 列；一致时保留原值。该逻辑对 soft-wrap 与非 soft-wrap 统一适用。
- 移除 `move_caret_vertical` 中前后两次无条件还原 `caret_preferred_col` 的做法。

## Capabilities

### New Capabilities
<!-- 无新增能力 -->

### Modified Capabilities
- `log-display`: `Home`/`End` 在 soft-wrap 下改为按当前 wrap chunk 边界定位；上下方向键的 preferred 列改为"仅 clamp 时更新"，使垂直移动遵循 display 列垂直规律。

## Impact

- `src/app.rs`：`move_caret_line_bound`、`move_caret_vertical`（及 `move_caret_vertical_wrapped`/`move_caret_vertical_nowrap` 的调用约定）。
- `src/ui/display.rs`：复用现有 `wrap_chunk_at_col`，预计无需新增公开函数。
- `src/ui/selection.rs`：无变更。
- 现有单元测试可能需要更新（`step_caret_horizontal` 等不受影响；垂直移动相关行为如有断言需同步）。

## Context

光标在日志视口的位置是逻辑 `(row, col)` gap 索引（`LogPos`）。soft-wrap 下一条逻辑行被 `WrapChunks`（`src/ui/display.rs`）切成多个 chunk，每个 chunk 占一个显示行；续行前 `hang` 列是 pad。`caret_preferred_col`（`src/app.rs`）存的是 **display 列**，由 `apply_caret_move` 通过 `display_col_of(new)` 落定。

当前两个相关函数：
- `move_caret_line_bound`（`src/app.rs`）：`Home`→col 0，`End`→`line_len`，与 soft-wrap 无关，导致续行上 Home/End 跨显示行。
- `move_caret_vertical`（`src/app.rs`）：在 `apply_caret_move` 前后两次无条件 `self.caret_preferred_col = preferred;`，使 preferred 列在垂直移动中被锁回原值，破坏"垂直"语义。

已有可复用工具：`wrap_chunk_at_col(s, width, indent, col) -> (chunk_index, chunk_start, chunk_len)`、`wrap_chunk_by_index`、`display_col_of`。

## Goals / Non-Goals

**Goals:**
- soft-wrap 下 Home/End 按当前 chunk 边界定位；非 soft-wrap 行为不变。
- 垂直移动遵循 display 列垂直：preferred 列"仅 clamp 时更新"，两种模式统一。
- 复用现有 `wrap_chunk_at_col` 等函数，避免新增公开 API。

**Non-Goals:**
- 不改变水平移动（Left/Right 跨行）、PageUp/PageDown、鼠标命中、find 跳转的现有语义。
- 不引入"软换行内 `\n` 字面换行"的处理（日志消息字面换行不在本次范围）。
- 不改变 `caret_preferred_col` 的存储语义（仍是 display 列）。

## Decisions

### 1. Home/End 在 soft-wrap 下用当前 chunk 边界

`move_caret_line_bound` 改为：当 `self.soft_wrap` 时，用 `wrap_chunk_at_col(line, width, indent, old.col)` 取当前 chunk（即 `old.col` 所渲染行的 chunk；边界间隙 `col == chunk_end` 归属下一 chunk 的行，与 `log_pos_to_screen_wrapped`/`mouse_to_log_pos_wrapped` 一致）：
- `Home` → `col = chunk_start`
- `End` → 末 chunk（`chunk_start + chunk_len == line_len`）时 `col = chunk_start + chunk_len`（= `line_len`，与现状一致）；非末 chunk 时 `col = chunk_start + chunk_len - 1`（当前行最后一个字符的左间隙，与右边缘鼠标点击一致）。

非末 chunk 不能用 `chunk_start + chunk_len`：该间隙（最后一个字符之后）按既有渲染约定属于**下一显示行**的起点，会让光标跳到续行行首。非末 chunk 行内可达的最右间隙是最后一个字符的左间隙。

非 soft-wrap 分支保持 `0` / `line_len`。

**为何不新增函数**：`wrap_chunk_at_col` 已返回所需三量，内联调用即可，避免在 `display.rs` 增加仅此处使用的包装。

**替代方案**：在 `display.rs` 新增 `wrap_chunk_end_for_col`。否决——无复用价值，徒增表面积。

### 2. preferred 列"仅 clamp 时更新"，去掉无条件还原

`move_caret_vertical` 改为：
1. 记 `old`、读 `preferred = self.caret_preferred_col`。
2. 算 `new`（soft-wrap 走 `move_caret_vertical_wrapped`，否则 `move_caret_vertical_nowrap`）。
3. 计算 `reached = self.display_col_of(new)`。
4. 若 `reached != preferred`（发生 clamp）：`self.caret_preferred_col = reached`；否则保持 `preferred`。
5. 调 `apply_caret_move(old, new, extend)`——它本就把 `caret_preferred_col` 设为 `display_col_of(new)`，与第 4 步结果一致；因此第 4 步主要是"显式化 clamp 判定"，最终值由 `apply_caret_move` 落定即可。

实现上最简：直接**移除** `move_caret_vertical` 中前后两次 `self.caret_preferred_col = preferred;`，让 `apply_caret_move` 的 `self.caret_preferred_col = self.display_col_of(new);` 单独负责。这样：
- clamp 时 `new` 的 display 列 ≠ preferred → preferred 被更新为 reached（自然成立）。
- 未 clamp 时 `new` 的 display 列 == preferred → preferred 不变（值相同）。

两种模式统一，因为 `display_col_of` 在非 soft-wrap 下即返回 `pos.col`，`move_caret_vertical_nowrap` 用 `preferred.min(line_len)` clamp，clamp 后 `display_col_of(new) = line_len < preferred`，preferred 被更新为 `line_len`；未 clamp时相等保持。

**为何不"每次都更新"**：会牺牲非 soft-wrap 下穿越短行的"记忆列"行为（长行列 50 → 短行末 → 再下到长行停在 10，回不到 50）。"仅 clamp 时更新"既满足用户"垂直"诉求，又保留标准编辑器的记忆列。

**替代方案**：在 `move_caret_vertical_wrapped`/`_nowrap` 内部返回是否 clamp。否决——`display_col_of(new) != preferred` 已是 clamp 的充分判据，无需改函数签名。

### 3. Shift（选区扩展）路径不受影响

`apply_caret_move` 在 `extend` 时走选区扩展，但仍会执行 `self.caret_preferred_col = self.display_col_of(new);`，与上述一致；Shift+Up/Down 的 preferred 行为与非 Shift 相同，无需特殊处理。

## Risks / Trade-offs

- [现有单测可能依赖"preferred 被还原"的隐式行为] → 实现后跑 `cargo test`，按需更新断言；`step_caret_horizontal` 等不涉及垂直 preferred 的测试不受影响。
- [非 soft-wrap 下用户预期"记忆列穿越短行"被改变] → 仅在 clamp（短行末）时更新；到达 preferred 的长行间移动仍保留记忆列，行为变化最小且符合"垂直"直觉。
- [Home/End 在末 chunk 与现状一致] → 末 chunk `chunk_start + chunk_len == line_len`，等价，无回归。

## Migration Plan

纯代码改动，无数据/配置迁移。实现后 `cargo test` + 手测：soft-wrap 长行上 Home/End 在续行停在 chunk 边界；level 列下移到续行行首再上移回到正上方。回滚即还原 `move_caret_line_bound` 与 `move_caret_vertical` 两处。

## Open Questions

无。

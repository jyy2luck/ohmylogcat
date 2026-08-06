## 1. Home/End 按当前 chunk 边界

- [x] 1.1 修改 `move_caret_line_bound`（`src/app.rs`）：soft-wrap 分支用 `wrap_chunk_at_col(line, width, indent, old.col)` 取当前 chunk，`Home`→`chunk_start`，`End`→末 chunk `chunk_start + chunk_len`（= `line_len`）/ 非末 chunk `chunk_start + chunk_len - 1`（当前行最右可达间隙）；非 soft-wrap 保持 `0`/`line_len`
- [x] 1.2 复用 `self.formatted_line_at(old.row)`、`self.viewport_width`、`self.entry_indent_at(old.row)` 取 wrap 参数，确保空行安全（`wrap_chunk_at_col` 对空行返回 `(0,0,0)`）
- [x] 1.3 验证末 chunk 上 `End` 结果等于 `line_len`（与现状一致，无回归）

## 2. 垂直移动 preferred 列仅 clamp 时更新

- [x] 2.1 移除 `move_caret_vertical` 中 `apply_caret_move` 前后两次 `self.caret_preferred_col = preferred;`，改由 `apply_caret_move` 的 `self.caret_preferred_col = self.display_col_of(new);` 单独落定
- [x] 2.2 确认 `move_caret_vertical_wrapped`/`_nowrap` 无需改签名：clamp 判据即 `display_col_of(new) != preferred`
- [x] 2.3 检查 Shift（`extend`）路径：`apply_caret_move` 在 extend 时仍执行 preferred 落定，行为与非 Shift 一致

## 3. 测试与验证

- [x] 3.1 `cargo test` 全量跑通；更新/新增 `move_caret_vertical` 与 `move_caret_line_bound` 相关断言
- [x] 3.2 新增单测：soft-wrap 续行上 `End` 停在当前 chunk 末、`Home` 停在当前 chunk 起点
- [x] 3.3 新增单测：level 列下移到续行行首（clamp）后上移回到正上方（display 列对齐），preferred 列被更新
- [x] 3.4 新增单测：非 soft-wrap 穿越短行时 clamp 到行末、preferred 更新为行末列；回到长行后从新 preferred 继续垂直
- [x] 3.5 手测：soft-wrap 长行 Home/End/上下方向键在续行上行为符合预期，`cargo run` 验证

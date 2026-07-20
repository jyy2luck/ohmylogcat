## Why

Scroll to End 当前在部分场景下表现像「点一次滚一次」，而非 Android Studio Logcat 的持久「跟随尾部」开关。过滤刷新、设备切换或 Virtuoso `followOutput` 边界情况下，开关虽为 ON 却可能不再贴底，与用户对 AS Logcat 的预期不一致。

## What Changes

- 将 Scroll to End 明确为持久 toggle：ON 时在实时新日志、过滤条件变更、切换设备、Clear 后新日志等场景均保持滚到最新可见条目
- 保留 toggle 交互：OFF→ON 时滚到底并开启；ON→点击 时关闭；用户向上滚动时自动关闭
- 开关状态持久化到 localStorage，重启后恢复（默认 ON）
- 增强 LogList 滚动跟随实现（`followOutput` 回调 + entries 变化兜底），避免仅 append 场景生效
- Find 激活时仍暂停 tail-following（现有行为保留）
- 按钮 pressed/unpressed 视觉对齐 AS 留待后续样式迭代；本 change 聚焦行为与持久化

## Capabilities

### New Capabilities

（无）

### Modified Capabilities

- `log-display`: 将 Scroll to End 从「点击跳底并恢复自动滚动」的一-shot 描述，更新为持久 toggle 语义；补充过滤刷新、设备切换、持久化等场景要求；细化 auto-scroll 与 toggle 的关系

## Impact

- `src/App.tsx` — 使用新的 `useScrollToEnd` hook 替代裸 `useState`
- `src/hooks/useScrollToEnd.ts` — 新增，localStorage 持久化
- `src/components/LogList.tsx` — `followOutput` 回调与 entries 变化兜底滚动
- `src/components/Toolbar.tsx` — 行为不变，title/语义与 spec 对齐
- `openspec/specs/log-display/spec.md` — 需求更新（通过 delta spec 归档时合并）

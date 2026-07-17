## Context

ohmylogcat 已有 `autoScrollToEnd` 状态、Toolbar 高亮与 Virtuoso `followOutput="auto"`，但在 `log-snapshot`（过滤刷新）、设备切换等同长度或全量替换场景下，贴底行为不稳定；开关也未持久化。用户要求行为对齐 Android Studio Logcat 的「跟随尾部」toggle。

当前相关文件：`App.tsx`（状态与 toggle handler）、`LogList.tsx`（Virtuoso + atBottomStateChange）、`Toolbar.tsx`（按钮 UI）、`useSoftWrap.ts`（localStorage 持久化参考）。

## Goals / Non-Goals

**Goals:**

- Scroll to End 作为持久 tail-following 开关：ON 时在 append、过滤刷新、设备切换、Clear 后新日志等场景保持贴底
- Toggle 交互：OFF→ON 滚底并开启；ON→点击关闭；向上滚自动关闭
- localStorage 持久化，默认 ON
- 可靠滚动：`followOutput` 回调 + entries 变化兜底

**Non-Goals:**

- 按钮 pressed/unpressed 视觉完全对齐 AS（后续样式 change）
- 改变 Find、Soft-Wrap、Pause/Clear 等其他 toolbar 行为
- Rust 端改动

## Decisions

### 1. 抽取 `useScrollToEnd` hook

**选择**：新建 `src/hooks/useScrollToEnd.ts`，模式与 `useSoftWrap` 一致。

```ts
STORAGE_KEY = "ohmylogcat.scrollToEnd"
default: true
```

**理由**：持久化逻辑与 App 解耦，toggle 时写 localStorage，读取失败静默降级。

**备选**：内联 App.tsx — 与 Soft-Wrap 不一致，不利测试。

### 2. 双层滚动保障

**选择**：

1. **Virtuoso `followOutput` 回调**：`( ) => enabled && !findActive ? "auto" : false` — 开关 ON 时不依赖 `isAtBottom`，避免 `scrollToIndex` 后内部状态不同步导致只滚一次。
2. **`useEffect` 兜底**：当 `entries` 变化（length 或末项 identity）且 tail-following ON 且 find 未激活时，调用 `scrollToIndex({ index: "LAST", align: "end" })`。

**理由**：

- 标量 `followOutput="auto"` 默认仅在已在底部时跟随；回调形式可强制跟随。
- Virtuoso issue #555：`totalCount` 不变的全量替换不触发 followOutput；过滤前后条数相同时需要 effect。

**备选**：仅 effect、不用 followOutput — 高频 append 可能抖动；双层更稳。

### 3. 向上滚自动关闭

**选择**：保留 `atBottomStateChange` → `onAutoScrollToEndChange(false)`，并同步写 localStorage。

**理由**：与 AS 一致；用户手动离开尾部即退出 follow 模式。

### 4. Clear / 切换设备不重置开关

**选择**：tail-following 状态独立于 `entries` 清空；列表空时不 scroll，有数据且 ON 时贴底。

**理由**：用户明确 AS 在这些操作后保持 follow 意图。

### 5. Find 暂停 follow

**选择**：`followOutput` 与 effect 均检查 `!findActive`（与现有一致）。

**理由**：搜索时不应把视图拽回尾部。

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| effect + followOutput 双重 scroll 造成闪烁 | effect 用 `requestAnimationFrame` 或仅在非 append 场景（snapshot）触发；或 debounce |
| 高频 log-batch 性能 | followOutput 处理 append；effect 依赖项精确，避免每批重复 scroll |
| soft-wrap 行高变化导致 atBottom 误判 | 回调形式 followOutput 减少误判影响；向上滚检测仍保留 |
| localStorage 不可用 | try/catch 忽略，内存状态仍有效 |

## Migration Plan

- 无数据迁移；新 key `ohmylogcat.scrollToEnd`，缺省视为 `true`
- 部署：前端-only change，常规 `tauri dev` / build 验证

## Open Questions

- Find 关闭后是否在 tail-following ON 时立即滚底 — 本 change 可选实现，非必须

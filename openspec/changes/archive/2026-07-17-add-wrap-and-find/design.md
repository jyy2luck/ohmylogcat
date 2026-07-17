## Context

ohmylogcat 使用 Tauri 2 + React 19 + react-virtuoso 渲染日志。当前 `LogList` 每行用 Tailwind `truncate` 单行截断，无 in-page find，无键盘快捷键。FilterBar 通过 Rust `set_filter` 做后端过滤，语义是缩小数据集，不是 VSCode/AS 式的 find-in-view。

本 change 纯前端展示层，不动 Rust buffer/filter 引擎。

## Goals / Non-Goals

**Goals:**

- Toolbar Soft-Wrap toggle，默认关闭（不换行），对齐 AS Logcat「Use Soft Wraps」
- Cmd+F / Ctrl+F 打开 Find Bar，case insensitive substring 搜索当前可见 entries
- 匹配高亮 + 当前 match 强高亮 + 计数 + 上/下导航 + Esc 关闭
- Soft-Wrap 偏好持久化 localStorage
- 虚拟列表在大 buffer 下保持可交互

**Non-Goals:**

- Regex、大小写敏感开关、Find & Replace
- 跨 Filter 搜索全 buffer
- Tauri native menu accelerator
- 后端 filter 语义变更

## Decisions

### 1. Filter 与 Find 分离

**决定**：Find 纯前端，在 React `entries`（Filter 后）上扫描；不过滤、不 invoke Rust。

**理由**：与 AS Logcat 一致——Filter 缩小范围，Find 在范围内定位并高亮，保留上下文。

**备选**：复用 FilterBar Message 输入 → 拒绝，会隐藏非匹配行。

### 2. Soft-Wrap 实现

**决定**：

- OFF（默认）：`whitespace-nowrap` + 行/容器 `overflow-x-auto`，移除 `truncate`
- ON：`whitespace-pre-wrap break-all`，Virtuoso 变高 item

**理由**：AS 默认 Soft-Wrap 关；不换行时水平滚动而非 ellipsis 裁切。

**持久化**：`localStorage` key `ohmylogcat.softWrap`，默认 `false`。

### 3. Find 匹配与高亮

**决定**：

- `useFindInLog(entries, query)` 返回 `{ matches: { lineIndex, start, end }[], currentIndex }`
- 扫描：`line.toLowerCase().indexOf(query.toLowerCase())` 循环找所有 occurrence
- `LogList` `itemContent` 内按 offset 拆成 text + `<mark>` 片段
- 当前 match：`bg-yellow-300`；其他 match：`bg-yellow-100`

**理由**：~10k entries 全量扫描 <50ms；高亮仅在 Virtuoso 可见行渲染。

**跳转**：`virtuosoRef.scrollToIndex({ index: match.lineIndex, align: 'center' })`

### 4. Find Bar UI 与快捷键

**决定**：`FindBar.tsx` 浮层，置于 FilterBar 与 LogList 之间（或 LogList 顶部 sticky），默认隐藏。

| 快捷键 | 行为 |
|--------|------|
| Cmd+F / Ctrl+F | 打开并聚焦输入 |
| Enter | 下一个 match（循环） |
| Shift+Enter | 上一个 match（循环） |
| Esc | 关闭，清除 query 与高亮 |

**实现**：`App.tsx` 或 `useKeyboardShortcuts` 在 `document` 监听 `keydown`，`preventDefault` 拦截浏览器默认 find。

**平台检测**：`navigator.platform` 或 `event.metaKey` vs `event.ctrlKey`。

### 5. entries 变化时的 match 稳定性

**决定**：`entries` 或 `query` 变化时重算 matches；若 currentIndex 越界则 clamp 到 0 或最后。

**理由**：新日志涌入时 index 可能偏移，简单重算即可；不做复杂 anchor 逻辑。

### 6. 组件结构

```
App.tsx
  ├── useSoftWrap()          → boolean + toggle, localStorage
  ├── useFindInLog()         → query, matches, nav, open/close
  ├── useKeyboardShortcuts() → Cmd+F, Enter, Esc
  ├── Toolbar                → Soft-Wrap toggle
  ├── FindBar                → 输入、计数、▲▼、✕
  └── LogList                → wrapMode, findState, highlight render
```

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| Virtuoso 变高 item 滚动估算抖动 | 设置合理 `increaseViewportBy`；手动测试 10k+ 行 + wrap ON |
| 新日志导致 match 计数变化 | 重算 matches 并更新 UI；可接受 |
| `<mark>` 与 level 文字色冲突 | 用背景高亮，保留 level 前景色 |
| 超长单行 nowrap 性能 | 仅可见行渲染；必要时后续加 max-width 提示 |

## Migration Plan

- 无数据迁移；localStorage 新 key，缺省为 false
- 部署即生效，可回滚前端 commit

## Open Questions

（无——explore 阶段已确认：case insensitive、AS 对齐、默认不换行、单 change）

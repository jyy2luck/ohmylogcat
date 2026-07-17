## Why

MVP 日志列表使用 `truncate` 单行截断，长 stack trace 和 JSON 无法完整查看；也没有 in-page find，只能靠 Filter 缩小范围，会丢失上下文。Android Studio Logcat 提供 Soft-Wrap 切换和 Ctrl+F / Cmd+F 查找高亮，ohmylogcat 需要对齐这两类基础交互。

## What Changes

- 在 Toolbar 增加 **Soft-Wrap** 切换开关，默认关闭（不换行）；关闭时单行显示并支持水平滚动，开启时自动折行
- 新增 **Find Bar**（Cmd+F / Ctrl+F 打开），在当前 Filter 后的可见日志中做 case insensitive substring 搜索
- 搜索匹配项背景高亮，支持上/下跳转与匹配计数，Esc 关闭并清除高亮
- Find 与 Filter 分离：Find 不过滤、不隐藏非匹配行
- Soft-Wrap 偏好持久化到 localStorage

## Capabilities

### New Capabilities

（无——行为扩展归入既有 log-display capability）

### Modified Capabilities

- `log-display`: 新增 Soft-Wrap 切换、Find in Log 搜索高亮与键盘导航 requirements

## Impact

- **前端**：`Toolbar.tsx`、`LogList.tsx`、`App.tsx`；新增 `FindBar.tsx` 与 `useFindInLog` hook
- **后端**：无改动（Find 与 Wrap 均为纯前端展示层）
- **Spec**：`openspec/specs/log-display/spec.md` 追加 requirements
- **依赖**：无新 npm 依赖

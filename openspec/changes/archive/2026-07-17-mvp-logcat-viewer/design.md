## Context

ohmylogcat 是绿field 项目，目标是用 Tauri v2 实现独立 Android Logcat 查看器。用户痛点是 Android Studio 内置 Logcat 长时间调试内存占用高；期望交互对齐 AS 基础能力，独立进程运行，默认 ring buffer 200k 行以支持数小时 debug session。开发在 macOS，主要交付 Windows，不做 iOS。

## Goals / Non-Goals

**Goals:**

- 独立桌面 app，内存可控（默认 ~130 MB 总量）
- 实时 adb logcat 流，Rust 端解析与过滤
- UI 交互对齐 AS Logcat：设备选择、Pause/Clear/Scroll to end、Tag/Message/Level 过滤
- 200k 行默认可配置 ring buffer，状态栏显示 buffer 用量
- 导出过滤结果或全量 buffer
- macOS 开发 + GitHub Actions 构建 Windows/macOS 安装包

**Non-Goals:**

- iOS / syslog 支持
- 暗黑模式（v1 仅浅色主题）
- 多设备 tab、复杂 query 语法（`tag~:`、`package:`、`age:`）
- stack trace 点击跳源码
- 捆绑 adb、disk spill、正则过滤、过滤预设保存

## Decisions

### 1. 技术栈：Tauri v2 + Rust + React + TypeScript

**选择**：Tauri v2 后端 Rust，前端 React + Tailwind，虚拟列表用 `react-virtuoso`。

**理由**：Rust 适合高吞吐 log 解析与 ring buffer；Tauri 比 Electron 轻；React 生态成熟。LogAnalysis、Lazy Blacktea 已验证此组合。

**备选**：纯 Rust TUI（UX 差）、Electron（内存偏高）、egui（UI 灵活性不足）。

### 2. ADB 集成：子进程调用系统 adb

**选择**：MVP 通过 `std::process::Command` 执行 `adb -s <serial> logcat -v threadtime`，读取 stdout 行流。

**理由**：实现简单、兼容性好；Android 开发者机器几乎都有 adb。

**备选**：纯 Rust `adb_client` crate（v2 可考虑，减少 adb 路径依赖）。

**Windows 注意**：adb 默认路径 `%LOCALAPPDATA%\Android\Sdk\platform-tools\adb.exe`，设置页可覆盖；路径含空格时用绝对路径，避免 shell 拼接。

### 3. 过滤：Rust 端 AND 组合，即时生效

**选择**：解析每行为结构化 `LogEntry { timestamp, pid, tid, level, tag, message }`，在 Rust 对 ring buffer 全量或增量应用过滤；Tag/Message 为子串匹配（Message 忽略大小写），Level 为下限（>= 所选 level）。

**理由**：adb 原生 `-e` 仅支持 message 正则且改过滤需重启 stream；应用内过滤可组合 Tag + Message + Level，改条件即时生效。

**性能**：200k 行全量扫描约 3–10 ms，可接受。

### 4. Ring Buffer：按行数，默认 200,000

**选择**：Rust 端固定容量环形缓冲区，满则丢弃最旧行。预设：

| 预设 | 行数 | 约内存（数据） |
|------|------|----------------|
| Light | 50,000 | ~25 MB |
| Normal（默认） | 200,000 | ~100 MB |
| Heavy | 500,000 | ~250 MB |
| Marathon | 1,000,000 | ~500 MB |

单行 message 超 4 KB 时截断显示，完整内容可展开（可选 v1.1）。

**理由**：用户 session 可达数小时（~20 行/秒 × 3h ≈ 216k 行）；200k 覆盖约 2.8 小时。AS 默认 cycle buffer 仅 1024 KB（~5000 行），远不够用。

### 5. 前后端通信：Tauri Events 批量推送

**选择**：Rust 每 50–100 ms 将新匹配 log 批次通过 `app.emit("log-batch", entries)` 推送到前端；过滤条件变更时 emit `log-snapshot` 或前端请求 `get_filtered_logs` command。

**理由**：避免逐行 IPC；前端只持有当前过滤结果用于虚拟列表，不全量驻留 JS 堆。

**备选**：SharedArrayBuffer / 前端只拉可见窗口（复杂度高，MVP 不必）。

### 6. UI 布局：对齐 AS Logcat

```
Toolbar:  Device | Pause | Clear | Scroll to End | Export
Filters:  Tag | Message | Level
Body:     Virtualized log list (level colors)
Footer:   Live indicator | buffer count | lines/s | memory estimate
```

v1 浅色主题，无 dark mode。

### 7. 构建与交付

**选择**：本地 `tauri dev` / `tauri build`（macOS）；GitHub Actions 矩阵构建 `windows-latest` + `macos-latest`，产出 .msi/.exe 与 .dmg。

**理由**：Mac 无法交叉编译 Windows，CI 必需。

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| 高 log 量 UI 卡顿 | Rust 过滤 + 批量 event + react-virtuoso |
| Windows adb 路径/驱动问题 | 设置页 + 首次启动检测 + 文档说明 |
| ring buffer 满后丢失早期 log | 状态栏提示用量；Export；v2 disk spill |
| 多行 log / stack trace 解析 | 解析器处理 continuation 行；超长行截断 |
| 未签名 Windows exe SmartScreen 警告 | 文档说明；后续可选代码签名 |
| 功能与 Lazy Blacktea 等重复 | 聚焦「轻量 logcat only」，不做 ADB 全家桶 |

## Migration Plan

绿field 首次发布，无迁移。交付步骤：

1. `tauri build` 本地验证 macOS
2. GitHub Actions 打 Windows/macOS 包
3. 自用验证后推荐给同事（附带 adb 配置说明）

## Open Questions

- Export 默认格式：`.txt` 还是 `.log`？（建议 `.log`，threadtime 原文）
- Clear 是否同时执行 `adb logcat -c` 清设备 buffer？（建议：默认仅清 UI buffer，可选勾选清设备）
- 设置页入口：菜单栏 vs 齿轮图标？（建议工具栏齿轮）

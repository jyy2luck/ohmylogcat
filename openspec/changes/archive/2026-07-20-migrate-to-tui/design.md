## Context

ohmylogcat 刚完成 Tauri→egui 迁移，空闲内存优于 WebView，但 eframe 默认 wgpu 仍带来可观基线，满载偶发 300MB+，与「极致轻量」冲突。探索后确认产品真实定位更接近 **增强型 logcat 插件/阅读器**：保留缓冲、过滤、查找、导出，交互可平移为顶栏控件 + Tag/Message 输入，无需 GPU 窗口。

约束：macOS 开发、Windows 优先交付；复用现有 `engine`/buffer/filter/adb；键盘完整可用，鼠标可选增强。

## Goals / Non-Goals

**Goals:**

- 单进程 TUI（ratatui + crossterm），去掉 egui/eframe/wgpu
- 布局与心智对齐现有 GUI：顶栏功能区、过滤输入、日志视口、状态栏
- 行为对齐现有 specs（streaming / filtering / buffer / display / export / device / settings），终端约定处做等价改写
- 焦点模型：日志区默认吃快捷键；过滤/Find 输入时字符进字段
- 诚实内存叙事：UI 税接近可忽略；满载仍随 buffer 预设增长

**Non-Goals:**

- 本 change 不做 stdin pipe 模式（可列为后续增强）
- 不做完整 Soft-Wrap 变高虚拟列表打磨（MVP：默认截断；可选简单折行或二期）
- 不保留 egui 与 TUI 双壳并行
- 不捆绑 adb；不做 disk spill / 紧凑 LogEntry 存储（可另开 change）
- 不追求像素级模仿 Android Studio

## Decisions

### 1. UI 栈：ratatui + crossterm

**选择**：`ratatui` 负责布局与控件绘制；`crossterm` 负责 raw mode、键盘、可选鼠标、跨平台终端。

**理由**：Rust TUI 事实标准；与现有 tokio engine 同进程；无 GPU/字体图集。

**备选**：纯 crossterm 自绘（更轻但控件成本高）、保持 egui+glow（仍有 GPU 基线，不符合极致目标）。

### 2. 只保留 TUI，删除 egui 壳

**选择**：**BREAKING** 移除 `eframe`/`egui`/`rfd` 与 egui 版 `src/ui`；`egui-shell` capability 废弃，由 `tui-shell` 替代。

**理由**：双壳维护成本高；产品叙事已转向终端插件。

### 3. 布局与控件映射

```
┌─ toolbar: Device | Pause | Clear | Follow | Wrap? | Export | Settings ─┐
│ filters: Tag [........]  Message [..............]  Level [Info ▾]      │
├────────────────────────────────────────────────────────────────────────┤
│ log viewport (virtual window into filtered indices)                    │
├────────────────────────────────────────────────────────────────────────┤
│ status: count/cap  lines/s  ~MB   | find hint / editing focus hint     │
└────────────────────────────────────────────────────────────────────────┘
```

- 顶栏「按钮」：可见标签 + 快捷键字母；支持鼠标点击（crossterm mouse）
- Tag/Message：可聚焦输入框（自研光标 `String` 或 `tui-textarea`）
- Level：弹层或循环选择
- Settings / Device list / Export path：居中模态（Clear + Block）

### 4. 焦点与快捷键状态机

**选择**：显式 `Focus` 枚举：`Logs` | `Tag` | `Message` | `Level` | `Find` | `Modal(_)`。

- `Logs`：`Space` pause、`c` clear、`f` follow、`d` devices、`e` export、`s` settings、`/` find、`q` quit、滚动键
- 输入焦点：可打印字符写入字段；`Esc`/`Tab` 离开；全局快捷键暂停或仅保留 `Esc`
- Follow：与现有逻辑一致——贴底 on；上滚自动 off；工具栏再开并跳到底
- Find：`/` 或 Ctrl+F / Cmd+F（终端能收到时）；`n`/`N` 导航；高亮用 ANSI 反色/黄底

**理由**：避免「打 Tag 时误触发 Clear」；GUI 平移的关键。

### 5. 与 Engine 集成

**选择**：保留 `Arc<Engine>` + `mpsc` `EngineEvent`；TUI 主循环：`poll` 终端事件 → drain engine 事件 → 按需 request 下一帧 render。adb 仍在 tokio runtime。

**理由**：业务零 IPC；换壳成本最低。

**渲染**：每帧只格式化可见行 `scroll_offset .. offset+height`；不做整表物化。Soft-Wrap off 时可不维护 `row_heights`。

### 6. 导出与设置

**选择**：Export 打开模态，默认文件名 `ohmylogcat.log`，用户可编辑路径后确认；Settings 模态编辑 adb path 与 buffer preset，仍写现有 JSON。

**理由**：无 `rfd`；符合 TUI；跨平台一致。

### 7. Soft-Wrap MVP

**选择**：默认 **截断**（或水平平移列偏移）；工具栏可留 Wrap 开关——若实现成本高，MVP 可先做截断 + 状态栏提示，Wrap 标为后续。

**本设计采纳**：MVP 实现 **无折行 + 水平列偏移**；Soft-Wrap **偏好可保留字段但 UI 可暂禁用或简单按终端宽度折行（不做精确变高缓存）**。tasks 中拆成「截断必达 / wrap 尽力」。

### 8. 依赖瘦身

**选择**：`tokio` 从 `full` 收敛到实际需要的 features（`rt-multi-thread`, `process`, `io-util`, `sync`, `macros` 等）；去掉 egui 相关。

**理由**：与极致轻量一致。

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| Windows 终端键码/鼠标差异 | CI/手工在 Windows Terminal 验收；鼠标失败时键盘完整 |
| 中文 IME 在过滤框异常 | 早期在 macOS iTerm/WT 试；文档说明推荐终端 |
| Soft-Wrap 体验弱于 GUI | 诚实默认截断；二期再做 |
| 用户期望「独立窗口 App」 | README 明确 TUI；可提供简单 launcher 脚本（非必须） |
| 删除 egui 后无法快速回滚 UI | git 历史 / tag 保留 egui 基线 |

## Migration Plan

1. 引入 ratatui 骨架 + 空布局，链上现有 `Engine`（可先假数据）
2. 实现焦点状态机与顶栏/过滤/日志/状态栏
3. 移植 Pause/Clear/Follow/设备/导出/设置/Find
4. 删除 egui 依赖与旧 UI 模块；更新 README、Cargo.toml
5. macOS + Windows 终端验收清单打勾

回滚：发布前用 git tag 冻结最后 egui 可运行提交。

## Open Questions

- Soft-Wrap：本 change 是否必须提供可用开关，还是 MVP 仅截断？（建议：偏好字段保留，UI 先截断+水平滚动；wrap 标可选任务）
- 是否在本 change 加入 `--pipe` / stdin 模式？（建议：Non-Goal，另开）
- 默认 buffer 是否顺带改为 Light 以强化轻量叙事？（建议：本 change 不改默认 200k，另开 product change）

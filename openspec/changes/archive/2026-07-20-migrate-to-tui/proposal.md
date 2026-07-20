## Why

egui/eframe（默认 wgpu）带来的 GPU/字体基线仍偏高，满载偶发冲到数百 MB，与「极致轻量的增强型 logcat」目标冲突。改用终端 UI（TUI）可去掉图形栈税，把产品定位从「桌面 GUI App」转为「增强型 logcat 插件/阅读器」，同时尽量平移现有顶栏按钮与 Tag/Message 过滤交互。

## What Changes

- **BREAKING**：移除 egui / eframe 图形桌面壳；主入口改为终端 TUI 应用
- 以 **ratatui + crossterm**（或等价）实现主界面：顶栏功能控件、Tag/Message/Level 过滤输入、虚拟滚动日志区、状态栏、Find、Settings 模态
- 保留并复用现有 Rust 核心：`adb` / `parser` / `buffer` / `filter` / `engine` / settings 持久化
- 导出路径改为终端内路径输入（或合理默认文件名），不再依赖原生文件对话框（`rfd`）
- Soft-Wrap：MVP 可用截断 + 水平滚动或简化折行；完整变高折行可二期
- README / 内存叙事改为 TUI 空闲接近零 UI 税、满载随 buffer 增长；强调键盘优先、鼠标可选
- （可选同期或紧随）支持 stdin pipe 模式，强化「插件」用法——若本 change 不做，在 design 中列为非目标或后续

## Capabilities

### New Capabilities

- `tui-shell`: 终端主壳、布局分区、焦点/快捷键模型、鼠标可选点击、模态面板（替代 egui-shell）

### Modified Capabilities

- `egui-shell`: **移除或废弃**——需求迁到 `tui-shell`（本 change 用 delta 标记停用/替换）
- `log-display`: Find / Soft-Wrap / Scroll-to-End 的触发与呈现改为终端约定（快捷键、ANSI 高亮、顶栏控件），行为目标尽量等价
- `app-settings`: 设置入口改为 TUI 模态面板，不再依赖桌面窗口菜单式入口表述
- `log-export`: 导出选路从原生对话框改为路径输入/默认文件名（若现有 spec 绑定 dialog，则改需求）

## Impact

- **删除/替换**：`eframe`/`egui` 依赖与 `src/ui` 中 egui 绘制；`rfd` 可能移除；`src/main.rs` / `src/app.rs` 改为 TUI 事件循环
- **保留**：`engine`、ring buffer、过滤、adb 流、JSON settings（字段可延续）
- **新增依赖**：`ratatui`、`crossterm`（及可选 `tui-textarea` 类输入组件）
- **用户可见**：**BREAKING**——需在终端中运行；不再有独立 GPU 窗口；交互以键盘为主、鼠标为辅
- **平台**：继续 macOS 开发 / Windows 交付；需验收 Windows Terminal / ConPTY 键鼠与颜色

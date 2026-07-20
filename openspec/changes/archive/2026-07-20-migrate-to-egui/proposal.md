## Why

当前 Tauri + WKWebView 在 macOS 上空壳就有约 100MB+ 平台税，与「轻量独立 Logcat 查看器」的目标不符；业务逻辑本就在 Rust，UI 壳却占了大部分常驻内存。改用纯 Rust GUI（egui）可去掉 WebView，压低空闲占用，并简化为单进程架构，便于长期打磨。

## What Changes

- **BREAKING**：移除 Tauri v2、React/TypeScript 前端、Vite 与 npm 前端构建链
- 以 **egui + eframe** 重写主窗口 UI（工具栏、过滤栏、虚拟日志列表、状态栏、设置、Find、Soft-Wrap）
- 保留并内聚现有 Rust 核心：`adb` / `parser` / `buffer` / `filter` / `engine`（去掉 Tauri IPC，改为同进程直接调用）
- Ring buffer 改为按需增长至容量上限（避免启动即预分配 20 万空槽）
- 打包与 CI 改为纯 Rust 产物（macOS `.app` / Windows exe），不再依赖 `tauri build`
- README / 文档叙事改为纯 Rust GUI，内存目标改为诚实基线（空闲明显低于 WebView 方案；满载仍随 buffer 预设增长）

## Capabilities

### New Capabilities

- `egui-shell`: 单进程 egui/eframe 桌面壳、主窗口布局挂载、原生对话框与应用打包入口（替代 Tauri WebView 运行时）

### Modified Capabilities

- `log-display`: Find 不再依赖「抑制浏览器默认查找」；虚拟列表与交互行为在 egui 下保持等价需求
- `log-buffer`: 去掉「Rust backend」前后端表述；要求容量内按需分配，避免空载预填全部槽位

## Impact

- **删除**：`src/`（React）、`src-tauri` 的 Tauri 脚手架与 `tauri.conf.json` 依赖形态；`package.json` 前端依赖可移除或仅保留无关脚本
- **保留/迁移**：`src-tauri/src/{adb,parser,buffer,filter,engine,...}` 逻辑迁入新 crate 布局（或扁平为根 `src/`）
- **新增依赖**：`eframe` / `egui`、文件对话框（如 `rfd`）、设置持久化（如 JSON 配置文件）
- **CI**：GitHub Actions 从 `tauri build` 改为 `cargo build --release` + 平台打包步骤
- **用户可见**：**BREAKING** 安装包形态变化；功能应对齐现有 specs（设备、流、过滤、导出、设置、Find、Soft-Wrap）

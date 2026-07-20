## Context

ohmylogcat 当前是 Tauri v2 + React 应用：Rust 负责 adb 流、解析、ring buffer、过滤；React 负责 UI，经 Tauri IPC/events 通信。macOS 实测空闲即需承担 WKWebView 多进程税（WebContent ~100MB 级），与「轻量」目标冲突。业务能力（设备、流、过滤、buffer、导出、Find、Soft-Wrap）已有 specs 且行为应尽量保持。

约束：macOS 开发、Windows 优先交付；可慢慢打磨；优先压空闲内存与架构简洁。

## Goals / Non-Goals

**Goals:**

- 单进程纯 Rust 桌面应用（egui + eframe），去掉 WebView
- 功能对齐现有 specs（log-streaming / filtering / display / buffer / export / device-connection / app-settings）
- 复用现有 `adb` / `parser` / `buffer` / `filter` / engine 逻辑，最小化重写业务
- Ring buffer 按需分配，空载不预填全部 capacity 槽位
- CI 能产出 macOS / Windows release 二进制（可先简后繁）
- 诚实的内存叙事：空闲显著低于 Tauri；满载随 buffer 预设线性增长

**Non-Goals:**

- 像素级模仿 Android Studio / 原生平台控件观感
- 本次一并做 disk spill、复杂 query 语法、多设备 tab
- 保留 Tauri/React 双栈并行长期维护
- 捆绑 adb

## Decisions

### 1. UI 框架：egui + eframe

**选择**：`eframe` 承载窗口生命周期，`egui` 绘制主界面。

**理由**：即时模式适合高频日志刷新；与现有 Rust engine 同进程零 IPC；工具型 UI 匹配产品形态；MIT/Apache。

**备选**：iced（架构更「应用化」，长列表虚拟化成熟度一般）、Slint（观感更好但引入 DSL/许可证考量）。探索阶段已选定 egui。

### 2. Crate 布局：扁平单一 binary crate

**选择**：将逻辑从 `src-tauri/` 迁到仓库根级 Rust 工程（例如 `Cargo.toml` + `src/`），模块大致为：

```
src/
  main.rs          # eframe 入口
  app.rs           # OhmylogcatApp : eframe::App
  ui/              # toolbar, filters, log_list, status, settings, find
  engine.rs        # 现有 Engine（去 Tauri Emitter）
  adb/ parser/ buffer/ filter/
  settings.rs      # 持久化
```

**理由**：去掉 Tauri 嵌套后无需 `src-tauri`；单 crate 降低换栈摩擦。若日后需要 lib+bin 再拆。

**备选**：保留 `src-tauri` 目录名（易混淆，不采用）。

### 3. 并发模型：tokio 后台 + egui 主线程

**选择**：adb 读取与解析在 tokio 任务中进行；经 `std::sync::mpsc` 或 `tokio::sync::mpsc` + 主线程 drain，把批次交给 UI/engine；egui `update` 内不阻塞 IO。

**理由**：现有 engine 已用 tokio；eframe 主循环需保持响应。

**备选**：纯线程 + channel（可行，但已有 tokio 依赖可继续用）。

### 4. 去掉 IPC：Engine 由 App 直接持有

**选择**：`OhmylogcatApp` 持有 `Arc<Engine>`（或等价结构）；过滤/暂停/清空/导出均为直接方法调用。UI 显示层维护「当前可见切片」索引或短缓存，不再镜像整份 JS 数组。

**理由**：消除序列化与双份驻留；Display limit 可改为「只渲染可见行」，逻辑更贴近虚拟列表本质。

### 5. 虚拟列表：固定行高优先用 egui ScrollArea 按行绘制

**选择**：默认 Soft-Wrap off、固定行高时，用 `ScrollArea` + 可见行范围只构建可见 `LogEntry` 行；Soft-Wrap on 时用变高策略（`egui_virtual_list` 或自行缓存行高）。

**理由**：logcat 默认单行场景最常见；千万级 jump 用内置 ScrollArea 更稳。

### 6. Ring buffer：`Vec`/`VecDeque` 按需增长至 capacity

**选择**：替换「启动即 `push(None)` × capacity」为惰性增长；满容后覆盖最旧（ring 语义不变）。

**理由**：空载内存是换栈收益的关键部分；满载行为与 presets 不变。

### 7. 设置与对话框

**选择**：设置存用户配置目录下的 JSON（如 `dirs` + `serde_json`）；导出/选文件用 `rfd`。

**理由**：无 Tauri plugin；跨平台够用。

### 8. 打包与 CI

**选择**：阶段 1：`cargo build --release` 产出二进制，文档说明运行方式；阶段 2：再补 `.app` bundle / Windows 安装器（可用 `cargo-bundle` 或自写脚本）。GitHub Actions 矩阵 `macos-latest` + `windows-latest` 跑 release build。

**理由**：换栈先打通运行与功能；安装器可后置，避免阻塞主路径。

### 9. 前端与 Tauri 资产处置

**选择**：功能对齐并在 egui 下验证后，删除 `src/` React、`index.html`、Vite/Tauri 配置与相关 npm 依赖；README 全面改写。

**理由**：**BREAKING** 已声明；避免双栈残留。

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| egui 观感偏「工具」、不像 AS | 接受工作流对齐而非皮肤对齐；后续可调 visual density / 配色 |
| 中文 IME / 输入法边界问题 | 过滤框与 Find 早期在 macOS/Windows 实机验收 |
| Soft-Wrap + 变高虚拟列表复杂 | MVP 先保证 no-wrap 高性能；wrap 作为第二阶段打磨 |
| 迁移期功能回退（Find/快捷键） | tasks 按现有 specs 列验收清单，逐项打勾 |
| CI 安装包暂缺 | 阶段 1 交付 raw binary；文档标明 |
| 删除 Tauri 后无法快速回滚 UI | git 历史保留；必要时 branch 冻结旧栈 |

## Migration Plan

1. 新建根级 Cargo 工程骨架 + eframe 空窗，链上迁移后的 `Engine`（先假数据或空流）
2. 逐个移植 UI 面：toolbar → filters → virtual list → status → settings → export → find/wrap
3. Ring buffer 惰性分配改动 + 单测保留/更新
4. 端到端：连真机/模拟器验证流、过滤、暂停、清空、导出
5. 更新 CI 与 README；删除 Tauri/React 树
6. （可选后续）安装包美化、主题微调

回滚：在删除旧栈前提交可运行的 egui 基线；旧栈以 git tag/branch 保留直至 egui 达到功能平价。

## Open Questions

- 配置文件路径：是否沿用/兼容旧 Tauri 设置文件格式，还是干净切换新 JSON schema？（建议干净切换，文档说明）
- Windows 是否需要即时的 `.msi`，还是先 zip+exe？（建议先 exe）
- 暗色主题是否在换栈后顺手做？（仍属非目标，除非打磨阶段主动加）

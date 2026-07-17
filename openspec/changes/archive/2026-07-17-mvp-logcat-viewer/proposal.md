## Why

Android Studio 内置 Logcat 长时间调试时会占用大量 JVM 内存（索引引擎 + 对象开销 + 与 IDE 共享堆），几小时的 debug session 下体验明显变差。需要一个独立、轻量的 Logcat 查看器：交互习惯对齐 AS，内存可控，主要面向 Windows 开发者（开发者在 macOS 上构建）。

## What Changes

- 新建 Tauri v2 桌面应用 **ohmylogcat**，作为独立 Logcat 查看器
- 通过系统 `adb` 拉取 `logcat -v threadtime` 实时流
- Rust 端解析、过滤（Tag / Message / Level）、200k 行默认可配置 ring buffer
- React 前端虚拟滚动展示，交互对齐 AS Logcat 基础工具栏与过滤栏
- 支持 macOS 开发与 Windows 优先交付（GitHub Actions CI 构建）
- 设置页可配置 adb 路径与 buffer 预设

## Capabilities

### New Capabilities

- `device-connection`: 发现 adb 设备、选择当前设备、检测 adb 可用性与路径配置
- `log-streaming`: 启动/停止 logcat 流、解析 threadtime 格式、pause/resume/clear
- `log-filtering`: Tag 子串、Message 关键字（忽略大小写）、Level 下限过滤，AND 组合
- `log-display`: 虚拟滚动列表、按 level 着色、自动滚到底 / scroll to end
- `log-buffer`: 固定容量 ring buffer，预设 Light/Normal/Heavy/Marathon，状态栏显示用量
- `log-export`: 导出当前过滤结果或全部 buffer 为文本文件
- `app-settings`: adb 路径、buffer 大小等持久化配置

### Modified Capabilities

（无——绿field 项目，尚无既有 spec）

## Impact

- **新建代码库**：Tauri v2 + Rust backend + React/TypeScript frontend
- **外部依赖**：用户机器上的 Android SDK platform-tools（adb）；不捆绑 adb
- **CI**：GitHub Actions 构建 Windows (.msi/.exe) 与 macOS (.dmg)
- **非目标（v1）**：iOS、暗黑模式、多设备 tab、复杂 query 语法、stack trace 跳源码、disk spill

## Why

`ohmylogcat update`（以及 `install.sh` / `install.ps1` 首次安装）通过未认证的 `api.github.com/repos/.../releases/latest` 解析下载地址。GitHub 对该路径限流约 60 次/小时/IP，超额返回 **403**（不是 429）。本机已装 v0.3.0 时执行 update 会失败（`curl: (56) ... 403`），而 macOS 安装脚本自 v0.2.0 起未改——发版/反复测 update 耗尽配额后暴露了既有脆弱点。安装与更新热路径不应依赖易限流的 REST API。

## What Changes

- `install.sh` 与 `install.ps1` 的主下载路径改为 GitHub 的 `releases/latest/download/<asset>` 直链（302 → 资产 CDN），**不再依赖** `api.github.com` 才能完成安装。
- `ohmylogcat update` 成功后的「最新版本」校验改为不依赖未认证 GitHub REST API（例如跟随 `releases/latest` 的 Location 解析 tag），避免装成功却只能 warning。
- 若保留 API 作为可选 fallback：失败时给出可操作提示（限流/403），且主路径成功时不需要 API。
- 不改变：仍从 default branch 拉取安装脚本、Release 资产命名、安装目录、Windows 延迟替换行为。
- 非 BREAKING：旧二进制继续 `curl|sh` / `irm|iex` 拉最新脚本即可吃到修复。

## Capabilities

### New Capabilities

### Modified Capabilities

- `cli-lifecycle`: 要求平台安装脚本与 update 后的 latest-version 校验在热路径上不依赖未认证的 GitHub Releases API；安装/更新在 API 限流时仍须能完成二进制刷新。

## Impact

- `install.sh`、`install.ps1`：下载 URL 解析策略。
- `src/cli.rs`：`fetch_latest_release_version`（及可能的辅助函数）改为非 API 发现方式。
- `openspec/specs/cli-lifecycle/spec.md`：补充「无 API 安装/校验」要求与场景。
- README / 发版说明：可选补充限流与直链说明。
- 测试：URL 构造/解析单测；尽量用本地 override 或 mock，避免 CI 依赖实时 GitHub API。

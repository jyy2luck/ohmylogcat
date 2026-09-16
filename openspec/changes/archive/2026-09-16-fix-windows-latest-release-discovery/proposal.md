## Why

Windows 上终端界面右下角从不出现「有更新」，命令行 `update` 能装上新包却总会打出「could not verify latest release version」。根因是同一条最新发行标签探测：系统自带 PowerShell 5.1 对 `releases/latest` 使用「禁止跳转、从异常里读 Location」时抛出无响应对象的 `MaximumRedirectExceeded`，探测恒失败。macOS 走 curl 能拿到 tag，所以同版本在 Mac 上提示正常。GitHub 已有更新而本机仍停在旧包时，Windows 用户看不到信号。

## What Changes

- 修复 Windows 上对 `https://github.com/<repo>/releases/latest` 的标签解析，使在 GitHub 可达时能得到当前最新发行版本号（去前导 `v`）。
- 终端界面与命令行 `update` 装完后的版本核实继续共用这一探测；Windows 在网络正常时应能显示更新后缀，并报出「已是最新 / 已更新到某版本」，而不是误入「安装成功但版本未核实」。
- 探测仍不得走未认证的 GitHub 发布接口；超时、真正离线、无法解析为 tag 时，终端界面仍静默只显示当前版本，命令行核实失败仍不把整个 `update` 判失败。
- 不改安装脚本的 zip 直链下载；不在终端界面内执行更新；不改语义化版本三元组比较规则。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `tui-shell`：Windows 上当公开的 `releases/latest` 跳转可达时，最新发行探测必须成功解析出版本号，从而能按既有规则显示更新后缀；不得把 PowerShell「最大跳转次数超限」当成常规静默失败。
- `cli-lifecycle`：立即完成的 `update` 之后，Windows 在同样可达条件下必须能完成非 REST 的最新版本核实；`MaximumRedirectExceeded` 不得再把一次成功安装送进「未核实」警告。

## Impact

- `src/cli.rs`：`resolve_latest_release_location` 的 Windows 分支（Unix 的 curl 路径预期保持）。
- `src/app.rs`：继续调用共用探测；布局与比较逻辑不作为本次范围。
- 单元测试：覆盖 Windows 探测不再把「禁止跳转导致的无效操作异常」当成唯一路径；能从 tag 地址或等效最终地址抽出版本。
- 不新增 HTTP 依赖库。
- 安装脚本 `install.ps1` / `install.sh` 的资产下载路径不改。

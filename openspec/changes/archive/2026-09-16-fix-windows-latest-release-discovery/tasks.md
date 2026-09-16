## 1. Windows 最新标签探测

- [x] 1.1 将 `resolve_latest_release_location` 的 Windows 主路径改为调用 `curl.exe`：先 `-fsSI` 解析 Location，缺省再 `-fsSL -o NUL -w %{url_effective}`，保留 `User-Agent: ohmylogcat` 与 5 秒超时
- [x] 1.2 仅在 `curl.exe` 进程无法创建（程序不存在）时回退到 PowerShell：`Invoke-WebRequest -Method Head -UseBasicParsing` 跟随跳转并读取最终地址；禁止 `-MaximumRedirection 0`；curl 已存在但请求失败或超时时不再二次探测
- [x] 1.3 让 `location_header_value` 在 Windows 编译路径下也可用，Unix 行为保持不变；`fetch_latest_release_version` 与界面/命令行调用点不改签名

## 2. 测试

- [x] 2.1 补充回归：PowerShell 回退脚本不含 `-MaximumRedirection 0`；Location / 最终 tag 地址仍能抽出版本号
- [x] 2.2 运行 Rust 测试，确认 CLI 生命周期与界面探测相关测试通过

## 3. 手工确认（Windows）

- [x] 3.1 在本机用修好的二进制跑一次最新标签探测（或 `ohmylogcat update` 装完后的核实）：GitHub 可达时应得到版本号，不再出现 `MaximumRedirectExceeded` 的未核实警告

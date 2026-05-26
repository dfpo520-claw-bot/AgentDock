# AgentDock 桌面端手动调试手册

本手册用于 Windows 环境下的桌面端手动调试、安装包验证和发布前人工 smoke。

当前开发工作区：

```text
D:\workSpace\ohter\agentdock
```

## 1. 调试前检查

在 PowerShell 中进入项目目录：

```powershell
cd D:\workSpace\ohter\agentdock
git status --short
git log --oneline -5
```

手动 smoke 前建议确认：

- 工作区是干净的，除非你正在有意测试未提交改动。
- 最新 Phase 5 相关提交已经存在。
- Node、Cargo、Rust target、Tauri CLI 可用。

快速检查环境：

```powershell
node --version
cargo --version
rustup target list --installed
cargo tauri --version
```

## 2. 快速代码验证

启动桌面端之前，先跑基础验证：

```powershell
node --test tests\*.test.js
cargo check --manifest-path src-tauri\Cargo.toml
node node_modules\vite\bin\vite.js build
```

当前已知前端构建提示：

- `i18n` chunk 超过 500 kB。
- 这是一个真实的体积优化后续项，后面应通过语言包/模块懒加载解决。
- 本地 QA 阶段不把这个提示当作发布阻塞。

## 3. 桌面开发模式

需要快速调试前端和 Tauri 命令时，使用开发模式：

```powershell
npm run tauri dev
```

开发模式下重点检查：

- 应用窗口能打开，没有空白 WebView。
- 标题和主要产品身份显示为 AgentDock。
- 左侧导航可切换路由。
- Dashboard、Settings、Services、Assistant、Logs、About 页面能正常渲染。
- 检测到外部 Gateway 时，顶部提示条行为正确。

只调试 Web 页面时，可用浏览器模式：

```powershell
npm run dev
```

然后打开：

```text
http://127.0.0.1:1420
```

## 4. 构建和打包

先构建前端：

```powershell
node node_modules\vite\bin\vite.js build
```

构建 Windows NSIS 安装包：

```powershell
cargo tauri build --target x86_64-pc-windows-msvc --bundles nsis
```

预期安装包位置：

```text
src-tauri\target\x86_64-pc-windows-msvc\release\bundle\nsis\AgentDock_0.15.3_x64-setup.exe
```

## 5. 发布产物 smoke

验证 release bundle 结构：

```powershell
node scripts\verify-release-smoke.mjs --bundle-dir src-tauri\target\x86_64-pc-windows-msvc\release\bundle --platform windows
```

本地 QA 阶段验证 Windows 签名状态：

```powershell
node scripts\verify-windows-signing.mjs --file src-tauri\target\x86_64-pc-windows-msvc\release\bundle\nsis\AgentDock_0.15.3_x64-setup.exe --allow-unsigned
```

正式证书尚未接入时，预期状态：

- 签名状态可能是 `NotSigned`。
- `Publishable` 必须是 `no`。
- 这只允许用于本地 QA smoke，不能发布。

发布候选包必须去掉 `--allow-unsigned`，并且 Authenticode 验证结果必须是 `Valid`。

## 6. 安装态人工 smoke

运行安装包：

```powershell
src-tauri\target\x86_64-pc-windows-msvc\release\bundle\nsis\AgentDock_0.15.3_x64-setup.exe
```

安装完成后，从以下入口启动 AgentDock：

- 开始菜单快捷方式。
- 桌面快捷方式，如果安装时创建了。
- 安装目录里的 `AgentDock.exe`。

记录这些检查项：

| 区域 | 检查内容 | 预期结果 |
| --- | --- | --- |
| 启动 | 从安装入口打开应用 | 不崩溃，无空白 WebView |
| Dashboard | 主页面渲染 | 状态卡片和页面主体可见 |
| Settings | 读取配置 | 使用产品自有配置路径 |
| Settings | 写入配置 | 保存成功，重启后仍生效 |
| Services | 检测 OpenClaw | 已安装/缺失/外部实例状态清晰 |
| Services | Gateway restart | 执行破坏性操作前出现确认 |
| Services | 安装/升级/卸载 | 流程有响应，错误可读 |
| Assistant | 只读模式 | 写入/命令类工具被阻止 |
| Assistant | 规划模式 | 危险操作仍需要确认 |
| Assistant | 无限模式 | 命令/写入/网络工具仍需要确认 |
| Logs | 查看日志 | 敏感信息已脱敏 |
| Logs | 导出日志 | token、密码、API key 已脱敏 |
| Updates | 检查更新 manifest | 使用 AgentDock 更新清单结构 |
| Uninstall | 卸载应用 | 应用文件移除；用户数据按策略保留 |

## 7. 路由级 UI smoke

如果只需要验证核心路由渲染，先启动构建产物静态服务：

```powershell
node scripts\serve.js --host 127.0.0.1 --port 1421
```

另开一个 PowerShell，读取本机面板密码并传给 smoke 脚本。不要打印或提交密码：

```powershell
$env:AGENTDOCK_SMOKE_PASSWORD = (Get-Content "$HOME\.openclaw\agentdock.json" -Raw | ConvertFrom-Json).accessPassword
node scripts\smoke-ui-routes.mjs --base-url http://127.0.0.1:1421 --password $env:AGENTDOCK_SMOKE_PASSWORD --out-dir docs\release\ui-smoke-2026-05-15
```

预期覆盖路由：

- `/dashboard`
- `/settings`
- `/services`
- `/assistant`
- `/logs`
- `/about`

预期输出：

- 所有路由通过。
- `summary.json` 被刷新。
- 截图写入 `docs\release\ui-smoke-2026-05-15`。

如果 smoke 报密码错误，确认 `$HOME\.openclaw\agentdock.json` 中的 `accessPassword` 是当前密码，然后重新执行。不要把密码写入文档或日志。

## 8. 日志和本地状态

常见本地状态目录：

```text
%USERPROFILE%\.openclaw
%USERPROFILE%\.agentdock
```

常见检查内容：

- `agentdock.json` 或产品自有面板配置。
- OpenClaw 运行时配置。
- Gateway 日志。
- Guardian 日志。
- 备份日志。
- Assistant 审计输出。

写入发布记录时需要脱敏：

- API key。
- 密码。
- bearer token。
- 包含敏感账号名的私有路径。

## 9. 常见问题定位

空白窗口：

1. 运行 `node node_modules\vite\bin\vite.js build`。
2. 在 `npm run tauri dev` 下查看 DevTools console。
3. 确认 `dist\assets` 下有生成资源。
4. 确认路由 hash 合法，例如 `#/dashboard`。

Gateway 是外部实例或不可管理：

1. 打开 Services。
2. 查看顶部 banner 是否提示外部 Gateway。
3. 只有在确认端口归属后再执行认领流程。
4. 在 smoke 记录中写下 PID 和端口。

安装或升级命令失败：

1. 复制界面上的完整错误信息。
2. 检查 Settings 中的代理设置。
3. 检查 Services 中的 Node 和 Git 检测结果。
4. 重试前先查看最近日志。

Hermes dashboard 无法打开：

1. 确认服务正在运行。
2. 检查端口 `9119`。
3. 查看 Hermes 日志导出。
4. 如果依赖缺失，优先使用界面给出的安装引导，不要手动做半截安装。

签名无效：

1. 确认 `signtool.exe` 存在。
2. 确认 `Cert:\CurrentUser\My` 或 `Cert:\LocalMachine\My` 中存在带私钥的 Code Signing 证书。
3. 设置 `WINDOWS_CODESIGN_CERT_THUMBPRINT`。
4. 运行 `npm run release:sign:windows -- --file <installer>`。
5. 不带 `--allow-unsigned` 重新运行签名验证。

## 10. 人工 smoke 记录模板

正式执行时，可复制这段到按日期命名的发布记录中：

````markdown
# Desktop Manual Smoke - YYYY-MM-DD

Artifact:

```text
src-tauri\target\x86_64-pc-windows-msvc\release\bundle\nsis\AgentDock_0.15.3_x64-setup.exe
```

Environment:

- Windows version:
- Install path:
- AgentDock version:
- OpenClaw state:
- Gateway state:
- Signing status:

Results:

| Check | Result | Notes |
| --- | --- | --- |
| Install |  |  |
| Launch |  |  |
| Dashboard |  |  |
| Settings read/write |  |  |
| Gateway restart |  |  |
| OpenClaw install/upgrade/uninstall |  |  |
| Assistant confirmation policy |  |  |
| Logs redaction |  |  |
| Update check |  |  |
| Uninstall |  |  |

Decision:

- Local QA only / Publish candidate / Blocked

Follow-ups:

- 
````

## 11. 清理

手动 smoke 结束后：

```powershell
git status --short
Get-Process agentdock -ErrorAction SilentlyContinue
Get-Process node -ErrorAction SilentlyContinue
```

只停止本次测试中启动的进程，不要结束无关的用户进程。

如果安装了本地 QA 版本，请通过 Windows Apps 或 NSIS 卸载器卸载，并确认用户数据保留行为符合测试计划。

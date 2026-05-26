# AgentDock Desktop Manual Debugging Handbook

This handbook is for manual desktop debugging and installed-app smoke testing on Windows.

Workspace used by the current development session:

```text
D:\workSpace\ohter\agentdock
```

## 1. Preflight

Open PowerShell in the workspace:

```powershell
cd D:\workSpace\ohter\agentdock
git status --short
git log --oneline -5
```

Expected before manual smoke:

- Working tree is clean, unless you are intentionally testing uncommitted changes.
- Latest relevant Phase 5 commit is present.
- Node, Cargo, Rust target, and Tauri CLI are available.

Quick environment checks:

```powershell
node --version
cargo --version
rustup target list --installed
cargo tauri --version
```

## 2. Fast Code Verification

Run these before launching the desktop app:

```powershell
node --test tests\*.test.js
cargo check --manifest-path src-tauri\Cargo.toml
node node_modules\vite\bin\vite.js build
```

Current known frontend build warning:

- `i18n` chunk is larger than 500 kB.
- This is a real bundle-size follow-up and should later be fixed with locale/module lazy loading.
- Do not treat this warning as a release blocker for local QA.

## 3. Desktop Dev Mode

Use dev mode when you need fast frontend/backend iteration:

```powershell
npm run tauri dev
```

Manual checks in dev mode:

- App window opens without a blank WebView.
- Title and visible brand show AgentDock where product-owned identity is expected.
- Sidebar route navigation works.
- Dashboard, Settings, Services, Assistant, Logs, and About routes render.
- Top gateway banner behaves correctly when an external Gateway is detected.

Useful browser-only fallback:

```powershell
npm run dev
```

Then open:

```text
http://127.0.0.1:1420
```

## 4. Build And Package

Frontend build:

```powershell
node node_modules\vite\bin\vite.js build
```

Windows NSIS package build, using the Windows MSVC target:

```powershell
cargo tauri build --target x86_64-pc-windows-msvc --bundles nsis
```

Expected installer location:

```text
src-tauri\target\x86_64-pc-windows-msvc\release\bundle\nsis\AgentDock_0.15.3_x64-setup.exe
```

## 5. Release Artifact Smoke

Verify the release bundle structure:

```powershell
node scripts\verify-release-smoke.mjs --bundle-dir src-tauri\target\x86_64-pc-windows-msvc\release\bundle --platform windows
```

Verify Windows signing status for local QA:

```powershell
node scripts\verify-windows-signing.mjs --file src-tauri\target\x86_64-pc-windows-msvc\release\bundle\nsis\AgentDock_0.15.3_x64-setup.exe --allow-unsigned
```

Expected while formal certificate hookup is deferred:

- Signing status may be `NotSigned`.
- `Publishable` must be `no`.
- This is acceptable only for local QA smoke.

Publish candidates must run the same command without `--allow-unsigned` and must return Authenticode `Valid`.

## 6. Installed-App Manual Smoke

Run the installer:

```powershell
src-tauri\target\x86_64-pc-windows-msvc\release\bundle\nsis\AgentDock_0.15.3_x64-setup.exe
```

After installation, launch AgentDock from:

- Start menu shortcut.
- Desktop shortcut, if created.
- Installed `AgentDock.exe`.

Record these results:

| Area | What To Check | Expected |
| --- | --- | --- |
| Launch | App opens from installed entry | No crash, no blank WebView |
| Dashboard | Main route renders | Status cards and route body visible |
| Settings | Read config | Product-owned config paths are used |
| Settings | Write config | Save succeeds and survives restart |
| Services | Detect OpenClaw | Installed/missing/foreign states are understandable |
| Services | Gateway restart | User confirmation appears before disruptive action |
| Services | Install/upgrade/uninstall | Flow responds and errors are readable |
| Assistant | Read-only mode | Mutating tools are blocked |
| Assistant | Plan mode | Dangerous actions still require confirmation |
| Assistant | Unlimited mode | Command/write/network tools still require confirmation |
| Logs | View logs | Secrets are redacted |
| Logs | Export logs | Exported content redacts tokens, passwords, API keys |
| Updates | Check manifest | Uses AgentDock update manifest shape |
| Uninstall | Remove app | App files removed; user data preserved unless explicitly selected |

## 7. Route-Level UI Smoke

If you only need route rendering evidence, serve the built `dist` output:

```powershell
node scripts\serve.js --host 127.0.0.1 --port 1421
```

In another PowerShell window, pass the local panel password without printing it:

```powershell
$env:AGENTDOCK_SMOKE_PASSWORD = (Get-Content "$HOME\.openclaw\agentdock.json" -Raw | ConvertFrom-Json).accessPassword
node scripts\smoke-ui-routes.mjs --base-url http://127.0.0.1:1421 --password $env:AGENTDOCK_SMOKE_PASSWORD --out-dir docs\release\ui-smoke-2026-05-15
```

Expected routes:

- `/dashboard`
- `/settings`
- `/services`
- `/assistant`
- `/logs`
- `/about`

Expected output:

- All routes pass.
- `summary.json` is refreshed.
- Screenshots are written under `docs\release\ui-smoke-2026-05-15`.

If the smoke fails with a password error, confirm `$HOME\.openclaw\agentdock.json` contains the current `accessPassword` and rerun without logging the password.

## 8. Logs And Local State

Common local state locations:

```text
%USERPROFILE%\.openclaw
%USERPROFILE%\.agentdock
```

Common things to inspect:

- `agentdock.json` or product-owned panel config.
- OpenClaw runtime config.
- Gateway logs.
- Guardian logs.
- Backup logs.
- Assistant audit output.

When recording logs in release notes:

- Mask API keys.
- Mask passwords.
- Mask bearer tokens.
- Mask private paths if they contain sensitive account names.

## 9. Common Debugging Patterns

Blank window:

1. Run `node node_modules\vite\bin\vite.js build`.
2. Check DevTools console in `npm run tauri dev`.
3. Confirm generated assets exist under `dist\assets`.
4. Confirm route hash is valid, for example `#/dashboard`.

Gateway is foreign or unmanaged:

1. Open Services.
2. Check whether the banner reports an external Gateway.
3. Use claim flow only when you understand which process owns the port.
4. Record PID and port in smoke notes.

Install or upgrade command fails:

1. Copy the visible error message.
2. Check proxy settings in Settings.
3. Check Node and Git detection in Services.
4. Inspect recent logs before retrying.

Hermes dashboard does not open:

1. Confirm the service is running.
2. Check port `9119`.
3. Inspect Hermes log export.
4. If dependencies are missing, use the displayed install guidance rather than manual partial installs.

Signing is not valid:

1. Confirm `signtool.exe` exists.
2. Confirm a Code Signing certificate with private key exists in `Cert:\CurrentUser\My` or `Cert:\LocalMachine\My`.
3. Set `WINDOWS_CODESIGN_CERT_THUMBPRINT`.
4. Run `npm run release:sign:windows -- --file <installer>`.
5. Run signing verification without `--allow-unsigned`.

## 10. Manual Smoke Record Template

Copy this block into a dated release note when doing a real pass:

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

## 11. Cleanup

After manual smoke:

```powershell
git status --short
Get-Process agentdock -ErrorAction SilentlyContinue
Get-Process node -ErrorAction SilentlyContinue
```

Stop only test processes that you started for this pass. Do not kill unrelated user processes.

If you installed a local QA build, uninstall it through Windows Apps or the generated NSIS uninstaller and confirm user data preservation behavior matches the test plan.

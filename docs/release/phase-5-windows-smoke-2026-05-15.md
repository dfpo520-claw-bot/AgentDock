# Phase 5 Windows Installer Smoke - 2026-05-15

Artifact under test:

```text
src-tauri\target\x86_64-pc-windows-msvc\release\bundle\nsis\AgentDock_0.15.3_x64-setup.exe
```

## Automated Evidence

| Check | Result | Evidence |
| --- | --- | --- |
| Authenticode status | Local smoke only | `Windows signing status: NotSigned`, `Publishable: no` |
| Signing execution readiness | Blocked | `signtool.exe` exists, but no Code Signing certificate was found in `CurrentUser/My` or `LocalMachine/My` |
| NSIS silent install | Pass | Installer exited with `ExitCode=0` |
| Installed metadata | Pass | Registry showed `AgentDock`, version `0.15.3`, publisher `AgentDock Team`, install location `D:\AgentDock` |
| Installed executable | Pass | `D:\AgentDock\AgentDock.exe`, `24521728` bytes |
| Launch installed app | Pass | Started installed process `agentdock`, observed running after 8 seconds |
| Stop launched app | Pass | Smoke process was stopped after launch verification |
| NSIS silent uninstall | Pass | Uninstaller exited with `ExitCode=0` |
| Executable removal | Pass | `D:\AgentDock\AgentDock.exe` no longer exists after uninstall |
| Uninstall registry removal | Pass | No remaining `AgentDock` uninstall entry was returned |

## Manual UI Checks Still Required

These checks require an interactive installed-app pass and are not covered by the automated smoke above:

- Dashboard renders with expected data and no blank WebView.
- Settings read/write uses product-owned config paths.
- OpenClaw install detection, upgrade, uninstall, Node detection, Git detection, proxy diagnostics, and gateway restart flows respond from the installed app.
- Assistant read-only, plan, and unlimited modes enforce dangerous tool confirmation policy.
- Logs, diagnostics, assistant command audit output, and Hermes log export redact secrets in visible UI/download surfaces.
- Update check reads the product update manifest shape and failed update handling follows rollback policy.
- Uninstall UX preserves user data unless explicit removal is selected.

## Release Decision

This artifact is suitable for local QA smoke only. It is not publishable until Windows signing verification returns `Valid` without `--allow-unsigned`.

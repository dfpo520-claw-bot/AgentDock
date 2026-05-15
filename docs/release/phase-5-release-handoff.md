# Phase 5 Release Handoff

This handoff records the remaining release work in the order currently selected for AgentDock production builds.

## 1. Signing Closure

Windows installers are not publishable until Authenticode verification returns `Valid`.

Local smoke may inspect an unsigned installer without treating it as publishable:

```powershell
npm run release:signing:windows -- --file src-tauri\target\x86_64-pc-windows-msvc\release\bundle\nsis\AgentDock_0.15.3_x64-setup.exe --allow-unsigned
```

Before publishing, remove `--allow-unsigned` and require a clean result:

```powershell
npm run release:signing:windows -- --file src-tauri\target\x86_64-pc-windows-msvc\release\bundle\nsis\AgentDock_0.15.3_x64-setup.exe
```

Required signing inputs remain secret-only environment or CI values:

- Windows: `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.
- macOS: `APPLE_SIGNING_IDENTITY`, `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_TEAM_ID`.
- Linux: `LINUX_SIGNING_KEY` only when detached signatures are required.

The release manifest may record whether signing inputs are present, but must never include certificate contents, private keys, passwords, or tokens.

## 2. Manual Installer Smoke

Use the generated Windows installer artifact for the current manual pass:

```text
src-tauri\target\x86_64-pc-windows-msvc\release\bundle\nsis\AgentDock_0.15.3_x64-setup.exe
```

The automated install/start/uninstall evidence for the current artifact is recorded in `docs/release/phase-5-windows-smoke-2026-05-15.md`.

Record pass/fail notes for each item before treating the installer as production-ready:

- Install succeeds from the NSIS setup executable.
- AgentDock launches from the installed shortcut or application entry.
- Dashboard loads without a blank window or startup panic.
- Settings can read and write product-owned config paths.
- OpenClaw install detection, upgrade, uninstall, Node detection, Git detection, proxy diagnostics, and gateway restart flows still respond.
- Assistant read-only, plan, and unlimited modes still enforce dangerous tool confirmation policy.
- Logs, diagnostics, assistant command audit output, and Hermes log export redact secrets.
- Update check reads the product update manifest shape and failed update handling follows rollback policy.
- Uninstall removes the application while preserving user data unless the installer explicitly asks for removal.

## 3. Automation Deferred

CI/release automation is intentionally paused for this pass. Do not expand workflow files until signing closure, manual smoke, and environment cleanup are finished.

When resumed, the automation scope should be limited to:

- Build the desktop package.
- Generate `release-manifest.json` and `checksums.sha256`.
- Run `release:smoke`.
- Run platform signing verification.
- Upload release artifacts and manifest outputs.

## 4. Environment Cleanup

The local PowerShell profile warning has been cleaned for the current Windows build host. The old one-line `fnm` profile was moved to:

```text
C:\Users\14772\Documents\WindowsPowerShell\Microsoft.PowerShell_profile.ps1.disabled-fnm-20260515.bak
```

New PowerShell commands no longer emit the startup warning during release verification.

## Later Queue

Defer these until signing, manual smoke, automation decision, and environment cleanup are settled:

- Vite dynamic/static import and chunk-size warning cleanup.
- Rust unused helper warning cleanup.
- Brand asset final replacement.
- Formal upstream fork and license strategy.
- Deeper backend command migration beyond current fork compatibility.

# Phase 5 UI Smoke - 2026-05-15

Target:

```text
http://127.0.0.1:1421
```

Command:

```powershell
$env:AGENTDOCK_SMOKE_PASSWORD = (Get-Content "$HOME\.openclaw\clawpanel.json" -Raw | ConvertFrom-Json).accessPassword
node scripts\smoke-ui-routes.mjs --base-url http://127.0.0.1:1421 --password $env:AGENTDOCK_SMOKE_PASSWORD --out-dir docs\release\ui-smoke-2026-05-15
```

## Results

| Route | Result | Text Length | Screenshot |
| --- | --- | ---: | --- |
| `/dashboard` | Pass | 1246 | `docs/release/ui-smoke-2026-05-15/dashboard.png` |
| `/settings` | Pass | 1337 | `docs/release/ui-smoke-2026-05-15/settings.png` |
| `/services` | Pass | 1187 | `docs/release/ui-smoke-2026-05-15/services.png` |
| `/assistant` | Pass | 695 | `docs/release/ui-smoke-2026-05-15/assistant.png` |
| `/logs` | Pass | 3716 | `docs/release/ui-smoke-2026-05-15/logs.png` |
| `/about` | Pass | 1653 | `docs/release/ui-smoke-2026-05-15/about.png` |

Summary output:

```text
UI smoke passed for 6 routes
PASS /dashboard text=1246
PASS /settings text=1337
PASS /services text=1187
PASS /assistant text=695
PASS /logs text=3716
PASS /about text=1653
```

Note: local profiles that have already changed the default password must pass the configured password through `AGENTDOCK_SMOKE_PASSWORD`; the password value must not be committed or printed in release notes.

Refresh note:

- Screenshots were refreshed after the Modern Ops UI/brand refactor on 2026-05-16.
- The smoke helper fails on unexpected modal overlays, and only dismisses the known Gateway ownership guidance overlay before screenshot capture so route content is visible.
- The default-password security banner remains visible and reserves app-shell viewport space so it does not overlap route headers or clip the bottom of the shell.
- In this local smoke profile, `/dashboard` redirects into the first-run OpenClaw setup surface because the OpenClaw CLI is not installed; the screenshot still covers the authenticated AgentDock shell, sidebar branding, release links, and setup route layout.
- A historical `[ClawPanel]` prefix may appear inside local runtime log content on the Logs route; this is local log data, not shipped UI copy.

## Scope

Covered:

- Login flow through the real login form.
- Dashboard, settings, services, assistant, logs, and about route rendering.
- Splash and login overlay removal after authentication.
- AgentDock sidebar branding and DeepAi assistant naming in the refreshed UI.
- Default-password security banner visibility without covering page headers or clipping shell content.
- Screenshot capture for each route.
- Installed desktop application launch smoke: `D:\AgentDock\AgentDock.exe` started and stayed alive after launch.

Not clicked during automation:

- Gateway restart and ownership-changing actions.
- Update/upgrade/uninstall actions inside OpenClaw service management.
- Assistant tool execution that would run shell/network/write operations.
- Settings save against production user config.

Those actions remain manual QA items because they can mutate the local machine, service state, or user configuration.

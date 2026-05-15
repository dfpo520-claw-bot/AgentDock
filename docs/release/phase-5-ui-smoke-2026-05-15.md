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
| `/dashboard` | Pass | 1663 | `docs/release/ui-smoke-2026-05-15/dashboard.png` |
| `/settings` | Pass | 1776 | `docs/release/ui-smoke-2026-05-15/settings.png` |
| `/services` | Pass | 1626 | `docs/release/ui-smoke-2026-05-15/services.png` |
| `/assistant` | Pass | 1123 | `docs/release/ui-smoke-2026-05-15/assistant.png` |
| `/logs` | Pass | 2235 | `docs/release/ui-smoke-2026-05-15/logs.png` |
| `/about` | Pass | 2076 | `docs/release/ui-smoke-2026-05-15/about.png` |

Summary output:

```text
UI smoke passed for 6 routes
PASS /dashboard text=1663
PASS /settings text=1776
PASS /services text=1626
PASS /assistant text=1123
PASS /logs text=2235
PASS /about text=2076
```

Note: local profiles that have already changed the default password must pass the configured password through `AGENTDOCK_SMOKE_PASSWORD`; the password value must not be committed or printed in release notes.

## Scope

Covered:

- Login flow through the real login form.
- Dashboard, settings, services, assistant, logs, and about route rendering.
- Splash and login overlay removal after authentication.
- Screenshot capture for each route.
- Installed desktop application launch smoke: `D:\AgentDock\AgentDock.exe` started and stayed alive after launch.

Not clicked during automation:

- Gateway restart and ownership-changing actions.
- Update/upgrade/uninstall actions inside OpenClaw service management.
- Assistant tool execution that would run shell/network/write operations.
- Settings save against production user config.

Those actions remain manual QA items because they can mutate the local machine, service state, or user configuration.

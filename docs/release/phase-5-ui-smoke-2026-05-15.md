# Phase 5 UI Smoke - 2026-05-15

Target:

```text
http://127.0.0.1:1421
```

Command:

```powershell
node scripts\smoke-ui-routes.mjs --base-url http://127.0.0.1:1421 --password 123456 --out-dir docs\release\ui-smoke-2026-05-15
```

## Results

| Route | Result | Text Length | Screenshot |
| --- | --- | ---: | --- |
| `/dashboard` | Pass | 695 | `docs/release/ui-smoke-2026-05-15/dashboard.png` |
| `/settings` | Pass | 2256 | `docs/release/ui-smoke-2026-05-15/settings.png` |
| `/services` | Pass | 1995 | `docs/release/ui-smoke-2026-05-15/services.png` |
| `/assistant` | Pass | 1417 | `docs/release/ui-smoke-2026-05-15/assistant.png` |
| `/logs` | Pass | 2529 | `docs/release/ui-smoke-2026-05-15/logs.png` |
| `/about` | Pass | 2379 | `docs/release/ui-smoke-2026-05-15/about.png` |

Summary output:

```text
UI smoke passed for 6 routes
PASS /dashboard text=695
PASS /settings text=2256
PASS /services text=1995
PASS /assistant text=1417
PASS /logs text=2529
PASS /about text=2379
```

## Scope

Covered:

- Default-password login flow through the real login form.
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

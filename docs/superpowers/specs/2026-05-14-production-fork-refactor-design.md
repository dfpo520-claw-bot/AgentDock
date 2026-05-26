# Production Fork Refactor Design

## Goal

Build a real production desktop application by first forking the current AgentDock codebase, keeping the existing Tauri v2 + Vite + Rust command architecture, and then gradually replacing branding, configuration ownership, module boundaries, and backend command implementations with our own product architecture.

This is not a learning clone. The first milestone must produce a runnable and packageable production baseline. Refactoring happens after the baseline is stable.

## Current Baseline

The existing project is a multi-engine AI Agent management panel.

- Frontend: Vite, vanilla JavaScript modules, global CSS, hash router.
- Desktop shell: Tauri v2.
- Backend capability layer: Rust Tauri commands under `src-tauri/src/commands`.
- Runtime integrations: OpenClaw, Hermes Agent, Node.js, Python, Git, Docker, local services, local files, logs, config files.
- Shared app shell: login overlay, sidebar, theme, i18n, engine manager, update banner, gateway status banner, tray behavior.

The current architecture already supports production concerns such as desktop packaging, Web deployment mode, local service management, update checks, logs, backups, diagnostics, and engine-specific pages. The fastest path is to preserve these while replacing identity and ownership layer by layer.

## Product Direction

The new product will initially remain functionally close to the fork so it can be shipped quickly. Its visible identity, distribution metadata, default configuration, documentation, and update channels will be replaced first.

After the first production baseline, OpenClaw and Hermes-specific assumptions will be isolated behind explicit engine and runtime interfaces. This allows the product to keep current functionality while making room for our own modules, naming, data directories, command implementations, and release flow.

## Architecture

The production app keeps the existing shape:

```text
Tauri desktop shell
  -> Vite frontend app
  -> Rust Tauri command layer
  -> local runtime adapters
  -> external CLIs, services, config files, logs, Docker, Python/Node tools
```

Rust remains the backend layer. A Java backend is no longer part of this plan. This avoids duplicating Tauri IPC behavior, sidecar supervision, filesystem access, tray integration, and process control that the existing project already handles.

The refactor introduces clearer ownership boundaries:

- App shell: boot, auth, route, theme, sidebar, updates, tray, global banners.
- Product identity: names, icons, links, package IDs, default URLs, release channels.
- Engine management: active engine, setup routes, readiness, health, feature gates.
- Runtime services: service status, start/stop/restart, guardian, diagnostics.
- Configuration: panel config, engine config, model config, gateway config, update config.
- Operational data: logs, backups, memory, sessions, usage, files.
- Integrations: messaging channels, plugins, Docker, cftunnel, ClawApp, AI assistant tools.

## Phase 1: Production Baseline

Phase 1 keeps functionality stable and changes only low-risk product identity and build metadata.

Scope:

- Verify local development and packaging commands.
- Establish a new production branch.
- Replace app name, package metadata, Tauri identifier, window title, icon set, logo, splash/brand assets, README, About page links, homepage, repository references, and default external links.
- Replace generated temporary brand assets with our own temporary product identity.
- Keep OpenClaw and Hermes functionality intact.
- Keep existing command implementations intact.
- Preserve Web mode unless it blocks desktop packaging.

Success criteria:

- `npm run build` succeeds.
- `cargo check --manifest-path src-tauri/Cargo.toml` succeeds.
- Desktop dev launch succeeds.
- A Windows package can be produced or the remaining packaging blockers are documented with exact commands and errors.
- No visible old brand remains in the first-run shell, sidebar, About page, app metadata, or public docs intended for users.

## Phase 2: Configuration Ownership

Phase 2 moves the app from fork identity to product-owned configuration.

Scope:

- Introduce product-level constants for app name, homepage, support links, update endpoints, default directories, config filenames, and release channels.
- Centralize identity references instead of scattering literals across frontend and Rust.
- Add compatibility aliases for old config paths so existing local data can still be read when needed.
- Separate panel config from engine config.
- Document config migration rules.

Success criteria:

- New installs use product-owned directories and config names.
- Existing fork data can be detected and either imported or ignored with a clear UI path.
- Brand and path changes are driven by centralized constants.

## Phase 3: Module Boundary Refactor

Phase 3 reshapes the codebase while keeping behavior equivalent.

Frontend target modules:

- `app-shell`: boot, auth, layout, route, theme, update banners.
- `engines`: engine registry, OpenClaw adapter, Hermes adapter, future custom adapters.
- `runtime`: API client, service status, logs, diagnostics, process operations.
- `features`: models, agents, chat, memory, cron, channels, skills, plugins, assistant.
- `shared-ui`: modal, toast, sidebar, badges, forms, state helpers.

Rust target modules:

- `commands/app_config.rs`
- `commands/runtime_services.rs`
- `commands/engine_openclaw.rs`
- `commands/engine_hermes.rs`
- `commands/integrations.rs`
- `commands/diagnostics.rs`
- `commands/storage.rs`

The existing files do not need to be renamed all at once. Refactoring should happen module by module, with tests after each moved boundary.

Success criteria:

- Existing routes still load.
- Existing command names remain compatible for the frontend until their replacements are complete.
- New module names make ownership clear.
- No large behavioral rewrite is mixed into boundary-only changes.

## Phase 4: Backend Command Replacement

Phase 4 replaces inherited Rust command implementations with product-owned implementations.

Replacement order:

1. Low-risk local commands: About/version info, panel config, static metadata, theme settings.
2. Storage commands: logs, backups, memory files, image storage, local file browsing.
3. Runtime commands: service status, start/stop/restart, gateway claim, diagnostics.
4. Engine commands: OpenClaw adapter, Hermes adapter, model testing, installation and upgrade flows.
5. Advanced integrations: messaging channels, plugins, Docker, AI assistant tools, hot update.

Each replacement must keep the frontend contract stable or include a deliberate API migration with adapter code.

Success criteria:

- Product-owned command code covers the target module.
- Tests or manual verification cover success and failure paths.
- The UI does not know whether the old or new implementation is active.

## Phase 5: Production Release Hardening

Scope:

- App signing strategy.
- Installer naming and metadata.
- Update manifest and rollback behavior.
- Crash/error reporting policy.
- Security review for command execution, file writes, network fetches, and assistant tools.
- License compliance review.
- Release checklist for Windows, macOS, Linux, and Web mode if retained.

Success criteria:

- Release artifacts are reproducible.
- Risky commands require explicit UI confirmation.
- Sensitive data such as API keys, tokens, and passwords are masked in logs and UI.
- The project has a clear license and attribution strategy.


## Risks

- Full feature parity is large. Trying to rewrite commands while rebranding will delay the first runnable production build.
- Tauri packaging can fail due to Rust targets, Windows build tools, icon metadata, sidecar paths, or signing setup.
- Some current comments and docs have encoding noise. Do not rely on comments as source of truth without checking behavior.
- Hermes and OpenClaw integrations depend on external CLIs and version-specific behavior.
- AI assistant tools can execute commands and write files, so production mode needs stricter permission handling.
- Open-source license compliance affects distribution strategy.

## Non-Goals For Initial Baseline

- No Java backend.
- No full UI redesign in Phase 1.
- No removal of OpenClaw or Hermes in Phase 1.
- No rewrite of all Rust commands in Phase 1.
- No broad framework migration such as React/Vue unless a later product decision justifies it.

## Current Release Hardening Status

The production baseline has advanced through Phase 5 release hardening:

1. Product identity, installer metadata, release manifest checks, signing verification, and UI smoke tooling are in place.
2. Windows NSIS installer local smoke passed for install, launch, and uninstall.
3. Browser-driven route smoke passed for the core routes recorded in `docs/release/phase-5-ui-smoke-2026-05-15.md`.
4. Windows signing execution is wired, but formal certificate hookup is deferred until a real certificate and thumbprint are available.
5. Rust product-config helper warnings and Vite mixed dynamic/static import warnings were cleaned; the remaining frontend warning is the real `i18n` chunk size warning.
6. CI/release automation remains paused until signing inputs and publish strategy are confirmed.

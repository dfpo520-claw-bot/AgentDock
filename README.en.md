# AgentDock

AgentDock is a production desktop console for multi-engine AI agent operations, built with Tauri v2, Vite, and a Rust command layer.

## Current Phase

This repository is in the production fork baseline phase:

- Keep the existing OpenClaw and Hermes Agent engine capabilities.
- Keep the Tauri desktop shell, Web mode, service management, model management, agent management, chat, logs, diagnostics, backup, and extension modules.
- Replace product identity, icons, documentation, app metadata, and release entry points first.
- Gradually refactor configuration ownership, module boundaries, and backend command implementations afterward.

## Tech Stack

- Frontend: Vite + vanilla JavaScript
- Desktop: Tauri v2
- Capability layer: Rust Tauri commands
- Build: Node.js, Cargo, Tauri CLI

## Local Development

```bash
npm install
npm run dev
```

Desktop development:

```bash
npm run tauri dev
```

Production build:

```bash
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
npm run tauri build
```

## Project Status

The first milestone is a runnable and packageable AgentDock baseline that can be safely refactored in later phases.

Current release-hardening status:

- Windows NSIS installer local smoke passed for install, launch, and uninstall.
- Browser-driven UI route smoke passed for the core desktop routes recorded in `docs/release/phase-5-ui-smoke-2026-05-15.md`.
- Windows signing is wired, but formal certificate hookup is deferred until a real code-signing certificate and thumbprint are available.
- CI/release automation is paused until signing inputs and the publish strategy are confirmed.
- The remaining known frontend build warning is the `i18n` chunk size warning; optimize it later with locale/module lazy loading.

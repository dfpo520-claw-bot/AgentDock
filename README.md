# AgentDock

<p align="center">
  <img src="docs/agentdock-logo-brand.png" width="420" alt="AgentDock">
</p>

<p align="center">
  A production desktop console for operating multi-engine AI agents.
</p>

<p align="center">
  <a href="https://github.com/dfpo520-claw-bot/AgentDock/releases">Releases</a>
  ·
  <a href="docs/release/desktop-manual-debugging-handbook.zh-CN.md">桌面端调试手册</a>
  ·
  <a href="docs/linux-deploy.md">Linux Web 部署</a>
  ·
  <a href="https://github.com/dfpo520-claw-bot/AgentDock/issues">Issues</a>
</p>

<p align="center">
  <img alt="Tauri" src="https://img.shields.io/badge/Tauri-v2-24c8db?style=flat-square">
  <img alt="Rust" src="https://img.shields.io/badge/Rust-command%20layer-f74c00?style=flat-square">
  <img alt="Vite" src="https://img.shields.io/badge/Vite-frontend-646cff?style=flat-square">
  <img alt="Platform" src="https://img.shields.io/badge/Desktop-Windows%20%7C%20macOS%20%7C%20Linux-111827?style=flat-square">
</p>

## Overview

AgentDock is a Tauri desktop application for managing AI agent runtimes from one local console. It keeps OpenClaw and Hermes Agent workflows in the same operational surface, with focused pages for engine setup, service lifecycle, model configuration, diagnostics, logs, chat, extensions, and release validation.

The project is currently in a production fork hardening phase. The goal is to keep the existing runtime capabilities usable while replacing product identity, release configuration, module boundaries, and backend command ownership step by step.

## Highlights

- **Multi-engine workspace**: switch between OpenClaw and Hermes Agent without leaving the desktop app.
- **First-run guidance**: detect installed runtimes, guide missing dependencies, and keep version checks non-blocking.
- **Service operations**: start, stop, reload, diagnose, repair, and inspect gateway/runtime state.
- **Model management**: configure provider API keys, base URLs, model presets, and DeepAi assistant settings.
- **Built-in assistant**: run a local product assistant for setup, troubleshooting, and command-aware workflows.
- **Release hardening**: artifact manifest generation, Windows signing checks, desktop smoke notes, and manual QA checklists.
- **Product-owned configuration**: AgentDock-owned paths, update URLs, storage keys, and release metadata are separated from runtime configs.

## Tech Stack

| Layer | Technology |
| --- | --- |
| Desktop shell | Tauri v2 |
| Command layer | Rust |
| Frontend | Vite + vanilla JavaScript |
| Runtime integrations | OpenClaw, Hermes Agent |
| Packaging | Tauri bundler, NSIS, release manifest scripts |

## Quick Start

Install dependencies:

```bash
npm install
```

Start the web development server:

```bash
npm run dev
```

Start the desktop app in development mode:

```bash
npm run tauri dev
```

Build the frontend:

```bash
npm run build
```

Check the Rust command layer:

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

Build desktop packages:

```bash
npm run tauri build
```

## Useful Scripts

| Command | Purpose |
| --- | --- |
| `npm run dev` | Start Vite for frontend development. |
| `npm run build` | Build production frontend assets. |
| `npm run tauri dev` | Run the Tauri desktop app locally. |
| `npm run release:manifest` | Generate release artifact manifests and checksums. |
| `npm run release:smoke` | Verify release bundle structure. |
| `npm run smoke:ui` | Run browser route smoke checks. |
| `npm run version:sync` | Sync version metadata across project files. |

## Project Layout

```text
src/                         Frontend shell, routes, components, engines, and shared libraries
src-tauri/                   Tauri app, Rust commands, tray integration, product configuration
scripts/                     Development, deployment, release, and smoke helper scripts
docs/                        Deployment guides, release notes, UI smoke records, and design plans
tests/                       Node-based guardrail tests for product identity, release, UI, and runtime logic
public/                      Static assets served by the frontend
```

## Documentation

- [Desktop manual debugging handbook](docs/release/desktop-manual-debugging-handbook.md)
- [桌面端手动调试手册](docs/release/desktop-manual-debugging-handbook.zh-CN.md)
- [Linux deployment guide](docs/linux-deploy.md)
- [Docker deployment guide](docs/docker-deploy.md)
- [Release checklist](docs/release/phase-5-release-checklist.md)
- [UI smoke summary](docs/release/phase-5-ui-smoke-2026-05-15.md)

## Current Status

- Windows NSIS installer smoke has passed for local install, launch, and uninstall.
- Browser-driven route smoke has passed for the core desktop routes recorded under `docs/release/`.
- Formal Windows signing is wired but waiting for a real certificate thumbprint.
- CI/release automation is intentionally paused until signing and publish policy are confirmed.
- The known frontend build warning is the large `i18n` chunk; locale lazy loading is planned as a later optimization.

## Configuration And Identity

AgentDock-owned product configuration lives under the `agentdock` identity:

- Product id: `agentdock`
- Product config file: `agentdock.json`
- Product data directory: `.agentdock`
- Update manifest: `https://raw.githubusercontent.com/dfpo520-claw-bot/AgentDock/master/update/latest.json`

Runtime configuration for OpenClaw and Hermes Agent remains separate so the desktop shell can evolve without silently mutating upstream runtime contracts.

## Contributing

This repository is still being hardened toward a production-ready application boundary. Good first contributions are usually small and verifiable:

- Fix inaccurate documentation.
- Add focused guardrail tests around product identity, release metadata, or runtime command behavior.
- Improve diagnostics and error messages without changing runtime semantics.
- Split large modules only when the new boundary has a clear owner and test surface.

Before opening a change, run the checks that match the touched area:

```bash
node --test tests/*.test.js
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
```

## Upstream and Referenced Projects

AgentDock integrates with and learns from the surrounding AI agent ecosystem:

- [OpenClaw](https://github.com/1186258278/OpenClawChineseTranslation) for compatible agent runtime workflows.
- [Hermes Agent](https://github.com/NousResearch/hermes-agent) for Hermes runtime integration.
- [Tauri](https://tauri.app/) for the cross-platform desktop shell.
- [Vite](https://vite.dev/) for frontend development and production builds.

Licensing and formal distribution notices will be reviewed after the product architecture and module boundaries are finalized.

# Phase 3 Module Boundary Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` or `superpowers:executing-plans`. Execute tasks continuously without changing frontend command contracts.

**Goal:** Continue the production fork cleanup by extracting clearer ownership boundaries from `src/main.js` and `src-tauri/src/commands/config.rs` while preserving current behavior.

**Architecture:** Start with low-risk boundary-only extractions. Frontend work moves app-shell responsibilities into `src/app-shell/*`. Rust work peels gateway/runtime and app-config command families out of `config.rs` behind compatibility re-exports.

**Tech Stack:** Vite, vanilla ES modules, Tauri v2, Rust command modules, Node test runner, Cargo

---

## File Structure

### Frontend target files

- Create: `src/app-shell/gateway-banner.js` - owns gateway conflict guidance, gateway banner rendering, and guardian recovery UI flows.
- Modify: `src/main.js` - delegates gateway shell behavior to `src/app-shell/gateway-banner.js`.
- Create: `tests/app-shell-gateway-banner.test.js` - guards the extracted module surface and `main.js` delegation.

### Rust target files

- Create: `src-tauri/src/commands/app_config.rs` - owns panel config commands, migration commands, registry commands, and path-cache invalidation moved out of `config.rs`.
- Later create: `src-tauri/src/commands/gateway_runtime.rs` or equivalent focused module - owns gateway runtime / doctor / restart command bodies currently mixed into `config.rs`, `service.rs`, and `diagnose.rs`.
- Modify: `src-tauri/src/commands/config.rs` - preserve compatibility by forwarding moved command implementations.
- Modify: `src-tauri/src/commands/mod.rs` and `src-tauri/src/lib.rs` - registration only.

### Supporting guardrails

- Modify: `tests/config-surface.test.js` - enforce new app-shell ownership and later Rust boundary rules.
- Modify: this plan file as execution notes change.

---

## Task 1: Extract Frontend Gateway Banner App-Shell Boundary

**Files:**
- Create: `src/app-shell/gateway-banner.js`
- Modify: `src/main.js`
- Test: `tests/app-shell-gateway-banner.test.js`

- [x] Add a focused boundary test for `createGatewayBannerController`.
- [x] Move `openGatewayConflict`, `setupGatewayBanner`, and `showGuardianRecovery` into `src/app-shell/gateway-banner.js`.
- [x] Update `src/main.js` to import and instantiate the controller.
- [x] Verify with:

```powershell
node --test tests/app-shell-gateway-banner.test.js
.\node_modules\.bin\vite.cmd build
```

**Checkpoint:** `main.js` keeps the same command names and call sites, but the gateway shell UI logic now lives in `src/app-shell/gateway-banner.js`.

## Task 2: Add Guardrails For Frontend App-Shell Boundary

**Files:**
- Modify: `tests/config-surface.test.js`

- [ ] Assert `src/main.js` imports `./app-shell/gateway-banner.js`.
- [ ] Assert `src/main.js` no longer owns `setupGatewayBanner` / `showGuardianRecovery`.
- [ ] Assert `src/app-shell/gateway-banner.js` exports `createGatewayBannerController`.
- [ ] Verify with:

```powershell
node --test tests/config-surface.test.js
```

## Task 3: Extract Rust App Config Command Boundary

**Files:**
- Create: `src-tauri/src/commands/app_config.rs`
- Modify: `src-tauri/src/commands/config.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `tests/config-surface.test.js`

- [ ] Add a failing guardrail for `src-tauri/src/commands/app_config.rs`.
- [ ] Move these command bodies out of `config.rs`:
  - `read_panel_config`
  - `write_panel_config`
  - `get_openclaw_dir`
  - `detect_legacy_config_migration`
  - `apply_legacy_config_migration`
  - `get_npm_registry`
  - `set_npm_registry`
  - `invalidate_path_cache`
- [ ] Keep frontend command names stable through re-exports or thin forwarding wrappers.
- [ ] Verify with:

```powershell
node --test tests/config-surface.test.js
cargo test --manifest-path src-tauri/Cargo.toml product_config
cargo check --manifest-path src-tauri/Cargo.toml
```

## Task 4: Extract Rust Gateway Runtime Boundary

**Files:**
- Create: focused runtime command module under `src-tauri/src/commands/`
- Modify: `src-tauri/src/commands/config.rs`
- Modify: `src-tauri/src/commands/service.rs`
- Modify: `src-tauri/src/commands/diagnose.rs`
- Modify: `src-tauri/src/commands/mod.rs`

- [ ] Move gateway runtime leftovers out of `config.rs`, prioritizing:
  - `get_status_summary`
  - `reload_gateway`
  - `restart_gateway`
  - `doctor_fix`
  - `doctor_check`
- [ ] Group related gateway lifecycle and diagnostics with existing `service.rs` / `diagnose.rs` ownership.
- [ ] Preserve existing Tauri command names and JS call sites.
- [ ] Verify with targeted Cargo checks plus existing JS surface tests.

## Task 5: Phase 3 Verification Checkpoint

- [ ] Run full JavaScript tests:

```powershell
node --test tests/*.test.js
```

- [ ] Run Rust verification:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml product_config
cargo check --manifest-path src-tauri/Cargo.toml
```

- [ ] Run production build:

```powershell
.\node_modules\.bin\vite.cmd build
```

- [ ] Confirm working tree only contains intended Phase 3 changes.

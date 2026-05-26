# Phase 2 Configuration Ownership Design

## Goal

Move AgentDock from fork-owned configuration paths and scattered literals to product-owned configuration constants, migration detection, and compatibility aliases for existing AgentDock/OpenClaw data.

Phase 2 must keep current behavior stable. It does not rewrite engine configuration formats, remove OpenClaw or Hermes support, or change the frontend route model. Its job is to make configuration ownership explicit so later module and backend command refactors have a stable foundation.

## Current State

Phase 1 established the AgentDock production identity and kept the existing Tauri v2, Vite, and Rust command architecture.

The current configuration ownership is still inherited:

- Rust path resolution lives mostly in `src-tauri/src/commands/mod.rs`.
- Panel settings are read from `agentdock.json`.
- The default engine data directory is still `~/.openclaw`.
- `openclawDir` in panel config can override the OpenClaw config directory.
- Frontend config calls are exposed through `src/lib/tauri-api.js` as `readPanelConfig` and `writePanelConfig`.
- Existing runtime code expects OpenClaw data such as `openclaw.json`, MCP config, logs, backups, memory, agents, and gateway state to remain readable.

This is acceptable for the production baseline, but Phase 2 needs AgentDock-owned defaults and a deliberate migration story.

## Product-Owned Configuration Model

Phase 2 introduces a product configuration layer with one source of truth for app-level names, filenames, directories, release channels, and migration policy.

The product-owned defaults are:

- Product id: `agentdock`
- Product name: `AgentDock`
- Panel config filename: `agentdock.json`
- Product data directory name: `.agentdock`
- Legacy panel config filename: `agentdock.json`
- Legacy OpenClaw data directory name: `.openclaw`
- Default release channel: `stable`
- Update manifest URL: `https://raw.githubusercontent.com/dfpo520-claw-bot/AgentDock/master/update/latest.json`

The OpenClaw engine config filename remains `openclaw.json` in Phase 2 because it belongs to the OpenClaw engine contract, not to the AgentDock app shell. Renaming that file belongs to a later engine boundary or backend command replacement phase.

## Rust Ownership Boundary

Rust owns real filesystem decisions.

Create a focused Rust module for product configuration ownership, tentatively `src-tauri/src/product_config.rs`. It should expose pure path and metadata helpers that command modules can call:

- `product_config_filename() -> &'static str`
- `legacy_panel_config_filename() -> &'static str`
- `product_data_dir() -> PathBuf`
- `legacy_openclaw_data_dir() -> PathBuf`
- `panel_config_candidate_paths() -> Vec<PathBuf>`
- `panel_config_path() -> PathBuf`
- `read_panel_config_value() -> Option<Value>`
- `detect_legacy_config() -> LegacyConfigDetection`
- `apply_legacy_config_decision(decision: LegacyConfigDecision) -> LegacyConfigDecisionResult`

Existing commands can keep their public Tauri command names during Phase 2. Internally, they should call the new ownership helpers instead of duplicating path logic.

`src-tauri/src/commands/mod.rs` should become a thin compatibility surface. It may re-export helpers for existing command modules during the transition, but new path logic should live in `product_config.rs`.

## Frontend Ownership Boundary

Frontend owns user-facing migration choice and state presentation.

Create a frontend config ownership module, tentatively `src/lib/product-config.js`, that mirrors only safe display constants and Tauri command wrappers:

- `PRODUCT_CONFIG.panelConfigFile`
- `PRODUCT_CONFIG.productDataDirName`
- `PRODUCT_CONFIG.legacyPanelConfigFile`
- `PRODUCT_CONFIG.legacyDataDirName`
- `PRODUCT_CONFIG.releaseChannel`
- `checkLegacyConfigMigration()`
- `applyLegacyConfigDecision(decision)`

The frontend must not reconstruct OS-specific paths. It should ask Rust for exact paths and detection results.

## Migration Detection

On startup, AgentDock checks whether a legacy config decision is needed.

A legacy migration prompt is needed when all of these are true:

- Product-owned panel config does not exist.
- A legacy panel config or legacy OpenClaw data directory exists.
- The user has not already recorded a migration decision.

The detection result returned to the frontend should include:

- `needed: boolean`
- `productConfigPath: string`
- `legacyConfigPath: string | null`
- `legacyDataDir: string | null`
- `detectedItems: string[]`
- `recommendedAction: "import" | "ignore"`

The recommended action should be `import` when a valid legacy `agentdock.json` exists. It should be `ignore` when only weak legacy signals exist, such as an empty legacy directory.

## First-Run User Choice

The user selected this strategy for legacy data:

1. First launch shows a prompt if legacy data is detected.
2. The prompt offers two actions: `Import` and `Ignore`.
3. Import copies compatible panel settings into `agentdock.json`.
4. Ignore writes a small AgentDock migration state so the prompt does not repeat.
5. Neither action deletes legacy data.

The first-run prompt should appear after the app shell can render but before users enter deep configuration flows. A modal is sufficient for Phase 2. It should be explicit that legacy data is copied or ignored, not moved or deleted.

## Import Rules

Import should be conservative.

When importing `agentdock.json` into `agentdock.json`:

- Copy recognized panel-level settings such as proxy settings, custom CLI paths, custom OpenClaw directory, search paths, download source choices, and tool path overrides.
- Add AgentDock migration metadata under an app-owned key, for example:

```json
{
  "agentdock": {
    "configVersion": 1,
    "migration": {
      "legacyProduct": "AgentDock",
      "decision": "imported",
      "sourceConfigPath": "...",
      "timestamp": "2026-05-14T00:00:00.000Z"
    }
  }
}
```

- Do not rewrite `openclaw.json`.
- Do not move logs, backups, memory files, agents, or session data in Phase 2.
- Do not delete legacy files.
- If an `openclawDir` override exists, preserve it so existing OpenClaw engine data remains reachable.

If import fails because the legacy file is unreadable or invalid JSON, AgentDock should keep the legacy data untouched and return an error that the frontend can display. The user can then choose Ignore or fix the file and retry.

## Ignore Rules

Ignoring legacy data should create product-owned panel config with migration metadata:

```json
{
  "agentdock": {
    "configVersion": 1,
    "migration": {
      "legacyProduct": "AgentDock",
      "decision": "ignored",
      "sourceConfigPath": "...",
      "timestamp": "2026-05-14T00:00:00.000Z"
    }
  }
}
```

Ignore does not prevent the user from manually configuring an OpenClaw directory later in Settings.

## Compatibility Aliases

Compatibility aliases keep existing installs usable while product-owned defaults are introduced.

Resolution order for panel config:

1. Product-owned panel config: `<productDataDir>/agentdock.json`
2. Existing legacy panel config with a recorded import decision: `<legacyOpenClawDir>/agentdock.json`
3. Other legacy panel config candidates found by the existing Windows home-directory fallback logic
4. Product-owned default path for new file creation

Resolution order for OpenClaw engine data:

1. Explicit `openclawDir` from AgentDock panel config
2. Explicit `openclawDir` from legacy panel config while compatibility aliasing is active
3. Existing legacy OpenClaw directory if it contains `openclaw.json`
4. Default legacy OpenClaw directory `~/.openclaw`

AgentDock should not default OpenClaw engine data to `.agentdock` in Phase 2 because the OpenClaw engine still expects its own config layout. Product app config moves first; engine data ownership moves later when engine adapters are isolated.

## Tauri Commands

Add two new Tauri commands:

- `detect_legacy_config_migration`
- `apply_legacy_config_migration`

Keep existing commands:

- `read_panel_config`
- `write_panel_config`
- `get_openclaw_dir`

During Phase 2, existing frontend callers should continue to work. New startup UI should use the new migration commands.

The command payloads should use simple JSON-compatible values so the frontend does not depend on Rust-specific structures.

## UI Integration

Add a startup migration check near the app boot path in `src/main.js` or a small app-shell helper module.

The UI should:

- Call `api.detectLegacyConfigMigration()` once after Tauri API readiness.
- Show a modal only when `needed` is true.
- Let the user choose Import or Ignore.
- Display the detected legacy path.
- Show a non-destructive explanation.
- On success, invalidate cached config data and continue normal app loading.
- On failure, show a toast or inline error and keep the modal open.

The modal is part of app-shell behavior, not a Settings page redesign.

## Tests

Testing should focus on path policy and migration decisions.

Rust tests should cover:

- New install chooses product-owned `agentdock.json` path.
- Legacy `agentdock.json` is detected when no product config exists.
- Product config suppresses migration prompt.
- Import copies compatible panel config keys and writes AgentDock migration metadata.
- Ignore writes only AgentDock migration metadata.
- Legacy alias keeps `openclawDir` readable until the user creates product config.

Frontend tests should cover:

- Product config constants expose the expected filenames and directory names.
- Startup migration wrapper calls detection and decision APIs.
- Brand/config surface tests prevent new `agentdock.json` literals outside declared compatibility modules.

Existing tests must keep passing:

- `node --test tests/*.test.js`
- `cargo check --manifest-path src-tauri/Cargo.toml`
- `.\node_modules\.bin\vite.cmd build`

## Non-Goals

- Do not rename `openclaw.json`.
- Do not migrate OpenClaw engine data, logs, backups, memory, agents, or session history.
- Do not remove OpenClaw or Hermes features.
- Do not change existing Tauri command names used by established screens.
- Do not introduce Java backend code.
- Do not redesign Settings or Setup pages beyond the minimal migration prompt integration.

## Risks And Mitigations

- Risk: path logic differs by operating system.
  Mitigation: keep path resolution in Rust and add tests for pure path helpers where possible.

- Risk: users lose access to existing OpenClaw config after product config moves.
  Mitigation: preserve `openclawDir` during import and keep legacy alias fallback active.

- Risk: migration prompt repeats.
  Mitigation: record both import and ignore decisions in product-owned `agentdock.json`.

- Risk: inherited encoding noise makes old comments unreliable.
  Mitigation: use behavior and tests as source of truth; avoid relying on comments.

## Success Criteria

- New installs create and read product-owned `agentdock.json`.
- AgentDock can detect legacy AgentDock/OpenClaw data.
- First launch prompts the user to Import or Ignore legacy data when appropriate.
- Import and Ignore are non-destructive.
- Existing OpenClaw engine configuration remains reachable through explicit settings or compatibility aliases.
- Product-owned constants drive config filenames, directory names, update channel, and migration policy.
- All existing Phase 1 verification commands still pass.

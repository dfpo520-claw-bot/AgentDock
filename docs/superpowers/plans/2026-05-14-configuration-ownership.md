# Configuration Ownership Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move AgentDock to product-owned panel configuration while preserving compatibility aliases for existing ClawPanel/OpenClaw data.

**Architecture:** Rust owns real filesystem decisions through a new `product_config` module and exposes migration commands through the existing Tauri command layer. The frontend owns display constants and the first-run Import/Ignore prompt, while existing OpenClaw engine data remains in the OpenClaw layout for this phase.

**Tech Stack:** Tauri v2, Rust 2021, serde/serde_json, Vite 6, vanilla JavaScript modules, Node.js `node:test`.

---

## Scope

This plan implements `docs/superpowers/specs/2026-05-14-configuration-ownership-design.md`.

Phase 2 changes only panel configuration ownership and migration decision flow:

- Product-owned panel config file: `agentdock.json`
- Product-owned app data directory: `.agentdock`
- Legacy panel config file: `clawpanel.json`
- Legacy engine data directory: `.openclaw`
- First-run legacy data strategy: prompt user to Import or Ignore
- Compatibility: keep OpenClaw engine config and data readable

Do not rename `openclaw.json`, move logs/backups/memory/agents/session data, or remove OpenClaw/Hermes behavior.

## File Structure

Create:

- `src-tauri/src/product_config.rs` - product config constants, path helpers, detection, import/ignore application, unit tests.
- `src/lib/product-config.js` - frontend constants and wrappers for migration commands.
- `tests/product-config.test.js` - frontend tests for product config constants.
- `tests/config-surface.test.js` - guardrails for legacy config literals.

Modify:

- `src-tauri/src/lib.rs` - register `product_config` module and new Tauri commands.
- `src-tauri/src/commands/mod.rs` - delegate panel config path/value helpers to `product_config` while keeping old helper names available.
- `src-tauri/src/commands/config.rs` - expose `detect_legacy_config_migration` and `apply_legacy_config_migration`.
- `src/lib/tauri-api.js` - add API wrappers for migration detection and decision.
- `src/main.js` - run startup migration check and show Import/Ignore modal.

Verification:

- `node --test tests/product-config.test.js`
- `node --test tests/config-surface.test.js`
- `node --test tests/*.test.js`
- `cargo test --manifest-path src-tauri/Cargo.toml product_config`
- `cargo check --manifest-path src-tauri/Cargo.toml`
- `.\node_modules\.bin\vite.cmd build`

## Task 1: Confirm Phase 2 Baseline

**Files:**
- Read: `docs/superpowers/specs/2026-05-14-configuration-ownership-design.md`
- Read: `src-tauri/src/commands/mod.rs`
- Read: `src-tauri/src/commands/config.rs`
- Read: `src/lib/tauri-api.js`
- Read: `src/main.js`

- [ ] **Step 1: Check branch and working tree**

Run:

```powershell
git -c safe.directory=D:/workSpace/ohter/clawpanel branch --show-current
git -c safe.directory=D:/workSpace/ohter/clawpanel status --short
```

Expected:

```text
codex/clawpanel-new
```

If unrelated files appear, do not revert them. Record them in the task notes and avoid editing them.

- [ ] **Step 2: Run current JavaScript tests**

Run:

```powershell
node --test tests/*.test.js
```

Expected:

```text
# pass
```

- [ ] **Step 3: Run current Rust check**

Run:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected:

```text
Finished `dev` profile
```

- [ ] **Step 4: Commit no code**

No commit is expected for this task unless baseline notes are added.

## Task 2: Add Frontend Product Config Constants

**Files:**
- Create: `src/lib/product-config.js`
- Create: `tests/product-config.test.js`

- [ ] **Step 1: Write the failing frontend constants test**

Create `tests/product-config.test.js`:

```javascript
import test from 'node:test'
import assert from 'node:assert/strict'
import {
  PRODUCT_CONFIG,
  isKnownLegacyPanelConfigFile,
  isProductPanelConfigFile,
} from '../src/lib/product-config.js'

test('PRODUCT_CONFIG exposes product-owned config defaults', () => {
  assert.equal(PRODUCT_CONFIG.productId, 'agentdock')
  assert.equal(PRODUCT_CONFIG.panelConfigFile, 'agentdock.json')
  assert.equal(PRODUCT_CONFIG.productDataDirName, '.agentdock')
  assert.equal(PRODUCT_CONFIG.legacyPanelConfigFile, 'clawpanel.json')
  assert.equal(PRODUCT_CONFIG.legacyDataDirName, '.openclaw')
  assert.equal(PRODUCT_CONFIG.releaseChannel, 'stable')
})

test('config filename helpers distinguish product and legacy files', () => {
  assert.equal(isProductPanelConfigFile('agentdock.json'), true)
  assert.equal(isProductPanelConfigFile('clawpanel.json'), false)
  assert.equal(isKnownLegacyPanelConfigFile('clawpanel.json'), true)
  assert.equal(isKnownLegacyPanelConfigFile('agentdock.json'), false)
})
```

- [ ] **Step 2: Run the test and verify RED**

Run:

```powershell
node --test tests/product-config.test.js
```

Expected:

```text
not ok
Cannot find module
```

- [ ] **Step 3: Implement frontend constants**

Create `src/lib/product-config.js`:

```javascript
import { api } from './tauri-api.js'

export const PRODUCT_CONFIG = Object.freeze({
  productId: 'agentdock',
  panelConfigFile: 'agentdock.json',
  productDataDirName: '.agentdock',
  legacyPanelConfigFile: 'clawpanel.json',
  legacyDataDirName: '.openclaw',
  releaseChannel: 'stable',
})

export function isProductPanelConfigFile(name) {
  return String(name || '').trim() === PRODUCT_CONFIG.panelConfigFile
}

export function isKnownLegacyPanelConfigFile(name) {
  return String(name || '').trim() === PRODUCT_CONFIG.legacyPanelConfigFile
}

export function checkLegacyConfigMigration() {
  return api.detectLegacyConfigMigration()
}

export function applyLegacyConfigDecision(decision) {
  return api.applyLegacyConfigMigration(decision)
}
```

- [ ] **Step 4: Run the test and verify GREEN**

Run:

```powershell
node --test tests/product-config.test.js
```

Expected:

```text
# pass
```

- [ ] **Step 5: Commit frontend constants**

Run:

```powershell
git -c safe.directory=D:/workSpace/ohter/clawpanel add src/lib/product-config.js tests/product-config.test.js
git -c safe.directory=D:/workSpace/ohter/clawpanel commit -m "feat: add product config constants"
```

Expected:

```text
[codex/clawpanel-new <hash>] feat: add product config constants
```

## Task 3: Add Rust Product Config Ownership Module

**Files:**
- Create: `src-tauri/src/product_config.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write the Rust module with unit tests**

Create `src-tauri/src/product_config.rs` with this initial implementation and tests:

```rust
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const PRODUCT_ID: &str = "agentdock";
pub const PRODUCT_CONFIG_FILENAME: &str = "agentdock.json";
pub const PRODUCT_DATA_DIR_NAME: &str = ".agentdock";
pub const LEGACY_PANEL_CONFIG_FILENAME: &str = "clawpanel.json";
pub const LEGACY_DATA_DIR_NAME: &str = ".openclaw";
pub const LEGACY_PRODUCT_NAME: &str = "ClawPanel";

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LegacyConfigDetection {
    pub needed: bool,
    pub product_config_path: String,
    pub legacy_config_path: Option<String>,
    pub legacy_data_dir: Option<String>,
    pub detected_items: Vec<String>,
    pub recommended_action: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacyConfigDecision {
    pub action: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LegacyConfigDecisionResult {
    pub action: String,
    pub product_config_path: String,
    pub imported_keys: Vec<String>,
}

pub fn product_config_filename() -> &'static str {
    PRODUCT_CONFIG_FILENAME
}

pub fn legacy_panel_config_filename() -> &'static str {
    LEGACY_PANEL_CONFIG_FILENAME
}

fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_default()
}

pub fn product_data_dir() -> PathBuf {
    home_dir().join(PRODUCT_DATA_DIR_NAME)
}

pub fn legacy_openclaw_data_dir() -> PathBuf {
    home_dir().join(LEGACY_DATA_DIR_NAME)
}

fn path_key(path: &Path) -> String {
    #[cfg(target_os = "windows")]
    {
        path.to_string_lossy().replace('/', "\\").to_lowercase()
    }
    #[cfg(not(target_os = "windows"))]
    {
        path.to_string_lossy().to_string()
    }
}

fn push_unique(paths: &mut Vec<PathBuf>, path: PathBuf) {
    let key = path_key(&path);
    if !paths.iter().any(|existing| path_key(existing) == key) {
        paths.push(path);
    }
}

pub fn panel_config_candidate_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    push_unique(&mut paths, product_data_dir().join(PRODUCT_CONFIG_FILENAME));
    push_unique(
        &mut paths,
        legacy_openclaw_data_dir().join(LEGACY_PANEL_CONFIG_FILENAME),
    );

    #[cfg(target_os = "windows")]
    {
        if let Ok(profile) = std::env::var("USERPROFILE") {
            let trimmed = profile.trim();
            if !trimmed.is_empty() {
                push_unique(
                    &mut paths,
                    PathBuf::from(trimmed)
                        .join(LEGACY_DATA_DIR_NAME)
                        .join(LEGACY_PANEL_CONFIG_FILENAME),
                );
            }
        }
        if let (Ok(home_drive), Ok(home_path)) =
            (std::env::var("HOMEDRIVE"), std::env::var("HOMEPATH"))
        {
            let combined = format!("{}{}", home_drive.trim(), home_path.trim());
            if !combined.trim().is_empty() {
                push_unique(
                    &mut paths,
                    PathBuf::from(combined)
                        .join(LEGACY_DATA_DIR_NAME)
                        .join(LEGACY_PANEL_CONFIG_FILENAME),
                );
            }
        }
        if let Ok(appdata) = std::env::var("APPDATA") {
            let appdata_path = PathBuf::from(appdata.trim());
            if let Some(profile_dir) = appdata_path.parent().and_then(|p| p.parent()) {
                push_unique(
                    &mut paths,
                    profile_dir
                        .join(LEGACY_DATA_DIR_NAME)
                        .join(LEGACY_PANEL_CONFIG_FILENAME),
                );
            }
        }
    }

    paths
}

fn read_json_file(path: &Path) -> Option<Value> {
    fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
}

fn has_recorded_migration(value: &Value) -> bool {
    value.pointer("/agentdock/migration/decision")
        .and_then(Value::as_str)
        .is_some()
}

pub fn read_panel_config_value() -> Option<Value> {
    for path in panel_config_candidate_paths() {
        if let Some(value) = read_json_file(&path) {
            return Some(value);
        }
    }
    None
}

pub fn panel_config_path() -> PathBuf {
    let candidates = panel_config_candidate_paths();
    for path in &candidates {
        if read_json_file(path).is_some() {
            return path.clone();
        }
    }
    candidates
        .into_iter()
        .next()
        .unwrap_or_else(|| product_data_dir().join(PRODUCT_CONFIG_FILENAME))
}

pub fn detect_legacy_config() -> LegacyConfigDetection {
    detect_legacy_config_for_paths(
        product_data_dir().join(PRODUCT_CONFIG_FILENAME),
        legacy_openclaw_data_dir().join(LEGACY_PANEL_CONFIG_FILENAME),
        legacy_openclaw_data_dir(),
    )
}

fn detect_legacy_config_for_paths(
    product_config_path: PathBuf,
    legacy_config_path: PathBuf,
    legacy_data_dir: PathBuf,
) -> LegacyConfigDetection {
    let product_config = read_json_file(&product_config_path);
    let legacy_config = read_json_file(&legacy_config_path);
    let legacy_dir_exists = legacy_data_dir.exists();
    let mut detected_items = Vec::new();

    if legacy_config.is_some() {
        detected_items.push("legacyPanelConfig".to_string());
    }
    if legacy_dir_exists {
        detected_items.push("legacyDataDir".to_string());
    }

    let already_decided = product_config
        .as_ref()
        .is_some_and(has_recorded_migration);
    let needed = product_config.is_none()
        && !already_decided
        && (legacy_config.is_some() || legacy_dir_exists);
    let recommended_action = if legacy_config.is_some() {
        "import"
    } else {
        "ignore"
    }
    .to_string();

    LegacyConfigDetection {
        needed,
        product_config_path: product_config_path.to_string_lossy().to_string(),
        legacy_config_path: legacy_config
            .as_ref()
            .map(|_| legacy_config_path.to_string_lossy().to_string()),
        legacy_data_dir: legacy_dir_exists.then(|| legacy_data_dir.to_string_lossy().to_string()),
        detected_items,
        recommended_action,
    }
}

pub fn apply_legacy_config_decision(
    decision: LegacyConfigDecision,
) -> Result<LegacyConfigDecisionResult, String> {
    apply_legacy_config_decision_for_paths(
        decision,
        product_data_dir().join(PRODUCT_CONFIG_FILENAME),
        legacy_openclaw_data_dir().join(LEGACY_PANEL_CONFIG_FILENAME),
    )
}

fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn migration_metadata(action: &str, source_config_path: &Path) -> Value {
    json!({
        "configVersion": 1,
        "migration": {
            "legacyProduct": LEGACY_PRODUCT_NAME,
            "decision": action,
            "sourceConfigPath": source_config_path.to_string_lossy(),
            "timestamp": now_millis().to_string()
        }
    })
}

const IMPORTABLE_PANEL_KEYS: &[&str] = &[
    "networkProxy",
    "useProxy",
    "openclawDir",
    "openclawSearchPaths",
    "openclawCliPath",
    "nodePath",
    "gitPath",
    "npmRegistry",
    "downloadSource",
    "githubMirror",
    "gitMirror",
];

fn build_imported_config(legacy: &Value, source_config_path: &Path) -> (Value, Vec<String>) {
    let mut root = Map::new();
    let mut imported_keys = Vec::new();
    if let Some(obj) = legacy.as_object() {
        for key in IMPORTABLE_PANEL_KEYS {
            if let Some(value) = obj.get(*key) {
                root.insert((*key).to_string(), value.clone());
                imported_keys.push((*key).to_string());
            }
        }
    }
    root.insert(
        PRODUCT_ID.to_string(),
        migration_metadata("imported", source_config_path),
    );
    (Value::Object(root), imported_keys)
}

fn build_ignored_config(source_config_path: &Path) -> Value {
    let mut root = Map::new();
    root.insert(
        PRODUCT_ID.to_string(),
        migration_metadata("ignored", source_config_path),
    );
    Value::Object(root)
}

fn apply_legacy_config_decision_for_paths(
    decision: LegacyConfigDecision,
    product_config_path: PathBuf,
    legacy_config_path: PathBuf,
) -> Result<LegacyConfigDecisionResult, String> {
    let action = decision.action.trim();
    if action != "import" && action != "ignore" {
        return Err("migration action must be import or ignore".into());
    }

    let (config, imported_keys) = if action == "import" {
        let legacy = read_json_file(&legacy_config_path)
            .ok_or_else(|| "legacy panel config is missing or invalid".to_string())?;
        build_imported_config(&legacy, &legacy_config_path)
    } else {
        (build_ignored_config(&legacy_config_path), Vec::new())
    };

    if let Some(parent) = product_config_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("create product config dir failed: {e}"))?;
    }
    let json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("serialize product config failed: {e}"))?;
    fs::write(&product_config_path, json)
        .map_err(|e| format!("write product config failed: {e}"))?;

    Ok(LegacyConfigDecisionResult {
        action: action.to_string(),
        product_config_path: product_config_path.to_string_lossy().to_string(),
        imported_keys,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "agentdock-product-config-{name}-{}",
            now_millis()
        ));
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn detects_legacy_config_when_product_config_is_missing() {
        let root = temp_root("detect");
        let product = root.join(PRODUCT_DATA_DIR_NAME).join(PRODUCT_CONFIG_FILENAME);
        let legacy_dir = root.join(LEGACY_DATA_DIR_NAME);
        fs::create_dir_all(&legacy_dir).unwrap();
        let legacy = legacy_dir.join(LEGACY_PANEL_CONFIG_FILENAME);
        fs::write(&legacy, r#"{"networkProxy":"http://127.0.0.1:7897"}"#).unwrap();

        let detected = detect_legacy_config_for_paths(product, legacy, legacy_dir);

        assert!(detected.needed);
        assert_eq!(detected.recommended_action, "import");
        assert!(detected.detected_items.contains(&"legacyPanelConfig".to_string()));
    }

    #[test]
    fn product_config_suppresses_migration_prompt() {
        let root = temp_root("product-present");
        let product_dir = root.join(PRODUCT_DATA_DIR_NAME);
        fs::create_dir_all(&product_dir).unwrap();
        let product = product_dir.join(PRODUCT_CONFIG_FILENAME);
        fs::write(&product, r#"{"agentdock":{"migration":{"decision":"ignored"}}}"#).unwrap();
        let legacy_dir = root.join(LEGACY_DATA_DIR_NAME);
        fs::create_dir_all(&legacy_dir).unwrap();
        let legacy = legacy_dir.join(LEGACY_PANEL_CONFIG_FILENAME);
        fs::write(&legacy, r#"{"openclawDir":"D:\\Data\\.openclaw"}"#).unwrap();

        let detected = detect_legacy_config_for_paths(product, legacy, legacy_dir);

        assert!(!detected.needed);
    }

    #[test]
    fn import_copies_compatible_keys_and_records_metadata() {
        let root = temp_root("import");
        let product = root.join(PRODUCT_DATA_DIR_NAME).join(PRODUCT_CONFIG_FILENAME);
        let legacy_dir = root.join(LEGACY_DATA_DIR_NAME);
        fs::create_dir_all(&legacy_dir).unwrap();
        let legacy = legacy_dir.join(LEGACY_PANEL_CONFIG_FILENAME);
        fs::write(
            &legacy,
            r#"{"networkProxy":"http://127.0.0.1:7897","openclawDir":"D:\\OpenClaw","unrelated":true}"#,
        )
        .unwrap();

        let result = apply_legacy_config_decision_for_paths(
            LegacyConfigDecision { action: "import".into() },
            product.clone(),
            legacy,
        )
        .unwrap();
        let written: Value = serde_json::from_str(&fs::read_to_string(product).unwrap()).unwrap();

        assert_eq!(result.action, "import");
        assert!(result.imported_keys.contains(&"networkProxy".to_string()));
        assert!(result.imported_keys.contains(&"openclawDir".to_string()));
        assert_eq!(written["networkProxy"], "http://127.0.0.1:7897");
        assert!(written.pointer("/agentdock/migration/decision").is_some());
        assert!(written.get("unrelated").is_none());
    }

    #[test]
    fn ignore_records_decision_without_importing_legacy_keys() {
        let root = temp_root("ignore");
        let product = root.join(PRODUCT_DATA_DIR_NAME).join(PRODUCT_CONFIG_FILENAME);
        let legacy = root.join(LEGACY_DATA_DIR_NAME).join(LEGACY_PANEL_CONFIG_FILENAME);

        let result = apply_legacy_config_decision_for_paths(
            LegacyConfigDecision { action: "ignore".into() },
            product.clone(),
            legacy,
        )
        .unwrap();
        let written: Value = serde_json::from_str(&fs::read_to_string(product).unwrap()).unwrap();

        assert_eq!(result.action, "ignore");
        assert!(result.imported_keys.is_empty());
        assert_eq!(written.pointer("/agentdock/migration/decision").unwrap(), "ignored");
        assert!(written.get("networkProxy").is_none());
    }
}
```

- [ ] **Step 2: Register the module**

Modify the top of `src-tauri/src/lib.rs`:

```rust
mod commands;
mod models;
mod product_config;
mod tray;
mod utils;
```

- [ ] **Step 3: Run targeted Rust tests and verify GREEN**

Run:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
cargo test --manifest-path src-tauri/Cargo.toml product_config
```

Expected:

```text
test result: ok
```

- [ ] **Step 4: Commit Rust product config module**

Run:

```powershell
git -c safe.directory=D:/workSpace/ohter/clawpanel add src-tauri/src/product_config.rs src-tauri/src/lib.rs
git -c safe.directory=D:/workSpace/ohter/clawpanel commit -m "feat: add product config ownership module"
```

Expected:

```text
[codex/clawpanel-new <hash>] feat: add product config ownership module
```

## Task 4: Delegate Existing Panel Config Helpers

**Files:**
- Modify: `src-tauri/src/commands/mod.rs`

- [ ] **Step 1: Update helper implementations**

In `src-tauri/src/commands/mod.rs`, keep the existing helper names but delegate product-owned panel config behavior:

```rust
fn panel_config_candidate_paths() -> Vec<PathBuf> {
    crate::product_config::panel_config_candidate_paths()
}

fn panel_config_path() -> PathBuf {
    crate::product_config::panel_config_path()
}

pub fn read_panel_config_value() -> Option<serde_json::Value> {
    crate::product_config::read_panel_config_value()
}
```

Keep `default_openclaw_dir()` and `openclaw_dir()` available for OpenClaw engine behavior. Do not change `openclaw.json` handling in this task.

- [ ] **Step 2: Run Rust check**

Run:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected:

```text
Finished `dev` profile
```

- [ ] **Step 3: Run targeted Rust tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml product_config
```

Expected:

```text
test result: ok
```

- [ ] **Step 4: Commit helper delegation**

Run:

```powershell
git -c safe.directory=D:/workSpace/ohter/clawpanel add src-tauri/src/commands/mod.rs
git -c safe.directory=D:/workSpace/ohter/clawpanel commit -m "refactor: route panel config paths through product ownership"
```

Expected:

```text
[codex/clawpanel-new <hash>] refactor: route panel config paths through product ownership
```

## Task 5: Add Migration Tauri Commands

**Files:**
- Modify: `src-tauri/src/commands/config.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/lib/tauri-api.js`

- [ ] **Step 1: Add command functions**

Add near the existing panel config commands in `src-tauri/src/commands/config.rs`:

```rust
#[tauri::command]
pub fn detect_legacy_config_migration() -> Result<Value, String> {
    serde_json::to_value(crate::product_config::detect_legacy_config())
        .map_err(|e| format!("serialize legacy migration detection failed: {e}"))
}

#[tauri::command]
pub fn apply_legacy_config_migration(action: String) -> Result<Value, String> {
    let result = crate::product_config::apply_legacy_config_decision(
        crate::product_config::LegacyConfigDecision { action },
    )?;
    serde_json::to_value(result)
        .map_err(|e| format!("serialize legacy migration result failed: {e}"))
}
```

- [ ] **Step 2: Register commands**

Add both commands to `tauri::generate_handler!` in `src-tauri/src/lib.rs` near `read_panel_config` and `write_panel_config`:

```rust
config::detect_legacy_config_migration,
config::apply_legacy_config_migration,
```

- [ ] **Step 3: Add frontend API wrappers**

In `src/lib/tauri-api.js`, add wrappers near `readPanelConfig`:

```javascript
detectLegacyConfigMigration: () => invoke('detect_legacy_config_migration'),
applyLegacyConfigMigration: (action) => {
  invalidate()
  return invoke('apply_legacy_config_migration', { action }).then(r => {
    invoke('invalidate_path_cache').catch(() => {})
    return r
  })
},
```

- [ ] **Step 4: Run Rust check**

Run:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected:

```text
Finished `dev` profile
```

- [ ] **Step 5: Run frontend product config tests**

Run:

```powershell
node --test tests/product-config.test.js
```

Expected:

```text
# pass
```

- [ ] **Step 6: Commit migration commands**

Run:

```powershell
git -c safe.directory=D:/workSpace/ohter/clawpanel add src-tauri/src/commands/config.rs src-tauri/src/lib.rs src/lib/tauri-api.js
git -c safe.directory=D:/workSpace/ohter/clawpanel commit -m "feat: expose legacy config migration commands"
```

Expected:

```text
[codex/clawpanel-new <hash>] feat: expose legacy config migration commands
```

## Task 6: Add First-Run Migration Prompt

**Files:**
- Modify: `src/main.js`
- Modify: `src/lib/product-config.js`

- [ ] **Step 1: Extend frontend config wrapper**

Update `src/lib/product-config.js`:

```javascript
export function describeLegacyConfigDetection(detection) {
  const items = Array.isArray(detection?.detectedItems) ? detection.detectedItems : []
  const legacyPath = detection?.legacyConfigPath || detection?.legacyDataDir || ''
  return {
    needed: detection?.needed === true,
    items,
    legacyPath,
    recommendedAction: detection?.recommendedAction || 'ignore',
  }
}
```

- [ ] **Step 2: Add startup prompt helper**

In `src/main.js`, import the helpers and modal:

```javascript
import { showContentModal } from './components/modal.js'
import { checkLegacyConfigMigration, applyLegacyConfigDecision, describeLegacyConfigDetection } from './lib/product-config.js'
```

If `showContentModal` is already imported in `src/main.js`, extend the existing import instead of adding a duplicate import.

Add this helper near the update checker helpers:

```javascript
let legacyMigrationChecked = false

async function checkLegacyConfigMigrationOnStartup() {
  if (legacyMigrationChecked) return
  legacyMigrationChecked = true
  let detection
  try {
    detection = await checkLegacyConfigMigration()
  } catch (err) {
    console.warn('[AgentDock] legacy config detection failed', err)
    return
  }

  const summary = describeLegacyConfigDetection(detection)
  if (!summary.needed) return

  const overlay = showContentModal({
    title: 'Import existing configuration?',
    width: 520,
    content: `
      <div style="display:grid;gap:12px;line-height:1.6">
        <p style="margin:0;color:var(--text-secondary)">AgentDock found an existing ClawPanel/OpenClaw configuration. You can copy compatible panel settings into AgentDock or ignore them and start fresh.</p>
        <div style="padding:10px 12px;border:1px solid var(--border);border-radius:8px;background:var(--bg-secondary);word-break:break-all">
          ${summary.legacyPath || 'Legacy configuration detected'}
        </div>
        <p style="margin:0;color:var(--text-tertiary);font-size:var(--font-size-sm)">Import and Ignore are both non-destructive. Legacy files will not be moved or deleted.</p>
        <div id="legacy-migration-error" style="display:none;color:var(--danger);font-size:var(--font-size-sm)"></div>
      </div>
    `,
    buttons: [
      { id: 'btn-legacy-import', label: 'Import', className: 'btn btn-primary btn-sm' },
      { id: 'btn-legacy-ignore', label: 'Ignore', className: 'btn btn-secondary btn-sm' },
    ],
  })

  const errorEl = overlay.querySelector('#legacy-migration-error')
  const apply = async (action) => {
    try {
      await applyLegacyConfigDecision(action)
      overlay.close()
      toast(action === 'import' ? 'Configuration imported' : 'Legacy configuration ignored', 'success')
    } catch (err) {
      if (errorEl) {
        errorEl.style.display = 'block'
        errorEl.textContent = err?.message || String(err)
      }
    }
  }

  overlay.querySelector('#btn-legacy-import')?.addEventListener('click', () => apply('import'))
  overlay.querySelector('#btn-legacy-ignore')?.addEventListener('click', () => apply('ignore'))
}
```

- [ ] **Step 3: Call the helper after app shell startup**

In the existing startup flow near `startUpdateChecker()` in `src/main.js`, add:

```javascript
checkLegacyConfigMigrationOnStartup()
```

Do not block app startup on this promise.

- [ ] **Step 4: Run frontend build**

Run:

```powershell
.\node_modules\.bin\vite.cmd build
```

Expected:

```text
built
```

- [ ] **Step 5: Commit startup prompt**

Run:

```powershell
git -c safe.directory=D:/workSpace/ohter/clawpanel add src/main.js src/lib/product-config.js
git -c safe.directory=D:/workSpace/ohter/clawpanel commit -m "feat: prompt for legacy config migration"
```

Expected:

```text
[codex/clawpanel-new <hash>] feat: prompt for legacy config migration
```

## Task 7: Add Config Surface Guardrails

**Files:**
- Create: `tests/config-surface.test.js`

- [ ] **Step 1: Write config surface tests**

Create `tests/config-surface.test.js`:

```javascript
import test from 'node:test'
import assert from 'node:assert/strict'
import fs from 'node:fs'

const ALLOWED_LEGACY_FILES = new Set([
  'docs/superpowers/specs/2026-05-14-configuration-ownership-design.md',
  'docs/superpowers/plans/2026-05-14-configuration-ownership.md',
  'src-tauri/src/product_config.rs',
  'src-tauri/src/commands/mod.rs',
  'src-tauri/src/commands/config.rs',
  'src/lib/product-config.js',
  'src/pages/settings.js',
  'src/pages/setup.js',
  'src/locales/modules/settings.js',
  'src/locales/modules/setup.js',
])

const SCANNED_FILES = [
  'src-tauri/src/product_config.rs',
  'src-tauri/src/commands/mod.rs',
  'src-tauri/src/commands/config.rs',
  'src/lib/product-config.js',
  'src/lib/tauri-api.js',
  'src/main.js',
  'src/pages/settings.js',
  'src/pages/setup.js',
  'src/locales/modules/settings.js',
  'src/locales/modules/setup.js',
]

test('product config surfaces declare AgentDock-owned config names', () => {
  const frontend = fs.readFileSync('src/lib/product-config.js', 'utf8')
  const rust = fs.readFileSync('src-tauri/src/product_config.rs', 'utf8')

  assert.match(frontend, /panelConfigFile:\s*'agentdock\.json'/)
  assert.match(frontend, /productDataDirName:\s*'\.agentdock'/)
  assert.match(rust, /PRODUCT_CONFIG_FILENAME:\s*&str\s*=\s*"agentdock\.json"/)
  assert.match(rust, /PRODUCT_DATA_DIR_NAME:\s*&str\s*=\s*"\.agentdock"/)
})

test('legacy panel config literals stay in declared compatibility files', () => {
  for (const file of SCANNED_FILES) {
    const text = fs.readFileSync(file, 'utf8')
    if (!text.includes('clawpanel.json') && !text.includes('.openclaw')) continue
    assert.equal(
      ALLOWED_LEGACY_FILES.has(file),
      true,
      `${file} uses legacy config names outside the compatibility boundary`
    )
  }
})
```

- [ ] **Step 2: Run guardrail tests**

Run:

```powershell
node --test tests/config-surface.test.js
```

Expected:

```text
# pass
```

- [ ] **Step 3: Run all JavaScript tests**

Run:

```powershell
node --test tests/*.test.js
```

Expected:

```text
# pass
```

- [ ] **Step 4: Commit guardrails**

Run:

```powershell
git -c safe.directory=D:/workSpace/ohter/clawpanel add tests/config-surface.test.js
git -c safe.directory=D:/workSpace/ohter/clawpanel commit -m "test: guard configuration ownership surfaces"
```

Expected:

```text
[codex/clawpanel-new <hash>] test: guard configuration ownership surfaces
```

## Task 8: Final Verification And Notes

**Files:**
- Modify: `docs/superpowers/plans/2026-05-14-configuration-ownership.md` only if verification notes are needed

- [ ] **Step 1: Run all JavaScript tests**

Run:

```powershell
node --test tests/*.test.js
```

Expected:

```text
# pass
```

- [ ] **Step 2: Run Rust product config tests**

Run:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
cargo test --manifest-path src-tauri/Cargo.toml product_config
```

Expected:

```text
test result: ok
```

- [ ] **Step 3: Run Rust check**

Run:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected:

```text
Finished `dev` profile
```

- [ ] **Step 4: Run frontend build**

Run:

```powershell
.\node_modules\.bin\vite.cmd build
```

Expected:

```text
built
```

- [ ] **Step 5: Verify Tauri metadata still resolves**

Run:

```powershell
.\node_modules\.bin\tauri.cmd info
```

Expected:

```text
Environment
Packages
App
```

- [ ] **Step 6: Check working tree**

Run:

```powershell
git -c safe.directory=D:/workSpace/ohter/clawpanel status --short
```

Expected:

```text
```

If generated output such as `dist/` appears and is already ignored, do not commit it.

- [ ] **Step 7: Record verification notes if a desktop smoke command is blocked**

If `.\node_modules\.bin\tauri.cmd build --debug` is attempted and fails because `npm` is unavailable in the current PowerShell PATH, append an `Execution Notes` section to this plan:

```markdown
## Execution Notes

- `node --test tests/*.test.js` passed.
- `cargo test --manifest-path src-tauri/Cargo.toml product_config` passed.
- `cargo check --manifest-path src-tauri/Cargo.toml` passed.
- `.\node_modules\.bin\vite.cmd build` passed.
- `.\node_modules\.bin\tauri.cmd info` passed.
- `.\node_modules\.bin\tauri.cmd build --debug` was not completed in this shell because Tauri invokes `npm run build` and `npm` is unavailable on PATH here.
```

Then commit the note:

```powershell
git -c safe.directory=D:/workSpace/ohter/clawpanel add docs/superpowers/plans/2026-05-14-configuration-ownership.md
git -c safe.directory=D:/workSpace/ohter/clawpanel commit -m "docs: record configuration ownership verification"
```

- [ ] **Step 8: Prepare final handoff**

Final handoff must mention:

- Product panel config now defaults to `agentdock.json`.
- Legacy `clawpanel.json` and `.openclaw` are compatibility-only.
- First-run user choice is Import or Ignore.
- OpenClaw `openclaw.json` was intentionally not renamed.
- Verification commands and outcomes.

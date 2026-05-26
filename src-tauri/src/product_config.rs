use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const PRODUCT_ID: &str = "agentdock";
pub const PRODUCT_NAME: &str = "AgentDock";
pub const PRODUCT_CONFIG_FILENAME: &str = "agentdock.json";
pub const PRODUCT_DATA_DIR_NAME: &str = ".agentdock";
pub const LEGACY_PANEL_CONFIG_FILENAME: &str = "agentdock.json";
pub const LEGACY_DATA_DIR_NAME: &str = ".openclaw";
pub const LEGACY_PRODUCT_NAME: &str = "AgentDock";
pub const UPDATE_MANIFEST_URL: &str =
    "https://raw.githubusercontent.com/dfpo520-claw-bot/AgentDock/master/update/latest.json";

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

pub fn product_name() -> &'static str {
    PRODUCT_NAME
}

pub fn update_manifest_url() -> &'static str {
    UPDATE_MANIFEST_URL
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
            let trimmed = combined.trim();
            if !trimmed.is_empty() {
                push_unique(
                    &mut paths,
                    PathBuf::from(trimmed)
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
    value
        .pointer("/agentdock/migration/decision")
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

    let already_decided = product_config.as_ref().is_some_and(has_recorded_migration);
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
        let root =
            std::env::temp_dir().join(format!("agentdock-product-config-{name}-{}", now_millis()));
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn detects_legacy_config_when_product_config_is_missing() {
        let root = temp_root("detect");
        let product = root
            .join(PRODUCT_DATA_DIR_NAME)
            .join(PRODUCT_CONFIG_FILENAME);
        let legacy_dir = root.join(LEGACY_DATA_DIR_NAME);
        fs::create_dir_all(&legacy_dir).unwrap();
        let legacy = legacy_dir.join(LEGACY_PANEL_CONFIG_FILENAME);
        fs::write(&legacy, r#"{"networkProxy":"http://127.0.0.1:7897"}"#).unwrap();

        let detected = detect_legacy_config_for_paths(product, legacy, legacy_dir);

        assert!(detected.needed);
        assert_eq!(detected.recommended_action, "import");
        assert!(detected
            .detected_items
            .contains(&"legacyPanelConfig".to_string()));
    }

    #[test]
    fn product_config_suppresses_migration_prompt() {
        let root = temp_root("product-present");
        let product_dir = root.join(PRODUCT_DATA_DIR_NAME);
        fs::create_dir_all(&product_dir).unwrap();
        let product = product_dir.join(PRODUCT_CONFIG_FILENAME);
        fs::write(
            &product,
            r#"{"agentdock":{"migration":{"decision":"ignored"}}}"#,
        )
        .unwrap();
        let legacy_dir = root.join(LEGACY_DATA_DIR_NAME);
        fs::create_dir_all(&legacy_dir).unwrap();
        let legacy = legacy_dir.join(LEGACY_PANEL_CONFIG_FILENAME);
        fs::write(&legacy, r#"{"openclawDir":"D:\\OpenClaw"}"#).unwrap();

        let detected = detect_legacy_config_for_paths(product, legacy, legacy_dir);

        assert!(!detected.needed);
    }

    #[test]
    fn import_copies_compatible_keys_and_records_metadata() {
        let root = temp_root("import");
        let product = root
            .join(PRODUCT_DATA_DIR_NAME)
            .join(PRODUCT_CONFIG_FILENAME);
        let legacy_dir = root.join(LEGACY_DATA_DIR_NAME);
        fs::create_dir_all(&legacy_dir).unwrap();
        let legacy = legacy_dir.join(LEGACY_PANEL_CONFIG_FILENAME);
        fs::write(
            &legacy,
            r#"{"networkProxy":"http://127.0.0.1:7897","openclawDir":"D:\\OpenClaw","unrelated":true}"#,
        )
        .unwrap();

        let result = apply_legacy_config_decision_for_paths(
            LegacyConfigDecision {
                action: "import".into(),
            },
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
        let product = root
            .join(PRODUCT_DATA_DIR_NAME)
            .join(PRODUCT_CONFIG_FILENAME);
        let legacy = root
            .join(LEGACY_DATA_DIR_NAME)
            .join(LEGACY_PANEL_CONFIG_FILENAME);

        let result = apply_legacy_config_decision_for_paths(
            LegacyConfigDecision {
                action: "ignore".into(),
            },
            product.clone(),
            legacy,
        )
        .unwrap();
        let written: Value = serde_json::from_str(&fs::read_to_string(product).unwrap()).unwrap();

        assert_eq!(result.action, "ignore");
        assert!(result.imported_keys.is_empty());
        assert_eq!(
            written.pointer("/agentdock/migration/decision").unwrap(),
            "ignored"
        );
        assert!(written.get("networkProxy").is_none());
    }
}

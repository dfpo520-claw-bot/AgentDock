use serde_json::{json, Value};
use std::fs;

const DEFAULT_REGISTRY: &str = "https://registry.npmmirror.com";

fn get_configured_registry() -> String {
    let path = super::openclaw_dir().join("npm-registry.txt");
    fs::read_to_string(&path)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| DEFAULT_REGISTRY.to_string())
}

pub fn get_openclaw_dir() -> Result<Value, String> {
    let resolved = super::openclaw_dir();
    let is_custom = super::read_panel_config_value()
        .and_then(|v| v.get("openclawDir")?.as_str().map(String::from))
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false);
    let config_exists = resolved.join("openclaw.json").exists();
    Ok(json!({
        "path": resolved.to_string_lossy(),
        "isCustom": is_custom,
        "configExists": config_exists,
    }))
}

pub fn read_panel_config() -> Result<Value, String> {
    let path = super::panel_config_path();
    if !path.exists() {
        return Ok(json!({}));
    }
    let content = fs::read_to_string(&path).map_err(|e| format!("读取失败: {e}"))?;
    serde_json::from_str(&content).map_err(|e| format!("解析失败: {e}"))
}

pub fn write_panel_config(config: Value) -> Result<(), String> {
    let path = super::panel_config_path();
    if let Some(dir) = path.parent() {
        if !dir.exists() {
            fs::create_dir_all(dir).map_err(|e| format!("创建目录失败: {e}"))?;
        }
    }
    let json = serde_json::to_string_pretty(&config).map_err(|e| format!("序列化失败: {e}"))?;
    fs::write(&path, json).map_err(|e| format!("写入失败: {e}"))
}

pub fn detect_legacy_config_migration() -> Result<Value, String> {
    serde_json::to_value(crate::product_config::detect_legacy_config())
        .map_err(|e| format!("serialize legacy migration detection failed: {e}"))
}

pub fn apply_legacy_config_migration(action: String) -> Result<Value, String> {
    let result = crate::product_config::apply_legacy_config_decision(
        crate::product_config::LegacyConfigDecision { action },
    )?;
    serde_json::to_value(result)
        .map_err(|e| format!("serialize legacy migration result failed: {e}"))
}

pub fn get_npm_registry() -> Result<String, String> {
    Ok(get_configured_registry())
}

pub fn set_npm_registry(registry: String) -> Result<(), String> {
    let path = super::openclaw_dir().join("npm-registry.txt");
    fs::write(&path, registry.trim()).map_err(|e| format!("保存失败: {e}"))
}

pub fn invalidate_path_cache() -> Result<(), String> {
    super::refresh_enhanced_path();
    crate::commands::service::invalidate_cli_detection_cache();
    Ok(())
}

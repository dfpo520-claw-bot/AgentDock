use serde_json::Value;

#[tauri::command]
pub fn check_installation() -> Result<Value, String> {
    let dir = super::openclaw_dir();
    let installed = dir.join("openclaw.json").exists();
    let mut result = serde_json::Map::new();
    result.insert("installed".into(), Value::Bool(installed));
    result.insert(
        "path".into(),
        Value::String(dir.to_string_lossy().to_string()),
    );
    Ok(Value::Object(result))
}

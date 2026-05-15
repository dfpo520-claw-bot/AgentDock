use serde_json::Value;

#[tauri::command]
pub async fn get_status_summary() -> Result<Value, String> {
    let output = crate::utils::openclaw_command_async()
        .args(["status", "--json"])
        .output()
        .await;

    match output {
        Ok(o) if o.status.success() => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            crate::commands::skills::extract_json_pub(&stdout)
                .ok_or_else(|| "解析失败: 未找到有效 JSON".to_string())
        }
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            Err(format!("openclaw status 失败: {}", stderr.trim()))
        }
        Err(e) => Err(format!("执行 openclaw 失败: {e}")),
    }
}

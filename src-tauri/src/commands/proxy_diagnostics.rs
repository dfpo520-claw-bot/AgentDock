use serde_json::{json, Value};

#[tauri::command]
pub async fn test_proxy(url: Option<String>) -> Result<Value, String> {
    let proxy_url = crate::commands::configured_proxy_url()
        .ok_or("未配置代理地址，请先在面板设置中保存代理地址")?;

    let target = url.unwrap_or_else(|| "https://registry.npmjs.org/-/ping".to_string());

    let client = crate::commands::build_http_client(
        std::time::Duration::from_secs(10),
        Some("ClawPanel"),
    )
    .map_err(|e| format!("创建代理客户端失败: {e}"))?;

    let start = std::time::Instant::now();
    let resp = client.get(&target).send().await.map_err(|e| {
        let elapsed = start.elapsed().as_millis();
        format!("代理连接失败 ({elapsed}ms): {e}")
    })?;

    let elapsed = start.elapsed().as_millis();
    let status = resp.status().as_u16();

    Ok(json!({
        "ok": status < 500,
        "status": status,
        "elapsed_ms": elapsed,
        "proxy": proxy_url,
        "target": target,
    }))
}

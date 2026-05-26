use serde_json::{json, Value};
#[cfg(target_os = "macos")]
use std::fs;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::process::Command;

/// Prefer HTTP reload first, then fall back to service restart if needed.
#[allow(dead_code)]
async fn reload_gateway_via_http() -> Result<String, String> {
    let config_path = super::openclaw_dir().join("openclaw.json");
    let content =
        std::fs::read_to_string(&config_path).map_err(|e| format!("读取配置失败: {e}"))?;
    let config: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("解析配置失败: {e}"))?;

    let gw_port = config
        .get("gateway")
        .and_then(|g| g.get("port"))
        .and_then(|p| p.as_u64())
        .unwrap_or(18789) as u16;

    let token = config
        .get("gateway")
        .and_then(|g| g.get("auth"))
        .and_then(|a| a.get("token"))
        .and_then(|t| t.as_str())
        .unwrap_or("");

    let control_ports = [gw_port + 2, 18792];

    for ctrl_port in control_ports {
        let url = format!("http://127.0.0.1:{}/__api/reload", ctrl_port);
        let client =
            super::build_http_client(std::time::Duration::from_secs(5), Some("AgentDock"))?;

        let mut req = client.post(&url);
        if !token.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", token));
        }

        match req.send().await {
            Ok(resp) if resp.status().is_success() => {
                return Ok("Gateway 已重载".to_string());
            }
            Ok(resp) => {
                eprintln!(
                    "[reload_gateway] control port {ctrl_port} returned {}",
                    resp.status()
                );
            }
            Err(e) => {
                eprintln!("[reload_gateway] control port {ctrl_port} request failed: {e}");
            }
        }
    }

    eprintln!("[reload_gateway] HTTP reload failed, falling back to restart");
    Err("Gateway HTTP 重载失败".to_string())
}

async fn reload_gateway_internal(app: Option<&tauri::AppHandle>) -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        let uid = crate::runtime_support::get_uid()?;
        let target = format!("gui/{uid}/ai.openclaw.gateway");
        let output = tokio::process::Command::new("launchctl")
            .args(["kickstart", "-k", &target])
            .output()
            .await
            .map_err(|e| format!("重载失败: {e}"))?;
        if output.status.success() {
            Ok("Gateway 已重载".to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("重载失败: {stderr}"))
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        match reload_gateway_via_http().await {
            Ok(msg) => Ok(msg),
            Err(_) => crate::commands::service::restart_service(
                app.cloned()
                    .ok_or_else(|| "缺少 AppHandle，无法回退为 Gateway 重启".to_string())?,
                "ai.openclaw.gateway".into(),
            )
            .await
            .map(|_| "Gateway 已重启".to_string()),
        }
    }
}

static RESTART_MUTEX: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
static LAST_RESTART_FINISHED_AT: std::sync::Mutex<Option<std::time::Instant>> =
    std::sync::Mutex::new(None);
const RESTART_COOLDOWN: std::time::Duration = std::time::Duration::from_secs(2);

async fn restart_gateway_guarded(app: Option<&tauri::AppHandle>) -> Result<String, String> {
    let _guard = RESTART_MUTEX.lock().await;

    let last_finished = {
        let guard = LAST_RESTART_FINISHED_AT.lock().unwrap();
        *guard
    };
    if let Some(last) = last_finished {
        if last.elapsed() < RESTART_COOLDOWN {
            return Ok("Gateway 刚刚处理过重启请求".to_string());
        }
    }

    let result = reload_gateway_internal(app).await;

    {
        let mut guard = LAST_RESTART_FINISHED_AT.lock().unwrap();
        *guard = Some(std::time::Instant::now());
    }

    result
}

pub async fn do_reload_gateway(app: &tauri::AppHandle) -> Result<String, String> {
    reload_gateway_internal(Some(app)).await
}

#[tauri::command]
pub async fn reload_gateway(app: tauri::AppHandle) -> Result<String, String> {
    restart_gateway_guarded(Some(&app)).await
}

#[tauri::command]
pub async fn restart_gateway(app: tauri::AppHandle) -> Result<String, String> {
    restart_gateway_guarded(Some(&app)).await
}

#[tauri::command]
pub async fn doctor_fix() -> Result<Value, String> {
    use crate::utils::openclaw_command_async;

    let result = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        openclaw_command_async().args(["doctor", "--fix"]).output(),
    )
    .await;

    match result {
        Ok(Ok(o)) => {
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            let stderr = String::from_utf8_lossy(&o.stderr).to_string();
            let success = o.status.success();
            Ok(json!({
                "success": success,
                "output": stdout.trim(),
                "errors": stderr.trim(),
                "exitCode": o.status.code(),
            }))
        }
        Ok(Err(e)) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                Err("OpenClaw CLI 未找到".to_string())
            } else {
                Err(format!("执行 doctor 失败: {e}"))
            }
        }
        Err(_) => Err("doctor --fix 执行超时 (30s)".to_string()),
    }
}

#[tauri::command]
pub async fn doctor_check() -> Result<Value, String> {
    use crate::utils::openclaw_command_async;

    let result = tokio::time::timeout(
        std::time::Duration::from_secs(20),
        openclaw_command_async().args(["doctor"]).output(),
    )
    .await;

    match result {
        Ok(Ok(o)) => {
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            let stderr = String::from_utf8_lossy(&o.stderr).to_string();
            Ok(json!({
                "success": o.status.success(),
                "output": stdout.trim(),
                "errors": stderr.trim(),
            }))
        }
        Ok(Err(e)) => Err(format!("执行 doctor 失败: {e}")),
        Err(_) => Err("doctor 执行超时 (20s)".to_string()),
    }
}

#[tauri::command]
pub async fn install_gateway() -> Result<String, String> {
    use crate::utils::openclaw_command_async;

    let _guardian_pause = crate::runtime_support::GuardianPause::new("install gateway");
    let cli_check = openclaw_command_async().arg("--version").output().await;
    match cli_check {
        Ok(o) if o.status.success() => {}
        _ => {
            return Err("openclaw CLI 未检测到，请先安装：\n\n\
                 npm install -g @DeepAi助手/openclaw-zh\n\n\
                 安装完成后再继续安装 Gateway"
                .into());
        }
    }

    let output = openclaw_command_async()
        .args(["gateway", "install"])
        .output()
        .await
        .map_err(|e| format!("安装失败: {e}"))?;

    if output.status.success() {
        Ok("Gateway 已安装".to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("安装失败: {stderr}"))
    }
}

#[tauri::command]
pub fn uninstall_gateway() -> Result<String, String> {
    let _guardian_pause = crate::runtime_support::GuardianPause::new("uninstall gateway");
    crate::commands::service::guardian_mark_manual_stop();
    #[cfg(target_os = "macos")]
    {
        let uid = crate::runtime_support::get_uid()?;
        let target = format!("gui/{uid}/ai.openclaw.gateway");

        let _ = Command::new("launchctl")
            .args(["bootout", &target])
            .output();

        let home = dirs::home_dir().unwrap_or_default();
        let plist = home.join("Library/LaunchAgents/ai.openclaw.gateway.plist");
        if plist.exists() {
            fs::remove_file(&plist).map_err(|e| format!("删除 plist 失败: {e}"))?;
        }
    }
    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("taskkill")
            .args(["/f", "/im", "node.exe", "/fi", "WINDOWTITLE eq openclaw*"])
            .creation_flags(0x08000000)
            .output();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("pkill")
            .args(["-f", "openclaw.*gateway"])
            .output();
    }
    Ok("Gateway 已卸载".to_string())
}

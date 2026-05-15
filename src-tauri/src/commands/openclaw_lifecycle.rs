use crate::utils::openclaw_command;
use std::process::Command;

#[tauri::command]
pub async fn uninstall_openclaw(
    app: tauri::AppHandle,
    clean_config: bool,
) -> Result<String, String> {
    let app2 = app.clone();
    tauri::async_runtime::spawn(async move {
        use tauri::Emitter;
        let result = uninstall_openclaw_inner(app2.clone(), clean_config).await;
        match result {
            Ok(msg) => {
                let _ = app2.emit("upgrade-done", &msg);
            }
            Err(err) => {
                let _ = app2.emit("upgrade-error", &err);
            }
        }
    });
    Ok("任务已启动".into())
}

async fn uninstall_openclaw_inner(
    app: tauri::AppHandle,
    clean_config: bool,
) -> Result<String, String> {
    use std::io::{BufRead, BufReader};
    use std::process::Stdio;
    use tauri::Emitter;
    let _guardian_pause = crate::runtime_support::GuardianPause::new("uninstall openclaw");
    crate::commands::service::guardian_mark_manual_stop();

    let source = super::openclaw_version::detect_installed_source();
    let pkg = super::openclaw_install_policy::npm_package_name(&source);

    // 1. 先停止 Gateway
    let _ = app.emit("upgrade-log", "正在停止 Gateway...");
    #[cfg(target_os = "macos")]
    {
        let uid = crate::runtime_support::get_uid().unwrap_or(501);
        let _ = Command::new("launchctl")
            .args(["bootout", &format!("gui/{uid}/ai.openclaw.gateway")])
            .output();
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = openclaw_command().args(["gateway", "stop"]).output();
    }

    // 2. 卸载 Gateway 服务
    let _ = app.emit("upgrade-log", "正在卸载 Gateway 服务...");
    #[cfg(not(target_os = "macos"))]
    {
        let _ = openclaw_command().args(["gateway", "uninstall"]).output();
    }

    // 等待进程完全退出（Gateway stop 是异步的，需要等文件锁释放）
    let _ = app.emit("upgrade-log", "等待进程退出...");
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    // 3. 清理 standalone 安装（所有可能的位置）
    for sa_dir in &crate::standalone_paths::all_standalone_dirs() {
        if sa_dir.exists() {
            let _ = app.emit(
                "upgrade-log",
                format!("清理 standalone 安装: {}", sa_dir.display()),
            );

            // Windows: 先尝试终止占用该目录的 node.exe 进程
            // 使用 PowerShell Get-Process（兼容 Windows 11，wmic 已废弃）
            #[cfg(target_os = "windows")]
            {
                let dir_lower = sa_dir
                    .to_string_lossy()
                    .to_lowercase()
                    .replace('\\', "\\\\");
                let ps_script = format!(
                    "Get-Process -Name node -ErrorAction SilentlyContinue | Where-Object {{ $_.Path -and $_.Path.ToLower().Contains('{}') }} | Select-Object -ExpandProperty Id",
                    dir_lower
                );
                if let Ok(output) = Command::new("powershell")
                    .args(["-NoProfile", "-Command", &ps_script])
                    .output()
                {
                    let text = String::from_utf8_lossy(&output.stdout);
                    for line in text.lines() {
                        if let Ok(pid) = line.trim().parse::<u32>() {
                            let _ = app.emit("upgrade-log", format!("终止占用进程 PID {pid}..."));
                            let _ = Command::new("taskkill")
                                .args(["/F", "/PID", &pid.to_string()])
                                .output();
                        }
                    }
                }
                // 短暂等待进程退出
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }

            // 尝试删除，失败则重试一次
            match std::fs::remove_dir_all(sa_dir) {
                Ok(()) => {
                    let _ = app.emit("upgrade-log", "standalone 安装已清理 ✓");
                }
                Err(_) => {
                    // 重试：等待后再删一次
                    let _ = app.emit("upgrade-log", "文件被占用，等待后重试...");
                    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                    if let Err(e) = std::fs::remove_dir_all(sa_dir) {
                        let _ = app.emit(
                            "upgrade-log",
                            format!(
                                "⚠️ 清理 standalone 失败: {e}（可手动删除 {}）",
                                sa_dir.display()
                            ),
                        );
                    } else {
                        let _ = app.emit("upgrade-log", "standalone 安装已清理（重试成功）✓");
                    }
                }
            }
        }
    }

    // 4. npm uninstall
    let _ = app.emit("upgrade-log", format!("$ npm uninstall -g {pkg}"));
    let _ = app.emit("upgrade-progress", 20);

    let mut child = super::openclaw_install_runtime::npm_command_elevated()
        .args(["uninstall", "-g", pkg])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("执行卸载命令失败: {e}"))?;

    let stderr = child.stderr.take();
    let stdout = child.stdout.take();

    let app2 = app.clone();
    let handle = std::thread::spawn(move || {
        if let Some(pipe) = stderr {
            for line in BufReader::new(pipe).lines().map_while(Result::ok) {
                let _ = app2.emit("upgrade-log", &line);
            }
        }
    });

    if let Some(pipe) = stdout {
        for line in BufReader::new(pipe).lines().map_while(Result::ok) {
            let _ = app.emit("upgrade-log", &line);
        }
    }

    let _ = handle.join();
    let _ = app.emit("upgrade-progress", 60);

    let status = child.wait().map_err(|e| format!("等待进程失败: {e}"))?;
    if !status.success() {
        let code = status
            .code()
            .map(|c| c.to_string())
            .unwrap_or("unknown".into());
        return Err(format!("卸载失败，exit code: {code}"));
    }

    // 4. 两个包都尝试卸载（确保干净）
    let other_pkg = if source == "official" {
        "@qingchencloud/openclaw-zh"
    } else {
        "openclaw"
    };
    let _ = app.emit("upgrade-log", format!("清理 {other_pkg}..."));
    let _ = super::openclaw_install_runtime::npm_command_elevated()
        .args(["uninstall", "-g", other_pkg])
        .output();
    let _ = app.emit("upgrade-progress", 80);

    // 5. 可选：清理配置目录
    if clean_config {
        let config_dir = super::openclaw_dir();
        if config_dir.exists() {
            let _ = app.emit(
                "upgrade-log",
                format!("清理配置目录: {}", config_dir.display()),
            );
            if let Err(e) = std::fs::remove_dir_all(&config_dir) {
                let _ = app.emit(
                    "upgrade-log",
                    format!("⚠️ 清理配置目录失败: {e}（可能有文件被占用）"),
                );
            }
        }
    }

    let _ = app.emit("upgrade-progress", 100);
    // #Compat-4: 卸载后刷缓存，否则 is_cli_installed（60s TTL）/ enhanced_path
    // 仍是旧快照，UI 会在 60 秒内继续显示「CLI 已安装」或 Gateway 还在运行。
    super::refresh_enhanced_path();
    crate::commands::service::invalidate_cli_detection_cache();
    let msg = if clean_config {
        "✅ OpenClaw 已完全卸载（包括配置文件）"
    } else {
        "✅ OpenClaw 已卸载（配置文件保留在 ~/.openclaw/）"
    };
    let _ = app.emit("upgrade-log", msg);
    Ok(msg.into())
}


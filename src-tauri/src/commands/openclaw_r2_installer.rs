use serde_json::Value;
use std::process::Command;

#[allow(dead_code)]
pub(crate) async fn try_r2_install(
    app: &tauri::AppHandle,
    version: &str,
    source: &str,
) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    use tauri::Emitter;

    let r2 = super::openclaw_install_policy::r2_config();
    if !r2.enabled {
        return Err("R2 加速未启用".into());
    }
    let base_url = r2.base_url.as_deref().ok_or("R2 baseUrl 未配置")?;
    let platform = super::openclaw_install_runtime::r2_platform_key();
    if platform == "unknown" {
        return Err("当前平台不支持 R2 预装归档".into());
    }

    // 1. 获取 latest.json
    let _ = app.emit("upgrade-log", "尝试从 CDN 加速下载...");
    let manifest_url = format!("{}/latest.json", base_url);
    let client = crate::commands::build_http_client(std::time::Duration::from_secs(10), None)
        .map_err(|e| format!("HTTP 客户端创建失败: {e}"))?;
    let manifest_resp = client
        .get(&manifest_url)
        .send()
        .await
        .map_err(|e| format!("获取 CDN 清单失败: {e}"))?;
    if !manifest_resp.status().is_success() {
        return Err(format!("CDN 清单不可用 (HTTP {})", manifest_resp.status()));
    }
    let manifest: Value = manifest_resp
        .json()
        .await
        .map_err(|e| format!("CDN 清单解析失败: {e}"))?;

    // 2. 查找归档：优先通用 tarball（全平台），其次平台特定 assets
    let source_key = if source == "official" {
        "official"
    } else {
        "chinese"
    };
    let source_obj = manifest.get(source_key);
    let cdn_version = source_obj
        .and_then(|s| s.get("version"))
        .and_then(|v| v.as_str())
        .unwrap_or(version);

    // 优先通用 tarball（npm pack 产物，~50MB，全平台通用）
    let tarball = source_obj.and_then(|s| s.get("tarball"));
    // 其次平台特定 assets（预装 node_modules，~200MB）
    let asset = source_obj
        .and_then(|s| s.get("assets"))
        .and_then(|a| a.get(platform));
    let use_tarball = tarball
        .and_then(|t| t.get("url"))
        .and_then(|v| v.as_str())
        .is_some();

    let (archive_url, expected_sha, expected_size) = if let Some(a) = asset {
        // 优先平台预装归档（直接解压，零网络依赖，最快）
        (
            a.get("url")
                .and_then(|v| v.as_str())
                .ok_or("归档 URL 缺失")?,
            a.get("sha256").and_then(|v| v.as_str()).unwrap_or(""),
            a.get("size").and_then(|v| v.as_u64()).unwrap_or(0),
        )
    } else if use_tarball {
        // 其次通用 tarball（需要 npm install，仍有网络依赖）
        let t = tarball.unwrap();
        (
            t.get("url")
                .and_then(|v| v.as_str())
                .ok_or("tarball URL 缺失")?,
            t.get("sha256").and_then(|v| v.as_str()).unwrap_or(""),
            t.get("size").and_then(|v| v.as_u64()).unwrap_or(0),
        )
    } else {
        return Err(format!("CDN 无 {source_key} 可用归档"));
    };

    // 版本匹配检查（如果用户指定了版本，CDN 版本必须匹配）
    if version != "latest" && !super::openclaw_install_policy::versions_match(cdn_version, version)
    {
        return Err(format!(
            "CDN 版本 {cdn_version} 与请求版本 {version} 不匹配"
        ));
    }

    let size_mb = if expected_size > 0 {
        format!("{:.0}MB", expected_size as f64 / 1_048_576.0)
    } else {
        "未知大小".into()
    };
    let _ = app.emit(
        "upgrade-log",
        format!("CDN 下载: {cdn_version} ({platform}, {size_mb})"),
    );
    let _ = app.emit("upgrade-progress", 15);

    // 3. 流式下载到临时文件
    let tmp_dir = std::env::temp_dir();
    let archive_path = tmp_dir.join(format!("openclaw-{platform}.tgz"));
    let dl_client = crate::commands::build_http_client(std::time::Duration::from_secs(300), None)
        .map_err(|e| format!("下载客户端创建失败: {e}"))?;
    let dl_resp = dl_client
        .get(archive_url)
        .send()
        .await
        .map_err(|e| format!("CDN 下载失败: {e}"))?;
    if !dl_resp.status().is_success() {
        return Err(format!("CDN 下载失败 (HTTP {})", dl_resp.status()));
    }
    let total_bytes = dl_resp.content_length().unwrap_or(expected_size);

    {
        use tokio::io::AsyncWriteExt;
        let mut file = tokio::fs::File::create(&archive_path)
            .await
            .map_err(|e| format!("创建临时文件失败: {e}"))?;
        let mut stream = dl_resp.bytes_stream();
        let mut downloaded: u64 = 0;
        let mut last_progress: u32 = 15;
        use futures_util::StreamExt;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| format!("下载中断: {e}"))?;
            file.write_all(&chunk)
                .await
                .map_err(|e| format!("写入失败: {e}"))?;
            downloaded += chunk.len() as u64;
            if total_bytes > 0 {
                let pct = 15 + ((downloaded as f64 / total_bytes as f64) * 50.0) as u32;
                if pct > last_progress {
                    last_progress = pct;
                    let _ = app.emit("upgrade-progress", pct.min(65));
                }
            }
        }
        file.flush()
            .await
            .map_err(|e| format!("刷新文件失败: {e}"))?;
    }

    let _ = app.emit("upgrade-log", "下载完成，校验中...");
    let _ = app.emit("upgrade-progress", 68);

    // 4. SHA256 校验
    if !expected_sha.is_empty() {
        let file_bytes = std::fs::read(&archive_path).map_err(|e| format!("读取归档失败: {e}"))?;
        let mut hasher = Sha256::new();
        hasher.update(&file_bytes);
        let actual_sha = format!("{:x}", hasher.finalize());
        if actual_sha != expected_sha {
            let _ = std::fs::remove_file(&archive_path);
            return Err(format!(
                "SHA256 校验失败: 期望 {expected_sha}, 实际 {actual_sha}"
            ));
        }
        let _ = app.emit("upgrade-log", "SHA256 校验通过 ✓");
    }

    let _ = app.emit("upgrade-progress", 72);

    // 5. 安装：通用 tarball 用 npm install -g，平台归档用 tar 解压
    if use_tarball {
        // 通用 tarball 模式：npm install -g ./file.tgz（全平台通用，npm 自动处理原生模块）
        let _ = app.emit("upgrade-log", "通用 tarball 模式，执行 npm install...");
        let mut install_cmd = super::openclaw_install_runtime::npm_command_elevated();
        install_cmd.args(["install", "-g", &archive_path.to_string_lossy(), "--force"]);
        super::git_runtime::apply_install_env(&mut install_cmd);
        let install_output = install_cmd
            .output()
            .map_err(|e| format!("npm install 执行失败: {e}"))?;
        if !install_output.status.success() {
            let stderr = String::from_utf8_lossy(&install_output.stderr);
            let _ = std::fs::remove_file(&archive_path);
            return Err(format!(
                "npm install -g tarball 失败: {}",
                &stderr[stderr.len().saturating_sub(300)..]
            ));
        }
        let _ = app.emit("upgrade-log", "npm install 完成 ✓");
    } else {
        // 平台特定归档模式：直接解压到 npm 全局 node_modules
        let modules_dir = super::openclaw_install_runtime::npm_global_modules_dir()
            .ok_or("无法确定 npm 全局 node_modules 目录")?;
        if !modules_dir.exists() {
            std::fs::create_dir_all(&modules_dir)
                .map_err(|e| format!("创建 node_modules 目录失败: {e}"))?;
        }
        let _ = app.emit("upgrade-log", format!("解压到 {}", modules_dir.display()));

        let qc_dir = modules_dir.join("@DeepAi助手");
        if qc_dir.exists() {
            let _ = std::fs::remove_dir_all(&qc_dir);
        }

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            let status = Command::new("tar")
                .args([
                    "-xzf",
                    &archive_path.to_string_lossy(),
                    "-C",
                    &modules_dir.to_string_lossy(),
                ])
                .creation_flags(0x08000000)
                .status()
                .map_err(|e| format!("解压失败: {e}"))?;
            if !status.success() {
                return Err("tar 解压失败".into());
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            let status = Command::new("tar")
                .args([
                    "-xzf",
                    &archive_path.to_string_lossy(),
                    "-C",
                    &modules_dir.to_string_lossy(),
                ])
                .status()
                .map_err(|e| format!("解压失败: {e}"))?;
            if !status.success() {
                return Err("tar 解压失败".into());
            }
        }

        // 归档内目录可能是 DeepAi助手/（Windows tar 不支持 @ 前缀），需要重命名
        let no_at_dir = modules_dir.join("DeepAi助手");
        if no_at_dir.exists() && !qc_dir.exists() {
            std::fs::rename(&no_at_dir, &qc_dir)
                .map_err(|e| format!("重命名 DeepAi助手 → @DeepAi助手 失败: {e}"))?;
            let _ = app.emit("upgrade-log", "目录已修正: DeepAi助手 → @DeepAi助手");
        }

        let _ = app.emit("upgrade-log", "解压完成，创建 bin 链接...");

        // 创建 bin 链接
        let bin_dir =
            super::openclaw_install_runtime::npm_global_bin_dir().ok_or("无法确定 npm bin 目录")?;
        let openclaw_js = modules_dir
            .join("@DeepAi助手")
            .join("openclaw-zh")
            .join("bin")
            .join("openclaw.js");

        if openclaw_js.exists() {
            #[cfg(target_os = "windows")]
            {
                let cmd_path = bin_dir.join("openclaw.cmd");
                let cmd_content = format!(
                    "@ECHO off\r\nGOTO start\r\n:find_dp0\r\nSET dp0=%~dp0\r\nEXIT /b\r\n:start\r\nSETLOCAL\r\nCALL :find_dp0\r\n\r\nIF EXIST \"%dp0%\\node.exe\" (\r\n  SET \"_prog=%dp0%\\node.exe\"\r\n) ELSE (\r\n  SET \"_prog=node\"\r\n  SET PATHEXT=%PATHEXT:;.JS;=;%\r\n)\r\n\r\nendLocal & goto #_undefined_# 2>NUL || title %COMSPEC% & \"%_prog%\"  \"{}\" %*\r\n",
                    openclaw_js.display()
                );
                std::fs::write(&cmd_path, cmd_content)
                    .map_err(|e| format!("创建 openclaw.cmd 失败: {e}"))?;
                let ps1_path = bin_dir.join("openclaw.ps1");
                let ps1_content = format!(
                    "#!/usr/bin/env pwsh\r\n$basedir=Split-Path $MyInvocation.MyCommand.Definition -Parent\r\n\r\n$exe=\"\"\r\nif ($PSVersionTable.PSVersion -lt \"6.0\" -or $IsWindows) {{\r\n  $exe=\".exe\"\r\n}}\r\n$ret=0\r\nif (Test-Path \"$basedir/node$exe\") {{\r\n  if ($MyInvocation.ExpectingInput) {{\r\n    $input | & \"$basedir/node$exe\"  \"{}\" $args\r\n  }} else {{\r\n    & \"$basedir/node$exe\"  \"{}\" $args\r\n  }}\r\n  $ret=$LASTEXITCODE\r\n}} else {{\r\n  if ($MyInvocation.ExpectingInput) {{\r\n    $input | & \"node$exe\"  \"{}\" $args\r\n  }} else {{\r\n    & \"node$exe\"  \"{}\" $args\r\n  }}\r\n  $ret=$LASTEXITCODE\r\n}}\r\nexit $ret\r\n",
                    openclaw_js.display(), openclaw_js.display(), openclaw_js.display(), openclaw_js.display()
                );
                let _ = std::fs::write(&ps1_path, ps1_content);
            }
            #[cfg(not(target_os = "windows"))]
            {
                let link_path = bin_dir.join("openclaw");
                let _ = std::fs::remove_file(&link_path);
                #[cfg(unix)]
                {
                    std::os::unix::fs::symlink(&openclaw_js, &link_path)
                        .map_err(|e| format!("创建 symlink 失败: {e}"))?;
                    let _ = Command::new("chmod")
                        .args(["+x", &openclaw_js.to_string_lossy()])
                        .status();
                    let _ = Command::new("chmod")
                        .args(["+x", &link_path.to_string_lossy()])
                        .status();
                }
            }
            let _ = app.emit("upgrade-log", "bin 链接已创建 ✓");
        } else {
            let _ = app.emit("upgrade-log", "⚠️ openclaw.js 未找到，bin 链接跳过");
        }
    }

    // 清理临时文件
    let _ = std::fs::remove_file(&archive_path);

    let _ = app.emit("upgrade-progress", 95);
    Ok(cdn_version.to_string())
}

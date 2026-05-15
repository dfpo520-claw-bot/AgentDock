use serde_json::Value;
#[cfg(not(target_os = "windows"))]
use std::path::PathBuf;
use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// 成功返回 Ok(版本号)，失败返回 Err(原因) 供 caller 降级到 R2/npm
pub(crate) async fn try_standalone_install(
    app: &tauri::AppHandle,
    version: &str,
    override_base_url: Option<&str>,
) -> Result<String, String> {
    let source_label = if override_base_url.is_some() {
        "GitHub"
    } else {
        "CDN"
    };
    use tauri::Emitter;

    let cfg = super::openclaw_install_policy::standalone_config();
    if !cfg.enabled {
        return Err("standalone 安装未启用".into());
    }
    let base_url = cfg.base_url.as_deref().ok_or("standalone baseUrl 未配置")?;
    let platform = super::openclaw_install_runtime::standalone_platform_key();
    if platform == "unknown" {
        return Err("当前平台不支持 standalone 安装包".into());
    }
    let install_dir = super::openclaw_install_runtime::standalone_install_dir()
        .ok_or("无法确定 standalone 安装目录")?;

    // 1. 动态查询最新版本
    let _ = app.emit(
        "upgrade-log",
        "\u{1F4E6} 尝试 standalone 独立安装包（汉化版专属，自带 Node.js 运行时，无需 npm）",
    );
    let _ = app.emit("upgrade-log", "查询最新版本...");
    let manifest_url = format!("{base_url}/latest.json");
    let client = crate::commands::build_http_client(std::time::Duration::from_secs(10), None)
        .map_err(|e| format!("HTTP 客户端创建失败: {e}"))?;
    let manifest_resp = client
        .get(&manifest_url)
        .send()
        .await
        .map_err(|e| format!("standalone 清单获取失败: {e}"))?;
    if !manifest_resp.status().is_success() {
        return Err(format!(
            "standalone 清单不可用 (HTTP {})",
            manifest_resp.status()
        ));
    }
    let manifest: Value = manifest_resp
        .json()
        .await
        .map_err(|e| format!("standalone 清单解析失败: {e}"))?;

    // 兼容两种 latest.json 格式：
    // 新格式（CI 生成）: { "editions": { "zh": { "version": "...", "base_url": "..." } } }
    // 旧格式（兼容）:   { "version": "...", "base_url": "..." }
    let edition_obj = manifest.get("editions").and_then(|e| e.get("zh"));
    let (remote_version, manifest_base_url, archive_prefix) = if let Some(ed) = edition_obj {
        let ver = ed
            .get("version")
            .and_then(|v| v.as_str())
            .ok_or("standalone 清单 editions.zh 缺少 version 字段")?;
        let bu = ed.get("base_url").and_then(|v| v.as_str());
        (ver, bu, "openclaw-zh")
    } else {
        let ver = manifest
            .get("version")
            .and_then(|v| v.as_str())
            .ok_or("standalone 清单缺少 version 字段")?;
        let bu = manifest.get("base_url").and_then(|v| v.as_str());
        (ver, bu, "openclaw")
    };

    // 版本匹配检查
    if version != "latest"
        && !super::openclaw_install_policy::versions_match(remote_version, version)
    {
        return Err(format!(
            "standalone 版本 {remote_version} 与请求版本 {version} 不匹配"
        ));
    }

    let default_base = format!("{base_url}/{remote_version}");
    let remote_base = if let Some(ovr) = override_base_url {
        ovr
    } else {
        manifest_base_url.unwrap_or(&default_base)
    };

    // 2. 构造下载 URL
    let ext = super::openclaw_install_runtime::standalone_archive_ext();
    let filename = format!("{archive_prefix}-{remote_version}-{platform}.{ext}");
    let download_url = format!("{remote_base}/{filename}");

    let _ = app.emit("upgrade-log", format!("从 {source_label} 下载: {filename}"));
    let _ = app.emit("upgrade-progress", 15);

    // 3. 流式下载
    let tmp_dir = std::env::temp_dir();
    let archive_path = tmp_dir.join(&filename);
    let dl_client = crate::commands::build_http_client(std::time::Duration::from_secs(600), None)
        .map_err(|e| format!("下载客户端创建失败: {e}"))?;
    let dl_resp = dl_client
        .get(&download_url)
        .send()
        .await
        .map_err(|e| format!("standalone 下载失败: {e}"))?;
    if !dl_resp.status().is_success() {
        return Err(format!(
            "standalone 下载失败 (HTTP {}): {download_url}",
            dl_resp.status()
        ));
    }
    let total_bytes = dl_resp.content_length().unwrap_or(0);
    let size_mb = if total_bytes > 0 {
        format!("{:.0}MB", total_bytes as f64 / 1_048_576.0)
    } else {
        "未知大小".into()
    };
    let _ = app.emit("upgrade-log", format!("下载中 ({size_mb})..."));

    {
        use futures_util::StreamExt;
        use tokio::io::AsyncWriteExt;
        let mut file = tokio::fs::File::create(&archive_path)
            .await
            .map_err(|e| format!("创建临时文件失败: {e}"))?;
        let mut stream = dl_resp.bytes_stream();
        let mut downloaded: u64 = 0;
        let mut last_progress: u32 = 15;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| format!("下载中断: {e}"))?;
            file.write_all(&chunk)
                .await
                .map_err(|e| format!("写入失败: {e}"))?;
            downloaded += chunk.len() as u64;
            if total_bytes > 0 {
                let pct = 15 + ((downloaded as f64 / total_bytes as f64) * 55.0) as u32;
                if pct > last_progress {
                    // 每 5% 输出一次文字进度
                    if pct / 5 > last_progress / 5 {
                        let dl_mb = downloaded as f64 / 1_048_576.0;
                        let total_mb = total_bytes as f64 / 1_048_576.0;
                        let real_pct = (downloaded as f64 / total_bytes as f64 * 100.0) as u32;
                        let _ = app.emit(
                            "upgrade-log",
                            format!("下载中 {real_pct}% ({dl_mb:.0}/{total_mb:.0}MB)"),
                        );
                    }
                    last_progress = pct;
                    let _ = app.emit("upgrade-progress", pct.min(70));
                }
            }
        }
        file.flush()
            .await
            .map_err(|e| format!("刷新文件失败: {e}"))?;
    }

    let _ = app.emit("upgrade-log", "下载完成，解压安装中...");
    let _ = app.emit("upgrade-progress", 72);

    // 4. 清理旧安装 & 创建目录
    if install_dir.exists() {
        let _ = std::fs::remove_dir_all(&install_dir);
    }
    std::fs::create_dir_all(&install_dir).map_err(|e| format!("创建安装目录失败: {e}"))?;

    // 5. 解压
    #[cfg(target_os = "windows")]
    {
        // Windows: zip 解压
        let archive_file =
            std::fs::File::open(&archive_path).map_err(|e| format!("打开归档失败: {e}"))?;
        let mut zip_archive =
            zip::ZipArchive::new(archive_file).map_err(|e| format!("ZIP 解析失败: {e}"))?;
        zip_archive
            .extract(&install_dir)
            .map_err(|e| format!("ZIP 解压失败: {e}"))?;
        // 归档内可能有 openclaw/ 子目录，需要提升一层
        let nested = install_dir.join("openclaw");
        if nested.exists() && nested.join("node.exe").exists() {
            for entry in std::fs::read_dir(&nested)
                .map_err(|e| format!("读取目录失败: {e}"))?
                .flatten()
            {
                let dest = install_dir.join(entry.file_name());
                let _ = std::fs::rename(entry.path(), &dest);
            }
            let _ = std::fs::remove_dir_all(&nested);
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        // Unix: tar.gz 解压
        let status = Command::new("tar")
            .args([
                "-xzf",
                &archive_path.to_string_lossy(),
                "-C",
                &install_dir.to_string_lossy(),
                "--strip-components=1",
            ])
            .status()
            .map_err(|e| format!("解压失败: {e}"))?;
        if !status.success() {
            return Err("tar 解压失败".into());
        }
    }

    // 清理临时文件
    let _ = std::fs::remove_file(&archive_path);
    let _ = app.emit("upgrade-progress", 85);

    // 6. 验证安装
    #[cfg(target_os = "windows")]
    let openclaw_bin = install_dir.join("openclaw.cmd");
    #[cfg(not(target_os = "windows"))]
    let openclaw_bin = install_dir.join("openclaw");

    if !openclaw_bin.exists() {
        return Err("standalone 解压后未找到 openclaw 可执行文件".into());
    }

    // 7. 添加到 PATH（Windows 用户 PATH，Unix 创建 symlink）
    #[cfg(target_os = "windows")]
    {
        let install_str = install_dir.to_string_lossy().to_string();
        // 检查是否已在 PATH 中
        let current_path = std::env::var("PATH").unwrap_or_default();
        if !current_path
            .split(';')
            .any(|p| p.eq_ignore_ascii_case(&install_str))
        {
            // 写入用户 PATH（注册表）
            let _ = Command::new("powershell")
                .args([
                    "-NoProfile",
                    "-Command",
                    &format!(
                        "$p = [Environment]::GetEnvironmentVariable('Path','User'); if ($p -notlike '*{}*') {{ [Environment]::SetEnvironmentVariable('Path', $p + ';{}', 'User') }}",
                        install_str.replace('\'', "''"),
                        install_str.replace('\'', "''")
                    ),
                ])
                .creation_flags(0x08000000)
                .status();
            // 同步更新当前进程的 PATH 环境变量，使后续 resolve_openclaw_cli_path()
            // 和 build_enhanced_path() 能立即发现 standalone 安装的 CLI，
            // 无需重启应用（注册表写入仅对新进程生效）
            // SAFETY: 在 Tauri 命令处理器中单次调用，此时无其他线程并发读写 PATH。
            // enhanced_path 使用独立的 RwLock 缓存，不受影响。
            unsafe {
                std::env::set_var("PATH", format!("{};{}", current_path, install_str));
            }
            let _ = app.emit("upgrade-log", format!("已添加到 PATH: {install_str}"));
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        // Unix: 创建 /usr/local/bin/openclaw symlink 或 ~/bin/openclaw
        let link_targets = [
            PathBuf::from("/usr/local/bin/openclaw"),
            dirs::home_dir()
                .unwrap_or_default()
                .join("bin")
                .join("openclaw"),
        ];
        for link in &link_targets {
            if let Some(parent) = link.parent() {
                if parent.exists() {
                    let _ = std::fs::remove_file(link);
                    #[cfg(unix)]
                    {
                        if std::os::unix::fs::symlink(&openclaw_bin, link).is_ok() {
                            let _ = Command::new("chmod")
                                .args(["+x", &openclaw_bin.to_string_lossy()])
                                .status();
                            let _ = app
                                .emit("upgrade-log", format!("symlink 已创建: {}", link.display()));
                            break;
                        }
                    }
                }
            }
        }
    }

    let _ = app.emit("upgrade-progress", 95);
    let _ = app.emit(
        "upgrade-log",
        format!("✅ standalone 独立安装包安装完成 ({remote_version})"),
    );
    let _ = app.emit(
        "upgrade-log",
        format!("安装目录: {}", install_dir.display()),
    );

    // 刷新 CLI 检测缓存
    crate::commands::service::invalidate_cli_detection_cache();

    Ok(remote_version.to_string())
}

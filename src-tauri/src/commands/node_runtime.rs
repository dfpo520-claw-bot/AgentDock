use serde_json::Value;
use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[tauri::command]
pub fn check_node() -> Result<Value, String> {
    let mut result = serde_json::Map::new();
    let enhanced = super::enhanced_path();
    let node_path = find_node_path(&enhanced);

    if let Some(path) = node_path {
        let mut cmd = Command::new(&path);
        cmd.arg("--version");
        #[cfg(target_os = "windows")]
        cmd.creation_flags(0x08000000);
        match cmd.output() {
            Ok(o) if o.status.success() => {
                let ver = String::from_utf8_lossy(&o.stdout).trim().to_string();
                let detected_from = detect_node_source(&path);
                result.insert("installed".into(), Value::Bool(true));
                result.insert("version".into(), Value::String(ver));
                result.insert("path".into(), Value::String(path));
                result.insert("detectedFrom".into(), Value::String(detected_from));
            }
            _ => {
                result.insert("installed".into(), Value::Bool(false));
                result.insert("version".into(), Value::Null);
                result.insert("path".into(), Value::Null);
                result.insert("detectedFrom".into(), Value::Null);
            }
        }
    } else {
        result.insert("installed".into(), Value::Bool(false));
        result.insert("version".into(), Value::Null);
        result.insert("path".into(), Value::Null);
        result.insert("detectedFrom".into(), Value::Null);
    }
    Ok(Value::Object(result))
}

fn find_node_path(enhanced_path: &str) -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        let mut cmd = Command::new("where");
        cmd.arg("node");
        cmd.creation_flags(0x08000000);
        if std::env::var("PATH").is_ok() {
            cmd.env("PATH", enhanced_path);
            if let Ok(output) = cmd.output() {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    if let Some(first_line) = stdout.lines().next() {
                        let path = first_line.trim().to_string();
                        if !path.is_empty() && std::path::Path::new(&path).exists() {
                            return Some(path);
                        }
                    }
                }
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let mut cmd = Command::new("which");
        cmd.arg("node");
        if let Ok(_current_path) = std::env::var("PATH") {
            cmd.env("PATH", enhanced_path);
            if let Ok(output) = cmd.output() {
                if output.status.success() {
                    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !path.is_empty() && std::path::Path::new(&path).exists() {
                        return Some(path);
                    }
                }
            }
        }
    }

    None
}

fn detect_node_source(node_path: &str) -> String {
    let path_lower = node_path.to_lowercase();
    let path_obj = std::path::Path::new(node_path);

    if let Some(parent) = path_obj.parent() {
        let parent_str = parent.to_string_lossy().to_lowercase();

        if parent_str.contains("nvm") || parent_str.contains(".nvm") {
            if let Ok(nvm_symlink) = std::env::var("NVM_SYMLINK") {
                if path_lower.contains(&nvm_symlink.to_lowercase()) {
                    return "NVM_SYMLINK".to_string();
                }
            }
            return "NVM".to_string();
        }

        if parent_str.contains(".volta") || parent_str.contains("volta") {
            return "VOLTA".to_string();
        }

        if parent_str.contains("fnm") || parent_str.contains("fnm_multishells") {
            return "FNM".to_string();
        }

        if parent_str.contains("nodenv") {
            return "NODENV".to_string();
        }

        if parent_str.contains("/n/bin") || parent_str.contains("\\n\\bin") {
            return "N".to_string();
        }

        if parent_str.contains("npm") && parent_str.contains("appdata") {
            return "NPM_GLOBAL".to_string();
        }

        if parent_str.contains("program files") || parent_str.contains("programs\\nodejs") {
            return "SYSTEM".to_string();
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(nvm_symlink) = std::env::var("NVM_SYMLINK") {
            if path_lower.contains(&nvm_symlink.to_lowercase()) {
                return "NVM_SYMLINK".to_string();
            }
        }
    }

    "PATH".to_string()
}

#[tauri::command]
pub fn check_node_at_path(node_dir: String) -> Result<Value, String> {
    let dir = std::path::PathBuf::from(&node_dir);
    #[cfg(target_os = "windows")]
    let node_bin = dir.join("node.exe");
    #[cfg(not(target_os = "windows"))]
    let node_bin = dir.join("node");

    let mut result = serde_json::Map::new();
    if !node_bin.exists() {
        result.insert("installed".into(), Value::Bool(false));
        result.insert("version".into(), Value::Null);
        return Ok(Value::Object(result));
    }

    let mut cmd = Command::new(&node_bin);
    cmd.arg("--version");
    #[cfg(target_os = "windows")]
    cmd.creation_flags(0x08000000);
    match cmd.output() {
        Ok(o) if o.status.success() => {
            let ver = String::from_utf8_lossy(&o.stdout).trim().to_string();
            result.insert("installed".into(), Value::Bool(true));
            result.insert("version".into(), Value::String(ver));
            result.insert("path".into(), Value::String(node_dir));
        }
        _ => {
            result.insert("installed".into(), Value::Bool(false));
            result.insert("version".into(), Value::Null);
        }
    }
    Ok(Value::Object(result))
}

#[tauri::command]
pub fn scan_node_paths() -> Result<Value, String> {
    let mut found: Vec<Value> = vec![];
    let home = dirs::home_dir().unwrap_or_default();

    let mut candidates: Vec<(String, String)> = vec![];

    #[cfg(target_os = "windows")]
    {
        let pf = std::env::var("ProgramFiles").unwrap_or_else(|_| r"C:\Program Files".into());
        let pf86 =
            std::env::var("ProgramFiles(x86)").unwrap_or_else(|_| r"C:\Program Files (x86)".into());
        let localappdata = std::env::var("LOCALAPPDATA").unwrap_or_default();
        let appdata = std::env::var("APPDATA").unwrap_or_default();

        if let Ok(nvm_symlink) = std::env::var("NVM_SYMLINK") {
            if std::path::Path::new(&nvm_symlink).is_dir() {
                candidates.push((nvm_symlink, "NVM_SYMLINK".to_string()));
            }
        }

        if let Ok(nvm_home) = std::env::var("NVM_HOME") {
            if std::path::Path::new(&nvm_home).is_dir() {
                if let Ok(entries) = std::fs::read_dir(&nvm_home) {
                    for entry in entries.flatten() {
                        let p = entry.path();
                        if p.is_dir() && p.join("node.exe").exists() {
                            let is_active = is_nvm_active_version(&nvm_home, &p);
                            let source = if is_active { "NVM_ACTIVE" } else { "NVM" };
                            candidates.push((p.to_string_lossy().to_string(), source.to_string()));
                        }
                    }
                }
            }
        }

        if !appdata.is_empty() {
            let nvm_dir = std::path::Path::new(&appdata).join("nvm");
            if nvm_dir.is_dir() {
                if let Ok(entries) = std::fs::read_dir(&nvm_dir) {
                    for entry in entries.flatten() {
                        let p = entry.path();
                        if p.is_dir() && p.join("node.exe").exists() {
                            let is_active =
                                is_nvm_active_version(nvm_dir.to_string_lossy().as_ref(), &p);
                            let source = if is_active { "NVM_ACTIVE" } else { "NVM" };
                            candidates.push((p.to_string_lossy().to_string(), source.to_string()));
                        }
                    }
                }
            }
        }

        let volta_bin = format!(r"{}\.volta\bin", home.display());
        candidates.push((volta_bin.clone(), "VOLTA".to_string()));
        if let Ok(volta_home) = std::env::var("VOLTA_HOME") {
            let volta_current = std::path::Path::new(&volta_home).join("current/bin");
            if volta_current.exists() {
                candidates.push((
                    volta_current.to_string_lossy().to_string(),
                    "VOLTA_ACTIVE".to_string(),
                ));
            }
        }

        if !localappdata.is_empty() {
            candidates.push((
                format!(r"{}\fnm_multishells", localappdata),
                "FNM_TEMP".to_string(),
            ));
        }
        let fnm_base = std::env::var("FNM_DIR")
            .ok()
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::Path::new(&appdata).join("fnm"));
        let fnm_current = fnm_base.join("current/installation");
        if fnm_current.is_dir() && fnm_current.join("node.exe").exists() {
            candidates.push((
                fnm_current.to_string_lossy().to_string(),
                "FNM_ACTIVE".to_string(),
            ));
        }
        let fnm_versions = fnm_base.join("node-versions");
        if fnm_versions.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&fnm_versions) {
                for entry in entries.flatten() {
                    let inst = entry.path().join("installation");
                    if inst.is_dir() && inst.join("node.exe").exists() {
                        let source = if inst == fnm_current {
                            "FNM_ACTIVE"
                        } else {
                            "FNM"
                        };
                        candidates.push((inst.to_string_lossy().to_string(), source.to_string()));
                    }
                }
            }
        }

        if !appdata.is_empty() {
            candidates.push((format!(r"{}\npm", appdata), "NPM_GLOBAL".to_string()));
        }
        if let Some(prefix) = super::windows_npm_global_prefix() {
            candidates.push((prefix, "NPM_GLOBAL".to_string()));
        }

        candidates.push((format!(r"{}\nodejs", pf), "SYSTEM".to_string()));
        candidates.push((format!(r"{}\nodejs", pf86), "SYSTEM".to_string()));
        if !localappdata.is_empty() {
            candidates.push((
                format!(r"{}\Programs\nodejs", localappdata),
                "SYSTEM".to_string(),
            ));
        }

        for drive in &["C", "D", "E", "F", "G"] {
            candidates.push((format!(r"{}:\nodejs", drive), "MANUAL".to_string()));
            candidates.push((format!(r"{}:\Node", drive), "MANUAL".to_string()));
            candidates.push((format!(r"{}:\Node.js", drive), "MANUAL".to_string()));
            candidates.push((
                format!(r"{}:\Program Files\nodejs", drive),
                "SYSTEM".to_string(),
            ));
            candidates.push((format!(r"{}:\AI\Node", drive), "MANUAL".to_string()));
            candidates.push((format!(r"{}:\AI\nodejs", drive), "MANUAL".to_string()));
            candidates.push((format!(r"{}:\Dev\nodejs", drive), "MANUAL".to_string()));
            candidates.push((format!(r"{}:\Tools\nodejs", drive), "MANUAL".to_string()));
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        candidates.push(("/usr/local/bin".into(), "SYSTEM".to_string()));
        candidates.push(("/opt/homebrew/bin".into(), "BREW".to_string()));
        candidates.push((
            format!("{}/.nvm/current/bin", home.display()),
            "NVM_ACTIVE".to_string(),
        ));
        candidates.push((
            format!("{}/.volta/bin", home.display()),
            "VOLTA".to_string(),
        ));
        candidates.push((
            format!("{}/.nodenv/shims", home.display()),
            "NODENV".to_string(),
        ));
        candidates.push((
            format!("{}/.fnm/current/bin", home.display()),
            "FNM_ACTIVE".to_string(),
        ));
        candidates.push((format!("{}/n/bin", home.display()), "N".to_string()));
        candidates.push((
            format!("{}/.npm-global/bin", home.display()),
            "NPM_GLOBAL".to_string(),
        ));
    }

    let mut seen_paths: std::collections::HashSet<String> = std::collections::HashSet::new();

    for (dir, source) in &candidates {
        let path = std::path::Path::new(dir);
        #[cfg(target_os = "windows")]
        let node_bin = path.join("node.exe");
        #[cfg(not(target_os = "windows"))]
        let node_bin = path.join("node");

        if node_bin.exists() {
            let node_path_str = node_bin.to_string_lossy().to_string();
            if seen_paths.contains(&node_path_str) {
                continue;
            }
            seen_paths.insert(node_path_str.clone());

            let mut cmd = Command::new(&node_bin);
            cmd.arg("--version");
            #[cfg(target_os = "windows")]
            cmd.creation_flags(0x08000000);
            if let Ok(o) = cmd.output() {
                if o.status.success() {
                    let ver = String::from_utf8_lossy(&o.stdout).trim().to_string();
                    let mut entry = serde_json::Map::new();
                    entry.insert("path".into(), Value::String(node_path_str));
                    entry.insert("version".into(), Value::String(ver));
                    entry.insert("source".into(), Value::String(source.clone()));
                    entry.insert("active".into(), Value::Bool(source.contains("ACTIVE")));
                    found.push(Value::Object(entry));
                }
            }
        }
    }

    found.sort_by(|a, b| {
        let a_active = a.get("active").and_then(|v| v.as_bool()).unwrap_or(false);
        let b_active = b.get("active").and_then(|v| v.as_bool()).unwrap_or(false);
        b_active.cmp(&a_active)
    });

    Ok(Value::Array(found))
}

#[allow(dead_code)]
fn is_nvm_active_version(nvm_dir: &str, version_dir: &std::path::Path) -> bool {
    let settings_path = std::path::Path::new(nvm_dir).join("settings.json");
    if !settings_path.exists() {
        return false;
    }

    if let Ok(content) = std::fs::read_to_string(&settings_path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(current_path) = json.get("path").and_then(|v| v.as_str()) {
                let expected_path: std::path::PathBuf =
                    if current_path.starts_with('/') || current_path.contains(':') {
                        std::path::Path::new(current_path).to_path_buf()
                    } else {
                        std::path::Path::new(nvm_dir).join(current_path)
                    };
                return version_dir == expected_path.as_path();
            }
        }
    }
    false
}

#[tauri::command]
pub fn save_custom_node_path(node_dir: String) -> Result<(), String> {
    let config_path = super::panel_config_path();
    if let Some(parent) = config_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let mut config: serde_json::Map<String, Value> = if config_path.exists() {
        let content =
            std::fs::read_to_string(&config_path).map_err(|e| format!("读取配置失败: {e}"))?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        serde_json::Map::new()
    };
    config.insert("nodePath".into(), Value::String(node_dir));
    let json = serde_json::to_string_pretty(&Value::Object(config))
        .map_err(|e| format!("序列化失败: {e}"))?;
    std::fs::write(&config_path, json).map_err(|e| format!("写入配置失败: {e}"))?;
    super::refresh_enhanced_path();
    crate::commands::service::invalidate_cli_detection_cache();
    Ok(())
}

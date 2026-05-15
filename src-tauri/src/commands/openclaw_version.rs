use crate::models::types::VersionInfo;
use serde_json::Value;
use std::path::PathBuf;

pub(crate) async fn get_local_version() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        if let Some(cli_path) = crate::utils::resolve_openclaw_cli_path() {
            let resolved = std::fs::canonicalize(&cli_path)
                .ok()
                .unwrap_or_else(|| PathBuf::from(&cli_path));
            if let Some(ver) =
                super::openclaw_installations::read_version_from_installation(&resolved).or_else(
                    || {
                        super::openclaw_installations::read_version_from_installation(
                            std::path::Path::new(&cli_path),
                        )
                    },
                )
            {
                return Some(ver);
            }
        }

        for brew_prefix in &["/opt/homebrew/bin", "/usr/local/bin"] {
            let openclaw_path = format!("{}/openclaw", brew_prefix);
            if let Ok(target) = std::fs::read_link(&openclaw_path) {
                let pkg_json = PathBuf::from(brew_prefix)
                    .join(&target)
                    .parent()
                    .map(|p| p.join("package.json"));
                if let Some(pkg_path) = pkg_json {
                    if let Ok(content) = std::fs::read_to_string(&pkg_path) {
                        if let Some(ver) = serde_json::from_str::<Value>(&content)
                            .ok()
                            .and_then(|v| v.get("version")?.as_str().map(String::from))
                        {
                            return Some(ver);
                        }
                    }
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Some(cli_path) = crate::utils::resolve_openclaw_cli_path() {
            let cli_pb = PathBuf::from(&cli_path);
            let resolved = std::fs::canonicalize(&cli_pb).unwrap_or_else(|_| cli_pb.clone());
            if let Some(ver) =
                super::openclaw_installations::read_version_from_installation(&resolved)
                    .or_else(|| {
                        super::openclaw_installations::read_version_from_installation(&cli_pb)
                    })
            {
                return Some(ver);
            }
        }

        for sa_dir in crate::standalone_paths::all_standalone_dirs() {
            if !sa_dir.join("openclaw.cmd").exists() {
                continue;
            }
            let version_file = sa_dir.join("VERSION");
            if let Ok(content) = std::fs::read_to_string(&version_file) {
                for line in content.lines() {
                    if let Some(ver) = line.strip_prefix("openclaw_version=") {
                        let ver = ver.trim();
                        if !ver.is_empty() {
                            return Some(ver.to_string());
                        }
                    }
                }
            }
            let sa_pkg = sa_dir
                .join("node_modules")
                .join("@qingchencloud")
                .join("openclaw-zh")
                .join("package.json");
            if let Ok(content) = std::fs::read_to_string(&sa_pkg) {
                if let Some(ver) = serde_json::from_str::<Value>(&content)
                    .ok()
                    .and_then(|v| v.get("version")?.as_str().map(String::from))
                {
                    return Some(ver);
                }
            }
        }

        if let Some(npm_bin) = super::openclaw_install_runtime::npm_global_bin_dir() {
            let shim_path = npm_bin.join("openclaw.cmd");
            if shim_path.exists() {
                let is_zh = detect_source_from_cmd_shim(&shim_path)
                    .map(|source| source == "chinese")
                    .unwrap_or(false);
                let pkgs: &[&str] = if is_zh {
                    &["@qingchencloud/openclaw-zh", "openclaw"]
                } else {
                    &["openclaw", "@qingchencloud/openclaw-zh"]
                };
                for pkg in pkgs {
                    let pkg_json = npm_bin.join("node_modules").join(pkg).join("package.json");
                    if let Ok(content) = std::fs::read_to_string(&pkg_json) {
                        if let Some(ver) = serde_json::from_str::<Value>(&content)
                            .ok()
                            .and_then(|v| v.get("version")?.as_str().map(String::from))
                        {
                            return Some(ver);
                        }
                    }
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(cli_path) = crate::utils::resolve_openclaw_cli_path() {
            let cli_pb = PathBuf::from(&cli_path);
            let resolved = std::fs::canonicalize(&cli_pb).unwrap_or_else(|_| cli_pb.clone());
            if let Some(ver) =
                super::openclaw_installations::read_version_from_installation(&resolved)
                    .or_else(|| {
                        super::openclaw_installations::read_version_from_installation(&cli_pb)
                    })
            {
                return Some(ver);
            }
        }
        for sa_dir in crate::standalone_paths::all_standalone_dirs() {
            if sa_dir.join("openclaw").exists() || sa_dir.join("VERSION").exists() {
                if let Some(ver) = super::openclaw_installations::read_version_from_installation(
                    &sa_dir.join("openclaw"),
                ) {
                    return Some(ver);
                }
            }
        }
        if let Ok(target) = std::fs::read_link("/usr/local/bin/openclaw") {
            let pkg_json = PathBuf::from("/usr/local/bin")
                .join(&target)
                .parent()
                .map(|p| p.join("package.json"));
            if let Some(ref pkg_path) = pkg_json {
                if let Ok(content) = std::fs::read_to_string(pkg_path) {
                    if let Some(ver) = serde_json::from_str::<Value>(&content)
                        .ok()
                        .and_then(|v| v.get("version")?.as_str().map(String::from))
                    {
                        return Some(ver);
                    }
                }
            }
        }
    }

    let mut status_cmd = crate::utils::openclaw_command_async();
    status_cmd.args(["status", "--json"]);
    if let Ok(Ok(output)) =
        tokio::time::timeout(std::time::Duration::from_secs(2), status_cmd.output()).await
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(ver) = crate::commands::skills::extract_json_pub(&stdout)
                .and_then(|v| v.get("runtimeVersion")?.as_str().map(String::from))
            {
                return Some(ver);
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        if let Ok(o) = std::process::Command::new("where")
            .arg("openclaw")
            .creation_flags(0x08000000)
            .output()
        {
            let stdout = String::from_utf8_lossy(&o.stdout).to_lowercase();
            let all_third_party = stdout
                .lines()
                .filter(|line| !line.trim().is_empty())
                .all(|line| line.contains(".cherrystudio") || line.contains("cherry-studio"));
            if all_third_party {
                return None;
            }
        }
    }

    let output = crate::utils::openclaw_command_async()
        .arg("--version")
        .output()
        .await
        .ok()?;
    let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
    raw.split_whitespace()
        .find(|word| word.chars().next().is_some_and(|c| c.is_ascii_digit()))
        .map(String::from)
}

async fn get_latest_version_for(source: &str) -> Option<String> {
    let client =
        crate::commands::build_http_client(std::time::Duration::from_secs(2), None).ok()?;
    let pkg = super::openclaw_install_policy::npm_package_name(source)
        .replace('/', "%2F")
        .replace('@', "%40");
    let registry = super::config::get_configured_registry();
    let url = format!("{registry}/{pkg}/latest");
    let resp = client.get(&url).send().await.ok()?;
    let json: Value = resp.json().await.ok()?;
    json.get("version")
        .and_then(|v| v.as_str())
        .map(String::from)
}

fn parse_version(value: &str) -> Vec<u32> {
    value
        .split(|c: char| !c.is_ascii_digit())
        .filter_map(|segment| segment.parse().ok())
        .collect()
}

#[tauri::command]
pub async fn list_openclaw_versions(source: String) -> Result<Vec<String>, String> {
    let client = crate::commands::build_http_client(std::time::Duration::from_secs(10), None)
        .map_err(|err| format!("HTTP 初始化失败: {err}"))?;
    let pkg = super::openclaw_install_policy::npm_package_name(&source).replace('/', "%2F");
    let registry = super::config::get_configured_registry();
    let url = format!("{registry}/{pkg}");
    let resp = client
        .get(&url)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|err| format!("查询版本失败: {err}"))?;
    let json: Value = resp
        .json()
        .await
        .map_err(|err| format!("解析响应失败: {err}"))?;
    let mut versions = json
        .get("versions")
        .and_then(|value| value.as_object())
        .map(|object| {
            let mut versions: Vec<String> = object.keys().cloned().collect();
            versions.sort_by(|left, right| {
                let left_parts = parse_version(left);
                let right_parts = parse_version(right);
                right_parts.cmp(&left_parts)
            });
            versions
        })
        .unwrap_or_default();
    if let Some(recommended) = super::openclaw_install_policy::recommended_version_for(&source) {
        if let Some(pos) = versions.iter().position(|version| version == &recommended) {
            let version = versions.remove(pos);
            versions.insert(0, version);
        } else {
            versions.insert(0, recommended);
        }
    }
    Ok(versions)
}

#[cfg(target_os = "windows")]
fn detect_source_from_cmd_shim(cmd_path: &std::path::Path) -> Option<String> {
    let content = std::fs::read_to_string(cmd_path).ok()?;
    let lower = content.to_lowercase();
    if lower.contains("openclaw-zh") || lower.contains("@qingchencloud") {
        return Some("chinese".into());
    }
    if lower.contains("node_modules") {
        return Some("official".into());
    }
    None
}

fn detect_standalone_source_from_dir(dir: &std::path::Path) -> Option<String> {
    let version_file = dir.join("VERSION");
    if let Ok(content) = std::fs::read_to_string(&version_file) {
        let mut edition = String::new();
        let mut package = String::new();
        for line in content.lines() {
            if let Some(value) = line.strip_prefix("edition=") {
                edition = value.trim().to_ascii_lowercase();
            } else if let Some(value) = line.strip_prefix("package=") {
                package = value.trim().to_ascii_lowercase();
            }
        }
        if package.contains("openclaw-zh") || package.contains("@qingchencloud") {
            return Some("chinese".into());
        }
        if package == "openclaw" {
            return Some("official".into());
        }
        if matches!(edition.as_str(), "zh" | "zh-cn" | "chinese" | "cn") {
            return Some("chinese".into());
        }
        if matches!(edition.as_str(), "en" | "official") {
            return Some("official".into());
        }
    }
    if dir
        .join("node_modules")
        .join("@qingchencloud")
        .join("openclaw-zh")
        .join("package.json")
        .exists()
    {
        return Some("chinese".into());
    }
    if dir
        .join("node_modules")
        .join("openclaw")
        .join("package.json")
        .exists()
    {
        return Some("official".into());
    }
    None
}

fn detect_standalone_source_from_cli_path(cli_path: &std::path::Path) -> Option<String> {
    cli_path
        .parent()
        .and_then(detect_standalone_source_from_dir)
}

pub(crate) fn detect_installed_source() -> String {
    #[cfg(target_os = "macos")]
    {
        if let Some(cli_path) = crate::utils::resolve_openclaw_cli_path() {
            let resolved = std::fs::canonicalize(&cli_path)
                .ok()
                .unwrap_or_else(|| PathBuf::from(&cli_path));
            let source = crate::utils::classify_cli_source(&resolved.to_string_lossy());
            if source == "standalone" {
                return detect_standalone_source_from_cli_path(&resolved)
                    .unwrap_or_else(|| "chinese".into());
            }
            if source == "npm-zh" {
                return "chinese".into();
            }
            if source == "npm-official" || source == "npm-global" {
                return "official".into();
            }
        }
        for brew_prefix in &["/opt/homebrew/bin/openclaw", "/usr/local/bin/openclaw"] {
            if let Ok(target) = std::fs::read_link(brew_prefix) {
                if target.to_string_lossy().contains("openclaw-zh") {
                    return "chinese".into();
                }
                return "official".into();
            }
        }
        for sa_dir in crate::standalone_paths::all_standalone_dirs() {
            if sa_dir.join("openclaw").exists() || sa_dir.join("VERSION").exists() {
                return detect_standalone_source_from_dir(&sa_dir)
                    .unwrap_or_else(|| "chinese".into());
            }
        }
        "unknown".into()
    }

    #[cfg(target_os = "windows")]
    {
        if let Some(cli_path) = crate::utils::resolve_openclaw_cli_path() {
            let source = crate::utils::classify_cli_source(&cli_path);
            if source == "standalone" {
                return detect_standalone_source_from_cli_path(std::path::Path::new(&cli_path))
                    .unwrap_or_else(|| "chinese".into());
            }
            if source == "npm-zh" {
                return "chinese".into();
            }
            if let Some(shim_source) = detect_source_from_cmd_shim(std::path::Path::new(&cli_path))
            {
                return shim_source;
            }
        }
        if let Some(npm_bin) = super::openclaw_install_runtime::npm_global_bin_dir() {
            let shim = npm_bin.join("openclaw.cmd");
            if let Some(source) = detect_source_from_cmd_shim(&shim) {
                return source;
            }
        }
        for sa_dir in crate::standalone_paths::all_standalone_dirs() {
            if sa_dir.join("openclaw.cmd").exists() || sa_dir.join("VERSION").exists() {
                return detect_standalone_source_from_dir(&sa_dir)
                    .unwrap_or_else(|| "chinese".into());
            }
        }
        "unknown".into()
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        if let Some(cli_path) = crate::utils::resolve_openclaw_cli_path() {
            let resolved = std::fs::canonicalize(&cli_path)
                .ok()
                .unwrap_or_else(|| PathBuf::from(&cli_path));
            let source = crate::utils::classify_cli_source(&resolved.to_string_lossy());
            if source == "standalone" {
                return detect_standalone_source_from_cli_path(&resolved)
                    .unwrap_or_else(|| "chinese".into());
            }
            if source == "npm-zh" {
                return "chinese".into();
            }
            if source == "npm-official" || source == "npm-global" {
                return "official".into();
            }
        }
        let home = dirs::home_dir().unwrap_or_default();
        for link in &[
            PathBuf::from("/usr/local/bin/openclaw"),
            home.join("bin").join("openclaw"),
        ] {
            if let Ok(target) = std::fs::read_link(link) {
                if target.to_string_lossy().contains("openclaw-zh") {
                    return "chinese".into();
                }
                return "official".into();
            }
        }
        for sa_dir in crate::standalone_paths::all_standalone_dirs() {
            if sa_dir.join("openclaw").exists() || sa_dir.join("VERSION").exists() {
                return detect_standalone_source_from_dir(&sa_dir)
                    .unwrap_or_else(|| "chinese".into());
            }
        }
        if let Ok(output) = super::openclaw_install_runtime::npm_command()
            .args(["list", "-g", "@qingchencloud/openclaw-zh", "--depth=0"])
            .output()
        {
            if String::from_utf8_lossy(&output.stdout).contains("openclaw-zh@") {
                return "chinese".into();
            }
        }
        "unknown".into()
    }
}

#[tauri::command]
pub async fn get_version_info() -> Result<VersionInfo, String> {
    let current = get_local_version().await;
    let mut source = detect_installed_source();
    if let Some(ref ver) = current {
        if ver.contains("-zh") && source != "chinese" {
            source = "chinese".to_string();
        }
    }

    let latest = if source == "unknown" {
        None
    } else {
        get_latest_version_for(&source).await
    };
    let recommended = if source == "unknown" {
        None
    } else {
        super::openclaw_install_policy::recommended_version_for(&source)
    };
    let update_available = match (&current, &recommended) {
        (Some(current), Some(recommended)) => {
            super::openclaw_install_policy::recommended_is_newer(recommended, current)
        }
        (None, Some(_)) => true,
        _ => false,
    };
    let latest_update_available = match (&current, &latest) {
        (Some(current), Some(latest)) => super::openclaw_install_policy::recommended_is_newer(latest, current),
        (None, Some(_)) => true,
        _ => false,
    };
    let is_recommended = match (&current, &recommended) {
        (Some(current), Some(recommended)) => {
            super::openclaw_install_policy::versions_match(current, recommended)
        }
        _ => false,
    };
    let ahead_of_recommended = match (&current, &recommended) {
        (Some(current), Some(recommended)) => {
            super::openclaw_install_policy::recommended_is_newer(current, recommended)
        }
        _ => false,
    };

    let cli_path = crate::utils::resolve_openclaw_cli_path();
    let cli_source = cli_path
        .as_ref()
        .map(|path| crate::utils::classify_cli_source(path));
    let all_installations = super::openclaw_installations::scan_all_installations(&cli_path);

    Ok(VersionInfo {
        current,
        latest,
        recommended,
        update_available,
        latest_update_available,
        is_recommended,
        ahead_of_recommended,
        panel_version: super::openclaw_install_policy::panel_version().to_string(),
        source,
        cli_path,
        cli_source,
        all_installations: Some(all_installations),
    })
}

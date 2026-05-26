use serde_json::Value;
use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

fn scan_cli_identity(cli_path: &std::path::Path) -> String {
    #[cfg(target_os = "windows")]
    let identity_path = {
        let mut identity_path = cli_path.to_path_buf();
        let file_name = cli_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        if matches!(
            file_name.as_str(),
            "openclaw" | "openclaw.exe" | "openclaw.ps1"
        ) {
            let cmd_path = cli_path.with_file_name("openclaw.cmd");
            if cmd_path.exists() {
                identity_path = cmd_path;
            }
        }
        identity_path
    };

    #[cfg(not(target_os = "windows"))]
    let identity_path = cli_path.to_path_buf();

    identity_path
        .canonicalize()
        .unwrap_or(identity_path)
        .to_string_lossy()
        .to_lowercase()
}

pub(crate) fn scan_all_installations(
    active_path: &Option<String>,
) -> Vec<crate::models::types::OpenClawInstallation> {
    use crate::models::types::OpenClawInstallation;

    let mut results: Vec<OpenClawInstallation> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let active_identity = active_path
        .as_ref()
        .map(|path| scan_cli_identity(std::path::Path::new(path)));

    let mut try_add = |path: std::path::PathBuf| {
        if !path.exists() {
            return;
        }
        if crate::openclaw_cli_paths::is_rejected_cli_path(&path.to_string_lossy()) {
            return;
        }
        let identity = scan_cli_identity(&path);
        if seen.contains(&identity) {
            return;
        }
        seen.insert(identity.clone());
        let path_str = path.to_string_lossy().to_string();
        let source = crate::utils::classify_cli_source(&path_str);
        let version = read_version_from_installation(&path);
        let is_active = active_identity
            .as_ref()
            .map(|active| active == &identity)
            .unwrap_or(false);
        results.push(OpenClawInstallation {
            path: path_str,
            source,
            version,
            active: is_active,
        });
    };

    for sa_dir in crate::standalone_paths::all_standalone_dirs() {
        #[cfg(target_os = "windows")]
        {
            try_add(sa_dir.join("openclaw.cmd"));
            try_add(sa_dir.join("openclaw.exe"));
        }
        #[cfg(not(target_os = "windows"))]
        {
            try_add(sa_dir.join("openclaw"));
        }
    }

    for configured in super::openclaw_search_paths() {
        if let Some(resolved) =
            crate::openclaw_cli_paths::resolve_openclaw_cli_input_path(&configured)
        {
            try_add(resolved);
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            try_add(
                std::path::PathBuf::from(&appdata)
                    .join("npm")
                    .join("openclaw.cmd"),
            );
            try_add(
                std::path::PathBuf::from(&appdata)
                    .join("npm")
                    .join("openclaw"),
            );
        }
        if let Some(prefix) = super::windows_npm_global_prefix() {
            let prefix_path = std::path::PathBuf::from(prefix);
            try_add(prefix_path.join("openclaw.cmd"));
            try_add(prefix_path.join("openclaw.exe"));
            try_add(prefix_path.join("openclaw"));
        }
        if let Ok(localappdata) = std::env::var("LOCALAPPDATA") {
            try_add(
                std::path::PathBuf::from(&localappdata)
                    .join("Programs")
                    .join("nodejs")
                    .join("openclaw.cmd"),
            );
        }
        if let Ok(program_files) = std::env::var("ProgramFiles") {
            try_add(
                std::path::PathBuf::from(&program_files)
                    .join("nodejs")
                    .join("openclaw.cmd"),
            );
            try_add(
                std::path::PathBuf::from(&program_files)
                    .join("OpenClaw")
                    .join("openclaw.cmd"),
            );
        }
        if let Ok(program_files_x86) = std::env::var("ProgramFiles(x86)") {
            try_add(
                std::path::PathBuf::from(&program_files_x86)
                    .join("nodejs")
                    .join("openclaw.cmd"),
            );
        }
        if let Ok(profile) = std::env::var("USERPROFILE") {
            try_add(
                std::path::PathBuf::from(&profile)
                    .join(".openclaw-bin")
                    .join("openclaw.cmd"),
            );
        }
        for drive in ["C", "D", "E", "F", "G"] {
            try_add(std::path::PathBuf::from(format!(
                r"{}:\OpenClaw\openclaw.cmd",
                drive
            )));
            try_add(std::path::PathBuf::from(format!(
                r"{}:\AI\OpenClaw\openclaw.cmd",
                drive
            )));
        }
        let mut where_cmd = Command::new("where");
        where_cmd.arg("openclaw");
        where_cmd.creation_flags(0x08000000);
        if let Ok(output) = where_cmd.output() {
            if output.status.success() {
                for line in String::from_utf8_lossy(&output.stdout).lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        try_add(std::path::PathBuf::from(trimmed));
                    }
                }
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Some(home) = dirs::home_dir() {
            try_add(home.join(".npm-global").join("bin").join("openclaw"));
            try_add(home.join(".local").join("bin").join("openclaw"));
            try_add(
                home.join(".nvm")
                    .join("current")
                    .join("bin")
                    .join("openclaw"),
            );
            try_add(home.join(".volta").join("bin").join("openclaw"));
            try_add(
                home.join(".fnm")
                    .join("current")
                    .join("bin")
                    .join("openclaw"),
            );
            try_add(home.join("bin").join("openclaw"));
        }
        try_add(std::path::PathBuf::from("/opt/openclaw/openclaw"));
        try_add(std::path::PathBuf::from("/opt/homebrew/bin/openclaw"));
        try_add(std::path::PathBuf::from("/usr/local/bin/openclaw"));
        try_add(std::path::PathBuf::from("/usr/bin/openclaw"));
        try_add(std::path::PathBuf::from("/snap/bin/openclaw"));
        if let Ok(output) = Command::new("which").args(["-a", "openclaw"]).output() {
            if output.status.success() {
                for line in String::from_utf8_lossy(&output.stdout).lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        try_add(std::path::PathBuf::from(trimmed));
                    }
                }
            }
        }
    }

    let enhanced = super::enhanced_path();
    #[cfg(target_os = "windows")]
    let sep = ';';
    #[cfg(not(target_os = "windows"))]
    let sep = ':';
    for dir in enhanced.split(sep) {
        let dir = dir.trim();
        if dir.is_empty() {
            continue;
        }
        let base = std::path::Path::new(dir);
        #[cfg(target_os = "windows")]
        {
            try_add(base.join("openclaw.cmd"));
        }
        #[cfg(not(target_os = "windows"))]
        {
            try_add(base.join("openclaw"));
        }
    }

    results.sort_by(|a, b| {
        b.active
            .cmp(&a.active)
            .then_with(|| a.source.cmp(&b.source))
            .then_with(|| a.path.cmp(&b.path))
    });

    results
}

#[tauri::command]
pub fn scan_openclaw_paths() -> Result<Vec<crate::models::types::OpenClawInstallation>, String> {
    super::refresh_enhanced_path();
    crate::commands::service::invalidate_cli_detection_cache();
    let active_path = crate::utils::resolve_openclaw_cli_path();
    Ok(scan_all_installations(&active_path))
}

#[tauri::command]
pub fn check_openclaw_at_path(cli_path: String) -> Result<Value, String> {
    let mut result = serde_json::Map::new();
    if let Some(resolved) = crate::openclaw_cli_paths::resolve_openclaw_cli_input(&cli_path) {
        let path_str = resolved.to_string_lossy().to_string();
        result.insert("installed".into(), Value::Bool(true));
        result.insert("path".into(), Value::String(path_str.clone()));
        result.insert(
            "source".into(),
            Value::String(crate::utils::classify_cli_source(&path_str)),
        );
        if let Some(version) = read_version_from_installation(&resolved) {
            result.insert("version".into(), Value::String(version));
        } else {
            result.insert("version".into(), Value::Null);
        }
    } else {
        result.insert("installed".into(), Value::Bool(false));
        result.insert("path".into(), Value::Null);
        result.insert("source".into(), Value::Null);
        result.insert("version".into(), Value::Null);
    }
    Ok(Value::Object(result))
}

pub(crate) fn read_version_from_installation(cli_path: &std::path::Path) -> Option<String> {
    if let Some(dir) = cli_path.parent() {
        let version_file = dir.join("VERSION");
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

        let own_pkg = dir.join("package.json");
        if let Ok(content) = std::fs::read_to_string(&own_pkg) {
            if let Some(ver) = serde_json::from_str::<serde_json::Value>(&content)
                .ok()
                .and_then(|v| v.get("version")?.as_str().map(String::from))
            {
                return Some(ver);
            }
        }

        let cli_source = crate::utils::classify_cli_source(&cli_path.to_string_lossy());
        let pkg_names: &[&str] = if cli_source == "npm-zh" || cli_source == "standalone" {
            &["@DeepAi助手/openclaw-zh", "openclaw"]
        } else {
            &["openclaw", "@DeepAi助手/openclaw-zh"]
        };

        for pkg_name in pkg_names {
            let pkg_json = dir.join("node_modules").join(pkg_name).join("package.json");
            if let Ok(content) = std::fs::read_to_string(&pkg_json) {
                if let Some(ver) = serde_json::from_str::<serde_json::Value>(&content)
                    .ok()
                    .and_then(|v| v.get("version")?.as_str().map(String::from))
                {
                    return Some(ver);
                }
            }
        }

        if let Some(parent) = dir.parent() {
            for pkg_name in pkg_names {
                let pkg_json = parent
                    .join("node_modules")
                    .join(pkg_name)
                    .join("package.json");
                if let Ok(content) = std::fs::read_to_string(&pkg_json) {
                    if let Some(ver) = serde_json::from_str::<serde_json::Value>(&content)
                        .ok()
                        .and_then(|v| v.get("version")?.as_str().map(String::from))
                    {
                        return Some(ver);
                    }
                }
            }
        }
    }
    None
}

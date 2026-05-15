use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

const GIT_HTTPS_REWRITES: &[(&str, &str)] = &[
    ("https://github.com/", "ssh://git@github.com/"),
    ("https://github.com/", "ssh://git@github.com"),
    ("https://github.com/", "ssh://git@://github.com/"),
    ("https://github.com/", "git@github.com:"),
    ("https://github.com/", "git://github.com/"),
    ("https://github.com/", "git+ssh://git@github.com/"),
    ("https://gitlab.com/", "ssh://git@gitlab.com/"),
    ("https://gitlab.com/", "git@gitlab.com:"),
    ("https://gitlab.com/", "git://gitlab.com/"),
    ("https://gitlab.com/", "git+ssh://git@gitlab.com/"),
    ("https://bitbucket.org/", "ssh://git@bitbucket.org/"),
    ("https://bitbucket.org/", "git@bitbucket.org:"),
    ("https://bitbucket.org/", "git://bitbucket.org/"),
    ("https://bitbucket.org/", "git+ssh://git@bitbucket.org/"),
];

fn configured_git_path() -> Option<String> {
    super::read_panel_config_value()
        .and_then(|v| v.get("gitPath")?.as_str().map(String::from))
        .map(|custom| custom.trim().to_string())
        .filter(|custom| !custom.is_empty())
}

fn executable() -> String {
    configured_git_path().unwrap_or_else(|| "git".into())
}

pub(crate) fn https_rewrite_rule_count() -> usize {
    GIT_HTTPS_REWRITES.len()
}

pub(crate) fn ensure_https_rewrites() -> usize {
    let git = executable();
    let targets: std::collections::HashSet<&str> =
        GIT_HTTPS_REWRITES.iter().map(|(target, _)| *target).collect();

    for target in &targets {
        let key = format!("url.{target}.insteadOf");
        let mut unset = Command::new(&git);
        unset.args(["config", "--global", "--unset-all", &key]);
        #[cfg(target_os = "windows")]
        unset.creation_flags(0x08000000);
        let _ = unset.output();
    }

    let mut success = 0;
    for (target, from) in GIT_HTTPS_REWRITES {
        let key = format!("url.{target}.insteadOf");
        let mut cmd = Command::new(&git);
        cmd.args(["config", "--global", "--add", &key, from]);
        #[cfg(target_os = "windows")]
        cmd.creation_flags(0x08000000);
        if cmd.output().map(|o| o.status.success()).unwrap_or(false) {
            success += 1;
        }
    }
    success
}

pub(crate) fn apply_install_env(cmd: &mut Command) {
    if let Some(custom_git) = configured_git_path() {
        let git_path = PathBuf::from(&custom_git);
        if let Some(parent) = git_path.parent() {
            let mut paths: Vec<PathBuf> = std::env::var_os("PATH")
                .map(|value| std::env::split_paths(&value).collect())
                .unwrap_or_default();
            if !paths.iter().any(|p| p == parent) {
                paths.insert(0, parent.to_path_buf());
            }
            if let Ok(joined) = std::env::join_paths(paths) {
                cmd.env("PATH", joined);
            }
        }
        cmd.env("GIT", &custom_git);
    }

    crate::commands::apply_proxy_env(cmd);
    cmd.env("GIT_TERMINAL_PROMPT", "0")
        .env(
            "GIT_SSH_COMMAND",
            "ssh -o BatchMode=yes -o StrictHostKeyChecking=no -o IdentitiesOnly=yes",
        )
        .env("GIT_ALLOW_PROTOCOL", "https:http:file");

    cmd.env("GIT_CONFIG_COUNT", https_rewrite_rule_count().to_string());
    for (idx, (target, from)) in GIT_HTTPS_REWRITES.iter().enumerate() {
        cmd.env(
            format!("GIT_CONFIG_KEY_{idx}"),
            format!("url.{target}.insteadOf"),
        )
        .env(format!("GIT_CONFIG_VALUE_{idx}"), *from);
    }
}

fn find_git_path() -> Option<String> {
    let enhanced = super::enhanced_path();

    #[cfg(target_os = "windows")]
    {
        let mut cmd = Command::new("where");
        cmd.arg("git");
        cmd.creation_flags(0x08000000);
        cmd.env("PATH", &enhanced);
        if let Ok(output) = cmd.output() {
            if output.status.success() {
                if let Some(first_line) = String::from_utf8_lossy(&output.stdout).lines().next() {
                    let path = first_line.trim().to_string();
                    if !path.is_empty() && std::path::Path::new(&path).exists() {
                        return Some(path);
                    }
                }
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let mut cmd = Command::new("which");
        cmd.arg("git");
        cmd.env("PATH", &enhanced);
        if let Ok(output) = cmd.output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() && std::path::Path::new(&path).exists() {
                    return Some(path);
                }
            }
        }
    }

    None
}

#[tauri::command]
pub fn check_git() -> Result<Value, String> {
    let mut result = serde_json::Map::new();
    let configured = configured_git_path();
    let git = configured.clone().unwrap_or_else(|| "git".into());
    let is_custom = configured.is_some();
    let git_path = if is_custom {
        Some(git.clone())
    } else {
        find_git_path()
    };

    let exec = git_path.as_deref().unwrap_or(&git);
    let mut cmd = Command::new(exec);
    cmd.arg("--version");
    cmd.env("PATH", super::enhanced_path());
    #[cfg(target_os = "windows")]
    cmd.creation_flags(0x08000000);

    match cmd.output() {
        Ok(o) if o.status.success() => {
            let ver = String::from_utf8_lossy(&o.stdout).trim().to_string();
            result.insert("installed".into(), Value::Bool(true));
            result.insert("version".into(), Value::String(ver));
            result.insert(
                "path".into(),
                git_path.map(Value::String).unwrap_or(Value::Null),
            );
            result.insert("isCustom".into(), Value::Bool(is_custom));
        }
        _ => {
            result.insert("installed".into(), Value::Bool(false));
            result.insert("version".into(), Value::Null);
            result.insert("path".into(), Value::Null);
            result.insert("isCustom".into(), Value::Bool(is_custom));
        }
    }

    Ok(Value::Object(result))
}

#[tauri::command]
pub fn scan_git_paths() -> Result<Value, String> {
    let mut found: Vec<Value> = vec![];
    let mut candidates: Vec<(String, String)> = vec![];

    #[cfg(target_os = "windows")]
    {
        let pf = std::env::var("ProgramFiles").unwrap_or_else(|_| r"C:\Program Files".into());
        let pf86 =
            std::env::var("ProgramFiles(x86)").unwrap_or_else(|_| r"C:\Program Files (x86)".into());
        let localappdata = std::env::var("LOCALAPPDATA").unwrap_or_default();

        candidates.push((format!(r"{}\Git\cmd\git.exe", pf), "SYSTEM".into()));
        candidates.push((format!(r"{}\Git\cmd\git.exe", pf86), "SYSTEM".into()));

        for drive in &["C", "D", "E", "F", "G"] {
            candidates.push((format!(r"{}:\Git\cmd\git.exe", drive), "MANUAL".into()));
            candidates.push((
                format!(r"{}:\Program Files\Git\cmd\git.exe", drive),
                "SYSTEM".into(),
            ));
            for sub in &["Tools", "Dev", "AI", "Apps", "Software"] {
                candidates.push((
                    format!(r"{}:\{}\Git\cmd\git.exe", drive, sub),
                    "MANUAL".into(),
                ));
            }
        }

        for drive in &["C", "D", "E", "F"] {
            candidates.push((
                format!(r"{}:\Data\exeApp\Git\cmd\git.exe", drive),
                "MANUAL".into(),
            ));
        }

        if !localappdata.is_empty() {
            let gh_dir = std::path::Path::new(&localappdata).join("GitHubDesktop");
            if gh_dir.is_dir() {
                if let Ok(entries) = std::fs::read_dir(&gh_dir) {
                    for entry in entries.flatten() {
                        let p = entry.path();
                        if p.is_dir() {
                            let git_exe = p
                                .join("resources")
                                .join("app")
                                .join("git")
                                .join("cmd")
                                .join("git.exe");
                            if git_exe.exists() {
                                candidates.push((
                                    git_exe.to_string_lossy().to_string(),
                                    "GITHUB_DESKTOP".into(),
                                ));
                            }
                        }
                    }
                }
            }
        }

        if !localappdata.is_empty() {
            let vscode_git = std::path::Path::new(&localappdata).join(
                r"Programs\Microsoft VS Code\resources\app\node_modules.asar.unpacked\vscode-git\git\cmd\git.exe",
            );
            if vscode_git.exists() {
                candidates.push((vscode_git.to_string_lossy().to_string(), "VSCODE".into()));
            }
        }

        candidates.push((format!(r"{}\Git\mingw64\bin\git.exe", pf), "MINGW".into()));
        for drive in &["C", "D"] {
            candidates.push((format!(r"{}:\msys64\usr\bin\git.exe", drive), "MSYS2".into()));
            candidates.push((format!(r"{}:\msys2\usr\bin\git.exe", drive), "MSYS2".into()));
        }

        let home = dirs::home_dir().unwrap_or_default();
        candidates.push((
            format!(r"{}\scoop\apps\git\current\cmd\git.exe", home.display()),
            "SCOOP".into(),
        ));
        candidates.push((
            format!(r"{}\scoop\shims\git.exe", home.display()),
            "SCOOP".into(),
        ));

        let choco_dir = std::env::var("ChocolateyInstall")
            .unwrap_or_else(|_| r"C:\ProgramData\chocolatey".into());
        candidates.push((format!(r"{}\bin\git.exe", choco_dir), "CHOCOLATEY".into()));
    }

    #[cfg(not(target_os = "windows"))]
    {
        candidates.push(("/usr/bin/git".into(), "SYSTEM".into()));
        candidates.push(("/usr/local/bin/git".into(), "SYSTEM".into()));
        candidates.push(("/opt/homebrew/bin/git".into(), "BREW".into()));
        candidates.push((
            "/Library/Developer/CommandLineTools/usr/bin/git".into(),
            "XCODE_CLT".into(),
        ));
        candidates.push((
            "/Applications/Xcode.app/Contents/Developer/usr/bin/git".into(),
            "XCODE".into(),
        ));
        candidates.push(("/snap/bin/git".into(), "SNAP".into()));
        let home = dirs::home_dir().unwrap_or_default();
        candidates.push((format!("{}/.nix-profile/bin/git", home.display()), "NIX".into()));
        candidates.push((format!("{}/.linuxbrew/bin/git", home.display()), "BREW".into()));
        candidates.push(("/home/linuxbrew/.linuxbrew/bin/git".into(), "BREW".into()));
    }

    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    for (path, source) in &candidates {
        let p = std::path::Path::new(path);
        if !p.exists() {
            continue;
        }
        let canonical = p.to_string_lossy().to_string();
        if seen.contains(&canonical) {
            continue;
        }
        seen.insert(canonical.clone());

        let mut cmd = Command::new(path);
        cmd.arg("--version");
        #[cfg(target_os = "windows")]
        cmd.creation_flags(0x08000000);
        if let Ok(o) = cmd.output() {
            if o.status.success() {
                let ver = String::from_utf8_lossy(&o.stdout).trim().to_string();
                let mut entry = serde_json::Map::new();
                entry.insert("path".into(), Value::String(canonical));
                entry.insert("version".into(), Value::String(ver));
                entry.insert("source".into(), Value::String(source.clone()));
                found.push(Value::Object(entry));
            }
        }
    }

    Ok(Value::Array(found))
}

#[tauri::command]
pub async fn auto_install_git(app: tauri::AppHandle) -> Result<String, String> {
    use std::process::Stdio;
    use tauri::Emitter;

    let _ = app.emit("upgrade-log", "正在尝试自动安装 Git...");

    #[cfg(target_os = "windows")]
    {
        use std::io::{BufRead, BufReader};

        let _ = app.emit("upgrade-log", "尝试使用 winget 安装 Git...");
        let mut child = Command::new("winget")
            .args([
                "install",
                "--id",
                "Git.Git",
                "-e",
                "--source",
                "winget",
                "--accept-package-agreements",
                "--accept-source-agreements",
            ])
            .creation_flags(0x08000000)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("winget 不可用，请手动安装 Git: {e}"))?;

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
        let status = child
            .wait()
            .map_err(|e| format!("等待 winget 完成失败: {e}"))?;
        if status.success() {
            let _ = app.emit("upgrade-log", "Git 安装成功！");
            super::refresh_enhanced_path();
            crate::commands::service::invalidate_cli_detection_cache();
            return Ok("Git 已通过 winget 安装".to_string());
        }
        Err("winget 安装 Git 失败，请手动下载安装: https://git-scm.com/downloads".to_string())
    }

    #[cfg(target_os = "macos")]
    {
        let _ = app.emit("upgrade-log", "尝试通过 xcode-select 安装 Git...");
        let mut child = Command::new("xcode-select")
            .arg("--install")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("xcode-select 不可用: {e}"))?;
        let status = child.wait().map_err(|e| format!("等待安装完成失败: {e}"))?;
        if status.success() {
            let _ = app.emit("upgrade-log", "Git 安装已触发，请在弹窗中确认安装。");
            super::refresh_enhanced_path();
            crate::commands::service::invalidate_cli_detection_cache();
            return Ok("已触发 xcode-select 安装，请在弹窗中确认".to_string());
        }
        Err("xcode-select 安装失败，请手动安装 Xcode Command Line Tools 或 brew install git".to_string())
    }

    #[cfg(target_os = "linux")]
    {
        use std::io::{BufRead, BufReader};

        let pkg_mgr = if Command::new("apt-get")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            "apt"
        } else if Command::new("yum")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            "yum"
        } else if Command::new("dnf")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            "dnf"
        } else if Command::new("pacman")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            "pacman"
        } else {
            return Err(
                "未找到包管理器，请手动安装 Git: sudo apt install git 或 sudo yum install git"
                    .to_string(),
            );
        };

        let (cmd_name, args): (&str, Vec<&str>) = match pkg_mgr {
            "apt" => ("sudo", vec!["apt-get", "install", "-y", "git"]),
            "yum" => ("sudo", vec!["yum", "install", "-y", "git"]),
            "dnf" => ("sudo", vec!["dnf", "install", "-y", "git"]),
            "pacman" => ("sudo", vec!["pacman", "-S", "--noconfirm", "git"]),
            _ => return Err("不支持的包管理器".to_string()),
        };

        let _ = app.emit("upgrade-log", format!("执行: {} {}", cmd_name, args.join(" ")));
        let mut child = Command::new(cmd_name)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("安装命令执行失败: {e}"))?;

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
        let status = child.wait().map_err(|e| format!("等待安装完成失败: {e}"))?;
        if status.success() {
            let _ = app.emit("upgrade-log", "Git 安装成功！");
            super::refresh_enhanced_path();
            crate::commands::service::invalidate_cli_detection_cache();
            return Ok("Git 已安装".to_string());
        }
        Err("Git 安装失败，请手动执行: sudo apt install git".to_string())
    }
}

#[tauri::command]
pub fn configure_git_https() -> Result<String, String> {
    let success = ensure_https_rewrites();
    if success > 0 {
        Ok(format!(
            "已配置 Git 使用 HTTPS（{success}/{} 条规则）",
            https_rewrite_rule_count()
        ))
    } else {
        Err("Git 未安装或配置失败".to_string())
    }
}

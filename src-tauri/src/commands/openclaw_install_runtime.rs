use crate::utils::openclaw_command;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

pub(crate) fn standalone_platform_key() -> &'static str {
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        "win-x64"
    }
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        "mac-arm64"
    }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        "mac-x64"
    }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        "linux-x64"
    }
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    {
        "linux-arm64"
    }
    #[cfg(not(any(
        all(target_os = "windows", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "aarch64"),
    )))]
    {
        "unknown"
    }
}

pub(crate) fn standalone_archive_ext() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "zip"
    }
    #[cfg(not(target_os = "windows"))]
    {
        "tar.gz"
    }
}

pub(crate) fn standalone_install_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var("LOCALAPPDATA")
            .ok()
            .map(|dir| PathBuf::from(dir).join("Programs").join("OpenClaw"))
    }
    #[cfg(not(target_os = "windows"))]
    {
        dirs::home_dir().map(|home| home.join(".openclaw-bin"))
    }
}

pub(crate) fn npm_command() -> Command {
    let registry = super::config::get_configured_registry();
    #[cfg(target_os = "windows")]
    {
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let mut cmd = Command::new("cmd");
        cmd.args(["/c", "npm", "--registry", &registry]);
        cmd.env("PATH", super::enhanced_path());
        crate::commands::apply_proxy_env(&mut cmd);
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd
    }
    #[cfg(not(target_os = "windows"))]
    {
        let mut cmd = Command::new("npm");
        cmd.args(["--registry", &registry]);
        cmd.env("PATH", super::enhanced_path());
        crate::commands::apply_proxy_env(&mut cmd);
        cmd
    }
}

#[cfg(target_os = "linux")]
pub(crate) fn nix_is_root() -> bool {
    std::env::var("USER")
        .or_else(|_| std::env::var("EUID"))
        .map(|value| value == "root" || value == "0")
        .unwrap_or(false)
}

#[cfg(target_os = "linux")]
pub(crate) fn npm_prefix_is_user_writable() -> bool {
    if nix_is_root() {
        return true;
    }
    let home = std::env::var("HOME").unwrap_or_default();
    if home.is_empty() {
        return false;
    }
    if let Ok(output) = Command::new("npm")
        .args(["config", "get", "prefix"])
        .env("PATH", super::enhanced_path())
        .output()
    {
        if output.status.success() {
            let prefix = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return !prefix.is_empty() && prefix.starts_with(&home);
        }
    }
    false
}

#[cfg(target_os = "linux")]
fn collect_elevated_env_args() -> Vec<String> {
    let mut env_args = vec![format!("PATH={}", super::enhanced_path())];
    if let Ok(home) = std::env::var("HOME") {
        env_args.push(format!("HOME={home}"));
    }
    if let Some(proxy) = crate::commands::configured_proxy_url() {
        env_args.push(format!("HTTP_PROXY={proxy}"));
        env_args.push(format!("HTTPS_PROXY={proxy}"));
        env_args.push(format!("http_proxy={proxy}"));
        env_args.push(format!("https_proxy={proxy}"));
        env_args.push("NO_PROXY=localhost,127.0.0.1,::1".to_string());
        env_args.push("no_proxy=localhost,127.0.0.1,::1".to_string());
    }
    env_args
}

pub(crate) fn npm_command_elevated() -> Command {
    #[cfg(not(target_os = "linux"))]
    {
        npm_command()
    }
    #[cfg(target_os = "linux")]
    {
        if nix_is_root() || npm_prefix_is_user_writable() {
            return npm_command();
        }
        let registry = super::config::get_configured_registry();
        let env_args = collect_elevated_env_args();
        let has_pkexec = Command::new("which")
            .arg("pkexec")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);
        let mut cmd = if has_pkexec {
            let mut command = Command::new("pkexec");
            command.arg("/usr/bin/env");
            for env_arg in &env_args {
                command.arg(env_arg);
            }
            command.args(["npm", "--registry", &registry]);
            command
        } else {
            let mut command = Command::new("sudo");
            command.arg("--non-interactive");
            command.arg("/usr/bin/env");
            for env_arg in &env_args {
                command.arg(env_arg);
            }
            command.args(["npm", "--registry", &registry]);
            command
        };
        cmd.env("PATH", super::enhanced_path());
        crate::commands::apply_proxy_env(&mut cmd);
        cmd
    }
}

fn run_with_timeout(
    mut child: std::process::Child,
    timeout_secs: u64,
) -> Option<std::process::Output> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout_secs);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let stdout = child
                    .stdout
                    .take()
                    .map(|mut stream| {
                        let mut buffer = Vec::new();
                        let _ = std::io::Read::read_to_end(&mut stream, &mut buffer);
                        buffer
                    })
                    .unwrap_or_default();
                return Some(std::process::Output {
                    status,
                    stdout,
                    stderr: Vec::new(),
                });
            }
            Ok(None) => {
                if std::time::Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return None;
                }
                std::thread::sleep(std::time::Duration::from_millis(200));
            }
            Err(_) => return None,
        }
    }
}

pub(crate) fn pre_install_cleanup() {
    if let Ok(child) = openclaw_command()
        .args(["gateway", "stop"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        run_with_timeout(child, 10);
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(child) = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "Get-CimInstance Win32_Process -Filter \"CommandLine like '%openclaw%gateway%'\" -ErrorAction SilentlyContinue | Select-Object -ExpandProperty ProcessId",
            ])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
        {
            if let Some(output) = run_with_timeout(child, 10) {
                let text = String::from_utf8_lossy(&output.stdout);
                for line in text.lines() {
                    if line.trim().parse::<u32>().is_ok() {
                        let _ = Command::new("taskkill")
                            .args(["/F", "/PID", line.trim()])
                            .output();
                    }
                }
            }
        }

        for standalone_dir in crate::standalone_paths::all_standalone_dirs() {
            if standalone_dir.exists() {
                let dir_lower = standalone_dir
                    .to_string_lossy()
                    .to_lowercase()
                    .replace('\\', "\\\\");
                let ps_script = format!(
                    "Get-Process -Name node -ErrorAction SilentlyContinue | Where-Object {{ $_.Path -and $_.Path.ToLower().Contains('{}') }} | Select-Object -ExpandProperty Id",
                    dir_lower
                );
                if let Ok(child) = Command::new("powershell")
                    .args(["-NoProfile", "-Command", &ps_script])
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::null())
                    .spawn()
                {
                    if let Some(output) = run_with_timeout(child, 10) {
                        let text = String::from_utf8_lossy(&output.stdout);
                        for line in text.lines() {
                            if line.trim().parse::<u32>().is_ok() {
                                let _ = Command::new("taskkill")
                                    .args(["/F", "/PID", line.trim()])
                                    .output();
                            }
                        }
                    }
                }
            }
        }

        std::thread::sleep(std::time::Duration::from_secs(2));
    }

    #[cfg(target_os = "macos")]
    {
        let uid = crate::runtime_support::get_uid().unwrap_or(501);
        if let Ok(child) = Command::new("launchctl")
            .args(["bootout", &format!("gui/{uid}/ai.openclaw.gateway")])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
        {
            run_with_timeout(child, 10);
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(child) = Command::new("pkill")
            .args(["-f", "openclaw.*gateway"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
        {
            run_with_timeout(child, 10);
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    #[cfg(target_os = "windows")]
    {
        if let Some(npm_bin) = npm_global_bin_dir() {
            for name in &["openclaw", "openclaw.cmd", "openclaw.ps1"] {
                let path = npm_bin.join(name);
                if path.exists() {
                    let _ = fs::remove_file(&path);
                }
            }
        }
    }
}

pub(crate) fn r2_platform_key() -> &'static str {
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        "win-x64"
    }
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        "darwin-arm64"
    }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        "darwin-x64"
    }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        "linux-x64"
    }
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    {
        "linux-arm64"
    }
    #[cfg(not(any(
        all(target_os = "windows", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "aarch64"),
    )))]
    {
        "unknown"
    }
}

#[allow(dead_code)]
pub(crate) fn npm_global_modules_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        super::windows_npm_global_prefix()
            .map(|prefix| PathBuf::from(prefix).join("node_modules"))
            .or_else(|| {
                std::env::var("APPDATA")
                    .ok()
                    .map(|appdata| PathBuf::from(appdata).join("npm").join("node_modules"))
            })
    }
    #[cfg(target_os = "macos")]
    {
        let brew = PathBuf::from("/opt/homebrew/lib/node_modules");
        if brew.exists() {
            return Some(brew);
        }
        let sys = PathBuf::from("/usr/local/lib/node_modules");
        if sys.exists() {
            return Some(sys);
        }
        Some(brew)
    }
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = Command::new("npm").args(["config", "get", "prefix"]).output() {
            let prefix = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !prefix.is_empty() {
                return Some(PathBuf::from(prefix).join("lib").join("node_modules"));
            }
        }
        Some(PathBuf::from("/usr/local/lib/node_modules"))
    }
}

#[allow(dead_code)]
pub(crate) fn npm_global_bin_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        super::windows_npm_global_prefix()
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var("APPDATA")
                    .ok()
                    .map(|appdata| PathBuf::from(appdata).join("npm"))
            })
    }
    #[cfg(target_os = "macos")]
    {
        let brew = PathBuf::from("/opt/homebrew/bin");
        if brew.exists() {
            return Some(brew);
        }
        Some(PathBuf::from("/usr/local/bin"))
    }
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = Command::new("npm").args(["config", "get", "prefix"]).output() {
            let prefix = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !prefix.is_empty() {
                return Some(PathBuf::from(prefix).join("bin"));
            }
        }
        Some(PathBuf::from("/usr/local/bin"))
    }
}

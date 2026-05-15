use std::path::PathBuf;

pub(crate) fn all_standalone_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    #[cfg(target_os = "windows")]
    {
        if let Ok(la) = std::env::var("LOCALAPPDATA") {
            dirs.push(PathBuf::from(&la).join("Programs").join("OpenClaw"));
            dirs.push(PathBuf::from(&la).join("OpenClaw"));
        }
        if let Ok(pf) = std::env::var("ProgramFiles") {
            dirs.push(PathBuf::from(pf).join("OpenClaw"));
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        if let Some(h) = dirs::home_dir() {
            dirs.push(h.join(".openclaw-bin"));
        }
        dirs.push(PathBuf::from("/opt/openclaw"));
    }
    dirs
}

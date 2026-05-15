pub(crate) fn is_rejected_cli_path(cli_path: &str) -> bool {
    let lower = cli_path.replace('\\', "/").to_lowercase();
    lower.contains("/.cherrystudio/") || lower.contains("cherry-studio")
}

pub(crate) fn resolve_openclaw_cli_input_path(
    cli_path: &std::path::Path,
) -> Option<std::path::PathBuf> {
    if cli_path.as_os_str().is_empty() {
        return None;
    }
    let input = cli_path.to_path_buf();
    let mut candidates: Vec<std::path::PathBuf> = Vec::new();

    if input.is_dir() {
        #[cfg(target_os = "windows")]
        {
            candidates.push(input.join("openclaw.cmd"));
            candidates.push(input.join("openclaw.exe"));
            candidates.push(input.join("openclaw"));
        }
        #[cfg(not(target_os = "windows"))]
        {
            candidates.push(input.join("openclaw"));
        }
    } else {
        candidates.push(input);
    }

    candidates
        .into_iter()
        .find(|candidate| candidate.exists() && !is_rejected_cli_path(&candidate.to_string_lossy()))
}

pub(crate) fn resolve_openclaw_cli_input(cli_path: &str) -> Option<std::path::PathBuf> {
    let raw = cli_path.trim();
    if raw.is_empty() {
        return None;
    }
    resolve_openclaw_cli_input_path(std::path::Path::new(raw))
}

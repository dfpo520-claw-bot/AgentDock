#[derive(Debug)]
pub(crate) struct GuardianPause {
    reason: &'static str,
}

impl GuardianPause {
    pub(crate) fn new(reason: &'static str) -> Self {
        crate::commands::service::guardian_pause(reason);
        Self { reason }
    }
}

impl Drop for GuardianPause {
    fn drop(&mut self) {
        crate::commands::service::guardian_resume(self.reason);
    }
}

#[allow(dead_code)]
pub(crate) fn get_uid() -> Result<u32, String> {
    #[cfg(target_os = "windows")]
    {
        Ok(0)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let output = std::process::Command::new("id")
            .arg("-u")
            .output()
            .map_err(|e| format!("获取 UID 失败: {e}"))?;
        String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse::<u32>()
            .map_err(|e| format!("解析 UID 失败: {e}"))
    }
}

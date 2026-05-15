/// 配置读写命令
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// 预设 npm 源列表
const DEFAULT_REGISTRY: &str = "https://registry.npmmirror.com";

pub(crate) fn get_configured_registry() -> String {
    let path = super::openclaw_dir().join("npm-registry.txt");
    fs::read_to_string(&path)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| DEFAULT_REGISTRY.to_string())
}

fn backups_dir() -> PathBuf {
    super::openclaw_dir().join("backups")
}

#[tauri::command]
pub fn read_openclaw_config() -> Result<Value, String> {
    let path = super::openclaw_dir().join("openclaw.json");
    let raw = fs::read(&path).map_err(|e| format!("读取配置失败: {e}"))?;

    // 自愈：自动剥离 UTF-8 BOM（EF BB BF），防止 JSON 解析失败
    let content = if raw.starts_with(&[0xEF, 0xBB, 0xBF]) {
        String::from_utf8_lossy(&raw[3..]).into_owned()
    } else {
        String::from_utf8_lossy(&raw).into_owned()
    };

    // 解析 JSON，失败时尝试自动修复或从备份恢复
    let mut config: Value = match serde_json::from_str(&content) {
        Ok(v) => {
            // BOM 被剥离过，静默写回干净文件
            if raw.starts_with(&[0xEF, 0xBB, 0xBF]) {
                let _ = fs::write(&path, &content);
            }
            v
        }
        Err(e) => {
            // JSON 解析失败，尝试自动修复
            let fixed_content = fix_common_json_errors(&content);
            if let Ok(v) = serde_json::from_str(&fixed_content) {
                eprintln!("自动修复了配置文件的 JSON 语法错误");
                // 写回修复后的配置
                let _ = fs::write(&path, &fixed_content);
                v
            } else {
                // 自动修复失败，尝试从备份恢复
                let bak = super::openclaw_dir().join("openclaw.json.bak");
                if bak.exists() {
                    let bak_raw = fs::read(&bak).map_err(|e2| format!("备份也读取失败: {e2}"))?;
                    let bak_content = if bak_raw.starts_with(&[0xEF, 0xBB, 0xBF]) {
                        String::from_utf8_lossy(&bak_raw[3..]).into_owned()
                    } else {
                        String::from_utf8_lossy(&bak_raw).into_owned()
                    };
                    let bak_config: Value = serde_json::from_str(&bak_content).map_err(|e2| {
                        format!("配置损坏且备份也无效: 原始错误='{}', 备份错误='{}'", e, e2)
                    })?;
                    // 备份有效，恢复主文件
                    let _ = fs::write(&path, &bak_content);
                    eprintln!("从备份恢复了配置文件");
                    bak_config
                } else {
                    return Err(format!(
                        "配置 JSON 损坏且无备份: {} (行: {}, 列: {})",
                        e,
                        e.line(),
                        e.column()
                    ));
                }
            }
        }
    };

    // 自动清理 UI 专属字段，防止污染配置导致 CLI 启动失败
    if has_ui_fields(&config) {
        config = strip_ui_fields(config);
        // 静默写回清理后的配置
        let bak = super::openclaw_dir().join("openclaw.json.bak");
        let _ = fs::copy(&path, &bak);
        let json = serde_json::to_string_pretty(&config).map_err(|e| format!("序列化失败: {e}"))?;
        let _ = fs::write(&path, json);
    }

    Ok(config)
}

/// 尝试自动修复常见的 JSON 语法错误
/// Issue #127: 增强配置读取容错性
fn fix_common_json_errors(content: &str) -> String {
    let mut fixed = content.to_string();

    // 修复尾随逗号（在 ] 或 } 之前的逗号）
    // 模式: ,] 或 ,}
    fixed = fixed.replace(",]", "]");
    fixed = fixed.replace(",}", "}");

    // 修复多余逗号（在键值对后面的逗号）
    while fixed.contains(",,") {
        fixed = fixed.replace(",,", ",");
    }

    // 修复单引号：在字符串外将单引号替换为双引号
    fixed = simple_fix_single_quotes(&fixed);

    // 移除 JavaScript 风格的注释（// 或 /* */）
    // 注意：必须正确处理字符串内的 // （如 URL 中的 https://）
    let lines: Vec<&str> = fixed.lines().collect();
    let cleaned_lines: Vec<&str> = lines
        .iter()
        .map(|line| {
            // 逐字符扫描，跳过字符串内部，找到字符串外的 //
            let chars: Vec<char> = line.chars().collect();
            let mut in_string = false;
            let mut i = 0;
            while i < chars.len() {
                if chars[i] == '\\' && in_string {
                    // 转义字符，跳过下一个字符
                    i += 2;
                    continue;
                }
                if chars[i] == '"' {
                    in_string = !in_string;
                }
                if !in_string && i + 1 < chars.len() && chars[i] == '/' && chars[i + 1] == '/' {
                    // 找到字符串外的 //，截断该行
                    let truncated: String = chars[..i].iter().collect();
                    return Box::leak(truncated.into_boxed_str()) as &str;
                }
                i += 1;
            }
            *line
        })
        .collect();
    fixed = cleaned_lines.join("\n");

    // 移除多行注释 /* ... */
    // 简化处理：只在确认不在字符串内时移除
    static RE_MULTI_COMMENT: std::sync::LazyLock<regex::Regex> =
        std::sync::LazyLock::new(|| regex::Regex::new(r"/\*[\s\S]*?\*/").unwrap());
    if RE_MULTI_COMMENT.is_match(&fixed) {
        fixed = RE_MULTI_COMMENT.replace_all(&fixed, "").to_string();
    }

    fixed
}

/// 简单的单引号修复（fallback 方案）
fn simple_fix_single_quotes(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let mut in_string = false;
    let chars: Vec<char> = content.chars().collect();

    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let prev_char = if i > 0 { Some(chars[i - 1]) } else { None };

        if c == '"' && prev_char != Some('\\') {
            in_string = !in_string;
            result.push(c);
        } else if !in_string && c == '\'' {
            // 在字符串外，将单引号替换为双引号
            result.push('"');
        } else {
            result.push(c);
        }
        i += 1;
    }

    result
}

/// 供其他模块复用：读取 openclaw.json 为 JSON Value
pub fn load_openclaw_json() -> Result<Value, String> {
    read_openclaw_config()
}

/// 供其他模块复用：将 JSON Value 写回 openclaw.json（含备份和清理）
pub fn save_openclaw_json(config: &Value) -> Result<(), String> {
    write_openclaw_config(config.clone())
}

#[tauri::command]
pub fn write_openclaw_config(config: Value) -> Result<(), String> {
    let path = super::openclaw_dir().join("openclaw.json");

    // Issue #127 修复：先读取现有配置，合并后写入
    // 这样可以保留用户手动添加的合法字段（如 browser.profiles）
    // 即使这些字段不在前端传入的配置对象中
    let existing_config = fs::read_to_string(&path)
        .ok()
        .and_then(|c| serde_json::from_str::<Value>(&c).ok());

    // 备份
    let bak = super::openclaw_dir().join("openclaw.json.bak");
    let _ = fs::copy(&path, &bak);

    // 合并配置：现有配置 + 新配置
    // 策略：遍历现有配置，保留所有非 UI 字段
    // 然后将新配置的值覆盖到合并结果中
    let merged = if let Some(existing) = existing_config {
        merge_configs_preserving_fields(&existing, &config)
    } else {
        config.clone()
    };

    // 清理 UI 专属字段，避免 CLI schema 校验失败
    let cleaned = strip_ui_fields(merged);

    // 写入
    let json = serde_json::to_string_pretty(&cleaned).map_err(|e| format!("序列化失败: {e}"))?;
    fs::write(&path, &json).map_err(|e| format!("写入失败: {e}"))?;

    // 同步 provider 配置到所有 agent 的 models.json（运行时注册表）
    sync_providers_to_agent_models(&config);

    Ok(())
}

const CALIBRATION_RESET_INHERIT_KEYS: &[&str] = &[
    "agents", "auth", "bindings", "browser", "channels", "commands", "env", "hooks", "models",
    "plugins", "session", "skills", "wizard",
];

fn calibration_required_origins() -> Vec<String> {
    vec![
        "tauri://localhost".into(),
        "https://tauri.localhost".into(),
        "http://tauri.localhost".into(),
        "http://localhost".into(),
        "http://localhost:1420".into(),
        "http://127.0.0.1:1420".into(),
        "http://localhost:18777".into(),
        "http://127.0.0.1:18777".into(),
    ]
}

fn calibration_last_touched_version() -> String {
    super::openclaw_install_policy::recommended_version_for("chinese")
        .unwrap_or_else(|| "2026.1.1".to_string())
}

fn calibration_default_workspace() -> String {
    super::openclaw_dir()
        .join("workspace")
        .to_string_lossy()
        .to_string()
}

fn generate_calibration_token() -> String {
    format!(
        "cp-{:016x}{:016x}",
        rand::random::<u64>(),
        rand::random::<u64>()
    )
}

fn decode_json_bytes(raw: &[u8]) -> String {
    if raw.starts_with(&[0xEF, 0xBB, 0xBF]) {
        String::from_utf8_lossy(&raw[3..]).into_owned()
    } else {
        String::from_utf8_lossy(raw).into_owned()
    }
}

fn parse_json_relaxed(content: &str) -> Option<Value> {
    serde_json::from_str(content)
        .ok()
        .or_else(|| serde_json::from_str(&fix_common_json_errors(content)).ok())
}

fn read_json_file_relaxed(path: &PathBuf) -> Option<Value> {
    let raw = fs::read(path).ok()?;
    let content = decode_json_bytes(&raw);
    parse_json_relaxed(&content)
}

fn calibration_has_usable_gateway_auth(auth: &Value) -> bool {
    let mode = auth.get("mode").and_then(|v| v.as_str()).unwrap_or("");
    match mode {
        "token" => auth
            .get("token")
            .and_then(|v| v.as_str())
            .map(|v| !v.trim().is_empty())
            .unwrap_or(false),
        "password" => auth
            .get("password")
            .and_then(|v| v.as_str())
            .map(|v| !v.trim().is_empty())
            .unwrap_or(false),
        _ => false,
    }
}

fn calibration_richness_score(config: &Value) -> usize {
    let mut score = 0;
    if config
        .pointer("/models/providers")
        .and_then(|v| v.as_object())
        .map(|v| !v.is_empty())
        .unwrap_or(false)
    {
        score += 4;
    }
    if config.pointer("/agents/defaults").is_some() {
        score += 2;
    }
    if config
        .pointer("/agents/list")
        .and_then(|v| v.as_array())
        .map(|v| !v.is_empty())
        .unwrap_or(false)
    {
        score += 3;
    }
    if config
        .get("channels")
        .and_then(|v| v.as_object())
        .map(|v| !v.is_empty())
        .unwrap_or(false)
    {
        score += 2;
    }
    if config
        .get("bindings")
        .and_then(|v| v.as_array())
        .map(|v| !v.is_empty())
        .unwrap_or(false)
    {
        score += 2;
    }
    if config
        .pointer("/plugins/entries")
        .and_then(|v| v.as_object())
        .map(|v| !v.is_empty())
        .unwrap_or(false)
        || config
            .pointer("/plugins/installs")
            .and_then(|v| v.as_object())
            .map(|v| !v.is_empty())
            .unwrap_or(false)
    {
        score += 2;
    }
    if config
        .get("env")
        .and_then(|v| v.as_object())
        .map(|v| !v.is_empty())
        .unwrap_or(false)
    {
        score += 1;
    }
    if config
        .pointer("/gateway/auth")
        .map(calibration_has_usable_gateway_auth)
        .unwrap_or(false)
    {
        score += 3;
    }
    if config
        .pointer("/gateway/controlUi/allowedOrigins")
        .and_then(|v| v.as_array())
        .map(|v| !v.is_empty())
        .unwrap_or(false)
    {
        score += 1;
    }
    score
}

fn select_calibration_source(current: Option<Value>, backup: Option<Value>) -> (String, Value) {
    match (current, backup) {
        (Some(current), Some(backup)) => {
            let current_score = calibration_richness_score(&current);
            let backup_score = calibration_richness_score(&backup);
            if backup_score > current_score {
                ("backup".into(), backup)
            } else {
                ("current".into(), current)
            }
        }
        (Some(current), None) => ("current".into(), current),
        (None, Some(backup)) => ("backup".into(), backup),
        (None, None) => ("empty".into(), json!({})),
    }
}

fn build_calibration_baseline() -> Value {
    json!({
        "$schema": "https://openclaw.ai/schema/config.json",
        "meta": {
            "lastTouchedVersion": calibration_last_touched_version(),
        },
        "models": { "providers": {} },
        "agents": {
            "defaults": {
                "workspace": calibration_default_workspace(),
            },
            "list": [],
        },
        "bindings": [],
        "channels": {},
        "commands": {
            "native": "auto",
            "nativeSkills": "auto",
            "ownerDisplay": "raw",
            "restart": true,
        },
        "plugins": {},
        "session": { "dmScope": "per-channel-peer" },
        "skills": { "entries": {} },
        "tools": {
            "profile": "full",
            "sessions": { "visibility": "all" },
        },
        "gateway": {
            "mode": "local",
            "bind": "loopback",
            "port": 18789,
            "auth": {
                "mode": "token",
                "token": generate_calibration_token(),
            },
            "controlUi": {
                "enabled": true,
                "allowedOrigins": calibration_required_origins(),
                "allowInsecureAuth": true,
            },
        },
    })
}

fn apply_reset_inheritance(mut config: Value, seed: &Value) -> (Value, Vec<String>) {
    let mut inherited = Vec::new();
    let Some(root) = config.as_object_mut() else {
        return (config, inherited);
    };

    for key in CALIBRATION_RESET_INHERIT_KEYS {
        if let Some(value) = seed.get(*key) {
            root.insert((*key).to_string(), value.clone());
            inherited.push((*key).to_string());
        }
    }

    if let Some(web) = seed.pointer("/tools/web").cloned() {
        let tools = root.entry("tools").or_insert_with(|| json!({}));
        if !tools.is_object() {
            *tools = json!({});
        }
        if let Some(tools_obj) = tools.as_object_mut() {
            tools_obj.insert("web".into(), web);
            inherited.push("tools.web".into());
        }
    }

    (config, inherited)
}

fn normalize_calibrated_config(mut config: Value) -> Value {
    let required_origins = calibration_required_origins();
    let last_touched_version = calibration_last_touched_version();
    let default_workspace = calibration_default_workspace();

    let Some(root) = config.as_object_mut() else {
        return build_calibration_baseline();
    };

    root.insert(
        "$schema".into(),
        Value::String("https://openclaw.ai/schema/config.json".into()),
    );

    let meta = root.entry("meta").or_insert_with(|| json!({}));
    if !meta.is_object() {
        *meta = json!({});
    }
    if let Some(meta_obj) = meta.as_object_mut() {
        meta_obj.insert(
            "lastTouchedVersion".into(),
            Value::String(last_touched_version),
        );
        meta_obj.insert(
            "lastTouchedAt".into(),
            Value::String(chrono::Utc::now().to_rfc3339()),
        );
    }

    let models = root.entry("models").or_insert_with(|| json!({}));
    if !models.is_object() {
        *models = json!({});
    }
    if let Some(models_obj) = models.as_object_mut() {
        let providers = models_obj.entry("providers").or_insert_with(|| json!({}));
        if !providers.is_object() {
            *providers = json!({});
        }
    }

    let agents = root.entry("agents").or_insert_with(|| json!({}));
    if !agents.is_object() {
        *agents = json!({});
    }
    if let Some(agents_obj) = agents.as_object_mut() {
        let defaults = agents_obj.entry("defaults").or_insert_with(|| json!({}));
        if !defaults.is_object() {
            *defaults = json!({});
        }
        if let Some(defaults_obj) = defaults.as_object_mut() {
            if !defaults_obj
                .get("workspace")
                .and_then(|v| v.as_str())
                .map(|v| !v.trim().is_empty())
                .unwrap_or(false)
            {
                defaults_obj.insert("workspace".into(), Value::String(default_workspace));
            }
        }
        let list = agents_obj.entry("list").or_insert_with(|| json!([]));
        if !list.is_array() {
            *list = json!([]);
        }
    }

    let bindings = root.entry("bindings").or_insert_with(|| json!([]));
    if !bindings.is_array() {
        *bindings = json!([]);
    }

    let channels = root.entry("channels").or_insert_with(|| json!({}));
    if !channels.is_object() {
        *channels = json!({});
    }

    let plugins = root.entry("plugins").or_insert_with(|| json!({}));
    if !plugins.is_object() {
        *plugins = json!({});
    }

    let tools = root.entry("tools").or_insert_with(|| json!({}));
    if !tools.is_object() {
        *tools = json!({});
    }
    if let Some(tools_obj) = tools.as_object_mut() {
        if !tools_obj
            .get("profile")
            .and_then(|v| v.as_str())
            .map(|v| !v.trim().is_empty())
            .unwrap_or(false)
        {
            tools_obj.insert("profile".into(), Value::String("full".into()));
        }
        let sessions = tools_obj.entry("sessions").or_insert_with(|| json!({}));
        if !sessions.is_object() {
            *sessions = json!({});
        }
        if let Some(sessions_obj) = sessions.as_object_mut() {
            if !sessions_obj
                .get("visibility")
                .and_then(|v| v.as_str())
                .map(|v| !v.trim().is_empty())
                .unwrap_or(false)
            {
                sessions_obj.insert("visibility".into(), Value::String("all".into()));
            }
        }
    }

    let gateway = root.entry("gateway").or_insert_with(|| json!({}));
    if !gateway.is_object() {
        *gateway = json!({});
    }
    if let Some(gateway_obj) = gateway.as_object_mut() {
        if !gateway_obj
            .get("mode")
            .and_then(|v| v.as_str())
            .map(|v| !v.trim().is_empty())
            .unwrap_or(false)
        {
            gateway_obj.insert("mode".into(), Value::String("local".into()));
        }

        let port_valid = gateway_obj
            .get("port")
            .and_then(|v| v.as_u64())
            .map(|port| (1..=65535).contains(&port))
            .unwrap_or(false);
        if !port_valid {
            gateway_obj.insert("port".into(), json!(18789));
        }

        if !gateway_obj
            .get("bind")
            .and_then(|v| v.as_str())
            .map(|v| !v.trim().is_empty())
            .unwrap_or(false)
        {
            gateway_obj.insert("bind".into(), Value::String("loopback".into()));
        }

        let auth_valid = gateway_obj
            .get("auth")
            .map(calibration_has_usable_gateway_auth)
            .unwrap_or(false);
        if !auth_valid {
            gateway_obj.insert(
                "auth".into(),
                json!({
                    "mode": "token",
                    "token": generate_calibration_token(),
                }),
            );
        }

        let control_ui = gateway_obj.entry("controlUi").or_insert_with(|| json!({}));
        if !control_ui.is_object() {
            *control_ui = json!({});
        }
        if let Some(control_ui_obj) = control_ui.as_object_mut() {
            let existing: Vec<String> = control_ui_obj
                .get("allowedOrigins")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|value| value.as_str().map(|value| value.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            let mut merged = existing;
            for origin in required_origins {
                if !merged.iter().any(|existing| existing == &origin) {
                    merged.push(origin);
                }
            }
            control_ui_obj.insert("allowedOrigins".into(), json!(merged));
            control_ui_obj.insert("enabled".into(), Value::Bool(true));
            control_ui_obj.insert("allowInsecureAuth".into(), Value::Bool(true));
        }
    }

    config
}

#[tauri::command]
pub fn calibrate_openclaw_config(mode: String) -> Result<Value, String> {
    let normalized_mode = match mode.trim() {
        "inherit" => "inherit",
        "reset" | "reinitialize" => "reset",
        _ => return Err("mode 必须是 inherit 或 reset".into()),
    };

    let dir = super::openclaw_dir();
    let config_path = dir.join("openclaw.json");
    let backup_path = dir.join("openclaw.json.bak");
    fs::create_dir_all(&dir).map_err(|e| format!("创建配置目录失败: {e}"))?;

    let mut warnings: Vec<String> = vec![];
    let pre_backup = if config_path.exists() {
        match create_backup() {
            Ok(result) => result
                .get("name")
                .and_then(|value| value.as_str())
                .map(|value| value.to_string()),
            Err(err) => {
                warnings.push(format!("修复前备份失败: {err}"));
                None
            }
        }
    } else {
        None
    };

    let current = read_json_file_relaxed(&config_path);
    let backup = read_json_file_relaxed(&backup_path);
    let (source, seed) = select_calibration_source(current, backup);

    let (calibrated, mut inherited_keys) = if normalized_mode == "inherit" {
        let inherited = seed
            .as_object()
            .map(|obj| obj.keys().cloned().collect())
            .unwrap_or_else(Vec::new);
        (
            merge_configs_preserving_fields(&build_calibration_baseline(), &seed),
            inherited,
        )
    } else {
        apply_reset_inheritance(build_calibration_baseline(), &seed)
    };

    inherited_keys.sort();
    inherited_keys.dedup();

    let calibrated = strip_ui_fields(normalize_calibrated_config(calibrated));
    let json = serde_json::to_string_pretty(&calibrated)
        .map_err(|e| format!("序列化校准配置失败: {e}"))?;

    fs::write(&config_path, &json).map_err(|e| format!("写入校准配置失败: {e}"))?;
    fs::write(&backup_path, &json).map_err(|e| format!("写入配置备份失败: {e}"))?;

    sync_providers_to_agent_models(&calibrated);

    Ok(json!({
        "mode": normalized_mode,
        "source": source,
        "backup": pre_backup,
        "inheritedKeys": inherited_keys,
        "warnings": warnings,
        "message": if normalized_mode == "inherit" {
            "配置已按继承模式校准"
        } else {
            "配置已按完全初始化修复模式校准"
        }
    }))
}

/// 合并两个配置对象，保留现有配置中的合法字段
///
/// Issue #127: 修复配置合并时丢失 browser.* 等合法字段的问题
///
/// 策略：对所有顶级 Object 类型字段做浅合并（新值覆盖旧值，旧值中新配置没有的字段保留）。
/// 这样用户通过 CLI / 手动编辑添加的自定义子字段不会被前端的部分配置所覆盖掉。
///
/// 清理的字段：
/// - UI 专属字段（通过 strip_ui_fields 处理）
fn merge_configs_preserving_fields(existing: &Value, new: &Value) -> Value {
    use serde_json::Value;

    match (existing, new) {
        (Value::Object(existing_obj), Value::Object(new_obj)) => {
            let mut merged = existing_obj.clone();

            for (key, new_value) in new_obj {
                if let Some(existing_value) = existing_obj.get(key) {
                    if let (Value::Object(existing_sub), Value::Object(new_sub)) =
                        (existing_value, new_value)
                    {
                        // 两边都是对象：浅合并（新值覆盖，旧值保留未覆盖的 key）
                        let mut sub_merged = existing_sub.clone();
                        for (sub_key, sub_value) in new_sub {
                            sub_merged.insert(sub_key.clone(), sub_value.clone());
                        }
                        merged.insert(key.clone(), Value::Object(sub_merged));
                    } else {
                        // 类型不同或不是对象，直接使用新值
                        merged.insert(key.clone(), new_value.clone());
                    }
                } else {
                    // 现有配置没有此 key，使用新值
                    merged.insert(key.clone(), new_value.clone());
                }
            }

            Value::Object(merged)
        }
        // 非对象类型，直接使用新配置
        _ => new.clone(),
    }
}

/// 已知需要清理的 UI 字段列表（用于诊断报告）
const KNOWN_UI_FIELDS: &[&str] = &[
    "current",
    "latest",
    "recommended",
    "update_available",
    "latest_update_available",
    "is_recommended",
    "ahead_of_recommended",
    "panel_version",
    "source",
    // models.providers 中的 UI 字段
    "lastTestAt",
    "latency",
    "testStatus",
    "testError",
    "profiles",
];

/// 已知需要保留的合法 OpenClaw 配置字段（用于诊断报告）
/// 这些字段虽然不在标准列表中，但不应被警告为未知字段
/// 注意：这些字段在 `merge_configs_preserving_fields` 中会被特殊处理
#[allow(dead_code)]
const KNOWN_LEGAL_FIELDS: &[&str] = &["browser", "agents", "gateway", "logging", "mcp"];

// KNOWN_LEGAL_FIELDS 目前在诊断逻辑中使用，用于生成报告信息

/// 验证 openclaw.json 配置，报告潜在问题
///
/// Issue #127: 新增诊断命令，帮助用户识别配置问题
///
/// 返回内容：
/// - config_valid: 配置是否可以正常读取
/// - ui_fields_found: 发现的 UI 专属字段（会被自动清理）
/// - unknown_fields: 未知的字段（可能是用户手动添加或 OpenClaw 新增）
/// - warnings: 警告信息和建议
#[tauri::command]
pub fn validate_openclaw_config() -> Result<Value, String> {
    let path = super::openclaw_dir().join("openclaw.json");

    // 读取原始内容（不经过自愈逻辑）
    let raw = fs::read(&path).map_err(|e| format!("读取配置失败: {e}"))?;
    let content = if raw.starts_with(&[0xEF, 0xBB, 0xBF]) {
        String::from_utf8_lossy(&raw[3..]).into_owned()
    } else {
        String::from_utf8_lossy(&raw).into_owned()
    };

    // 尝试解析 JSON
    let config: Value = match serde_json::from_str(&content) {
        Ok(v) => {
            // BOM 被剥离过，静默写回干净文件
            if raw.starts_with(&[0xEF, 0xBB, 0xBF]) {
                let _ = fs::write(&path, &content);
            }
            v
        }
        Err(e) => {
            // JSON 解析失败，尝试自动修复
            let fixed_content = fix_common_json_errors(&content);
            if let Ok(v) = serde_json::from_str(&fixed_content) {
                eprintln!("自动修复了配置文件的 JSON 语法错误");
                // 写回修复后的配置
                let _ = fs::write(&path, &fixed_content);
                v
            } else {
                // 自动修复失败，尝试从备份恢复
                let bak = super::openclaw_dir().join("openclaw.json.bak");
                if bak.exists() {
                    if let Ok(bak_content) = fs::read_to_string(&bak) {
                        if serde_json::from_str::<Value>(&bak_content).is_ok() {
                            return Ok(json!({
                                "config_valid": false,
                                "json_error": format!("JSON 解析失败 (行: {}, 列: {}), 建议从备份恢复", e.line(), e.column()),
                                "backup_exists": true,
                                "warnings": [
                                    "配置文件损坏，建议使用备份恢复",
                                    "备份文件：openclaw.json.bak"
                                ]
                            }));
                        }
                    }
                }
                return Ok(json!({
                    "config_valid": false,
                    "json_error": format!("JSON 解析失败 (行: {}, 列: {}): {}", e.line(), e.column(), e),
                    "warnings": [
                        "配置文件严重损坏且无有效备份",
                        "建议：手动检查或重新创建配置文件"
                    ]
                }));
            }
        }
    };

    // 分析配置内容
    let mut ui_fields_found: Vec<String> = Vec::new();
    let mut unknown_fields: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    // 检查根层级的 UI 字段
    if let Some(obj) = config.as_object() {
        for key in obj.keys() {
            if KNOWN_UI_FIELDS.contains(&key.as_str()) {
                ui_fields_found.push(format!("根层级.{}", key));
            }
        }

        // 检查 browser 字段是否存在
        if obj.contains_key("browser") {
            if let Some(browser) = obj.get("browser") {
                if let Some(browser_obj) = browser.as_object() {
                    // 检查 browser.profiles
                    if browser_obj.contains_key("profiles") {
                        warnings.push(
                            "发现 browser.profiles 字段，这是 OpenClaw 合法的配置字段，将被保留"
                                .to_string(),
                        );
                    }
                    // 报告 browser 中的其他未知字段
                    for key in browser_obj.keys() {
                        if key != "profiles" {
                            unknown_fields.push(format!("browser.{}", key));
                        }
                    }
                }
            }
        }

        // 检查 agents 字段
        if obj.contains_key("agents") {
            if let Some(agents) = obj.get("agents") {
                if let Some(agents_obj) = agents.as_object() {
                    // 检查 agents 子字段（上游 schema 只定义 agents.list）
                    if agents_obj.contains_key("profiles") {
                        warnings.push(
                            "发现 agents.profiles 字段，上游 schema 未定义此字段，ClawPanel 会自动清理"
                                .to_string(),
                        );
                    }
                    // 检查 agents.list 中的元素
                    if let Some(Value::Array(list)) = agents_obj.get("list") {
                        for (idx, agent) in list.iter().enumerate() {
                            if let Some(agent_obj) = agent.as_object() {
                                for key in agent_obj.keys() {
                                    if KNOWN_UI_FIELDS.contains(&key.as_str()) {
                                        ui_fields_found
                                            .push(format!("agents.list[{}].{}", idx, key));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 检查 models.providers 中的测试状态字段
        if let Some(models) = obj.get("models") {
            if let Some(models_obj) = models.as_object() {
                if let Some(providers) = models_obj.get("providers") {
                    if let Some(providers_obj) = providers.as_object() {
                        for (provider_name, provider_val) in providers_obj {
                            if let Some(provider_obj) = provider_val.as_object() {
                                if let Some(Value::Array(models_arr)) = provider_obj.get("models") {
                                    for (model_idx, model) in models_arr.iter().enumerate() {
                                        if let Some(model_obj) = model.as_object() {
                                            for field in
                                                ["lastTestAt", "latency", "testStatus", "testError"]
                                            {
                                                if model_obj.contains_key(field) {
                                                    ui_fields_found.push(format!(
                                                        "models.providers.{}.models[{}].{}",
                                                        provider_name, model_idx, field
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 生成警告信息
        if !ui_fields_found.is_empty() {
            warnings.push(format!(
                "发现 {} 个 UI 专属字段，将被自动清理",
                ui_fields_found.len()
            ));
        }
    }

    Ok(json!({
        "config_valid": true,
        "ui_fields_found": ui_fields_found,
        "unknown_fields": unknown_fields,
        "warnings": warnings,
        "suggestions": if !ui_fields_found.is_empty() || !unknown_fields.is_empty() {
            vec![
                "UI 专属字段会被 ClawPanel 自动清理，不影响 OpenClaw 运行".to_string(),
                "未知字段如果是用户手动添加的，请确保符合 OpenClaw schema".to_string(),
                "如果遇到 'Unrecognized key' 错误，请检查配置文件是否包含 OpenClaw 不支持的字段".to_string(),
            ]
        } else {
            vec!["配置文件看起来正常，没有发现已知问题".to_string()]
        }
    }))
}

/// 将 openclaw.json 的 models.providers 完整同步到每个 agent 的 models.json
/// 包括：同步 baseUrl/apiKey/api + 清理已删除的 models
/// 确保 Gateway 运行时不会引用 openclaw.json 中已不存在的模型
fn sync_providers_to_agent_models(config: &Value) {
    let src_providers = config
        .pointer("/models/providers")
        .and_then(|p| p.as_object());

    // 收集 openclaw.json 中所有有效的 provider/model 组合
    let mut valid_models: std::collections::HashSet<String> = std::collections::HashSet::new();
    if let Some(providers) = src_providers {
        for (pk, pv) in providers {
            if let Some(models) = pv.get("models").and_then(|m| m.as_array()) {
                for m in models {
                    let id = m.get("id").and_then(|v| v.as_str()).or_else(|| m.as_str());
                    if let Some(id) = id {
                        valid_models.insert(format!("{}/{}", pk, id));
                    }
                }
            }
        }
    }

    // 收集所有 agent ID
    let mut agent_ids = vec!["main".to_string()];
    if let Some(Value::Array(list)) = config.pointer("/agents/list") {
        for agent in list {
            if let Some(id) = agent.get("id").and_then(|v| v.as_str()) {
                if id != "main" {
                    agent_ids.push(id.to_string());
                }
            }
        }
    }

    let agents_dir = super::openclaw_dir().join("agents");
    for agent_id in &agent_ids {
        let models_path = agents_dir.join(agent_id).join("agent").join("models.json");
        if !models_path.exists() {
            continue;
        }
        let Ok(content) = fs::read_to_string(&models_path) else {
            continue;
        };
        let Ok(mut models_json) = serde_json::from_str::<Value>(&content) else {
            continue;
        };

        let mut changed = false;

        if models_json
            .get("providers")
            .and_then(|p| p.as_object())
            .is_none()
        {
            if let Some(root) = models_json.as_object_mut() {
                root.insert("providers".into(), json!({}));
                changed = true;
            }
        }

        // 同步 providers
        if let Some(dst_providers) = models_json
            .get_mut("providers")
            .and_then(|p| p.as_object_mut())
        {
            // 1. 删除 openclaw.json 中已不存在的 provider
            if let Some(src) = src_providers {
                let to_remove: Vec<String> = dst_providers
                    .keys()
                    .filter(|k| !src.contains_key(k.as_str()))
                    .cloned()
                    .collect();
                for k in to_remove {
                    dst_providers.remove(&k);
                    changed = true;
                }

                for (provider_name, src_provider) in src.iter() {
                    if !dst_providers.contains_key(provider_name) {
                        dst_providers.insert(provider_name.clone(), src_provider.clone());
                        changed = true;
                    }
                }

                // 2. 同步存在的 provider 的 baseUrl/apiKey/api + 清理已删除的 models
                for (provider_name, src_provider) in src.iter() {
                    if let Some(dst_provider) = dst_providers.get_mut(provider_name) {
                        if let Some(dst_obj) = dst_provider.as_object_mut() {
                            // 同步连接信息
                            for field in ["baseUrl", "apiKey", "api"] {
                                if let Some(src_val) =
                                    src_provider.get(field).and_then(|v| v.as_str())
                                {
                                    if dst_obj.get(field).and_then(|v| v.as_str()) != Some(src_val)
                                    {
                                        dst_obj.insert(
                                            field.to_string(),
                                            Value::String(src_val.to_string()),
                                        );
                                        changed = true;
                                    }
                                }
                            }
                            // 注意：不删除 agent models.json 中用户手动添加的模型。
                            // 只同步连接信息（baseUrl/apiKey/api），保留用户通过 CLI
                            // 或手动编辑添加的自定义模型。
                        }
                    }
                }
            }
        }

        if changed {
            if let Ok(new_json) = serde_json::to_string_pretty(&models_json) {
                let _ = fs::write(&models_path, new_json);
            }
        }
    }
}

/// 检测配置中是否包含 UI 专属字段
fn has_ui_fields(val: &Value) -> bool {
    if let Some(obj) = val.as_object() {
        for key in &[
            "current",
            "latest",
            "recommended",
            "update_available",
            "latest_update_available",
            "is_recommended",
            "ahead_of_recommended",
            "panel_version",
            "source",
            "qqbot",
            "profiles",
        ] {
            if obj.contains_key(*key) {
                return true;
            }
        }
        if obj
            .get("auth")
            .and_then(|v| v.as_object())
            .map(|auth| auth.contains_key("profiles"))
            .unwrap_or(false)
        {
            return true;
        }
        if obj
            .get("agents")
            .and_then(|v| v.as_object())
            .map(|agents| agents.contains_key("profiles"))
            .unwrap_or(false)
        {
            return true;
        }
        if let Some(models_val) = obj.get("models") {
            if let Some(models_obj) = models_val.as_object() {
                if let Some(providers_val) = models_obj.get("providers") {
                    if let Some(providers_obj) = providers_val.as_object() {
                        for (_provider_name, provider_val) in providers_obj.iter() {
                            if let Some(provider_obj) = provider_val.as_object() {
                                if let Some(Value::Array(arr)) = provider_obj.get("models") {
                                    for model in arr.iter() {
                                        if let Some(mobj) = model.as_object() {
                                            if mobj.contains_key("lastTestAt")
                                                || mobj.contains_key("latency")
                                                || mobj.contains_key("testStatus")
                                                || mobj.contains_key("testError")
                                            {
                                                return true;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

/// 清理 ClawPanel 内部字段，避免污染 openclaw.json 导致 Gateway 启动失败
/// Issue #89: version info 字段被写入 openclaw.json → Unknown config keys
/// Issue #127: 增强清理逻辑，保留 OpenClaw 合法的配置字段
///
/// 保留的合法配置字段（不清理）：
/// - `browser.*` - OpenClaw browser profiles 配置（如 browser.profiles）
/// - `agents.list` - OpenClaw agent list 配置
/// - 其他 OpenClaw schema 定义的字段
///
/// 清理的 UI 专属字段：
/// - 根层级：current, latest, update_available 等版本信息
/// - models.providers 中每个 model 的测试状态：lastTestAt, latency, testStatus, testError
fn strip_ui_fields(mut val: Value) -> Value {
    if let Some(obj) = val.as_object_mut() {
        // 清理根层级 ClawPanel 内部字段（version info 等）
        // 注意：保留 browser.* 和 agents.list，这些是 OpenClaw 合法的配置字段
        for key in &[
            "current",
            "latest",
            "recommended",
            "update_available",
            "latest_update_available",
            "is_recommended",
            "ahead_of_recommended",
            "panel_version",
            "source",
            // 渠道插件别名：OpenClaw schema 不承认 qqbot 作为根键（应写在 channels.qqbot）
            "qqbot",
            "profiles",
        ] {
            obj.remove(*key);
        }
        if let Some(auth_val) = obj.get_mut("auth") {
            if let Some(auth_obj) = auth_val.as_object_mut() {
                auth_obj.remove("profiles");
            }
        }
        // 处理 models.providers.xxx.models 结构
        if let Some(models_val) = obj.get_mut("models") {
            if let Some(models_obj) = models_val.as_object_mut() {
                if let Some(providers_val) = models_obj.get_mut("providers") {
                    if let Some(providers_obj) = providers_val.as_object_mut() {
                        for (_provider_name, provider_val) in providers_obj.iter_mut() {
                            if let Some(provider_obj) = provider_val.as_object_mut() {
                                if let Some(Value::Array(arr)) = provider_obj.get_mut("models") {
                                    for model in arr.iter_mut() {
                                        if let Some(mobj) = model.as_object_mut() {
                                            mobj.remove("lastTestAt");
                                            mobj.remove("latency");
                                            mobj.remove("testStatus");
                                            mobj.remove("testError");
                                            if !mobj.contains_key("name") {
                                                if let Some(id) =
                                                    mobj.get("id").and_then(|v| v.as_str())
                                                {
                                                    mobj.insert(
                                                        "name".into(),
                                                        Value::String(id.to_string()),
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        // 递归处理 agents 数组中的元素（保留 agents.list 等合法字段）
        if let Some(agents_val) = obj.get_mut("agents") {
            if let Some(agents_obj) = agents_val.as_object_mut() {
                agents_obj.remove("profiles");
                // 保留 agents 子字段不做修改
                // 只清理 agents 数组中的元素（如果有 UI 字段）
                if let Some(Value::Array(arr)) = agents_obj.get_mut("list") {
                    for agent in arr.iter_mut() {
                        if let Some(agent_obj) = agent.as_object_mut() {
                            // 清理 agent 中的 UI 字段，但保留 profiles
                            agent_obj.remove("current");
                            agent_obj.remove("latest");
                            agent_obj.remove("update_available");
                        }
                    }
                }
            }
        }
    }
    val
}

#[tauri::command]
pub fn read_mcp_config() -> Result<Value, String> {
    let path = super::openclaw_dir().join("mcp.json");
    if !path.exists() {
        return Ok(Value::Object(Default::default()));
    }
    let content = fs::read_to_string(&path).map_err(|e| format!("读取 MCP 配置失败: {e}"))?;
    serde_json::from_str(&content).map_err(|e| format!("解析 JSON 失败: {e}"))
}

#[tauri::command]
pub fn write_mcp_config(config: Value) -> Result<(), String> {
    let path = super::openclaw_dir().join("mcp.json");
    let json = serde_json::to_string_pretty(&config).map_err(|e| format!("序列化失败: {e}"))?;
    fs::write(&path, json).map_err(|e| format!("写入失败: {e}"))
}

/// 获取本地安装的 openclaw 版本号（异步版本）
/// macOS: 优先从 npm 包的 package.json 读取（含完整后缀），fallback 到 CLI
/// Windows/Linux: 优先读文件系统，fallback 到 CLI
/// 获取 OpenClaw 运行时状态摘要（openclaw status --json）
/// 包含 runtimeVersion、会话列表（含 token 用量、fastMode 等标签）

/// 获取指定源的所有可用版本列表（从 npm registry 查询）
/// 执行 npm 全局安装/升级/降级 openclaw（后台执行，通过 event 推送进度）

#[tauri::command]
pub fn init_openclaw_config() -> Result<Value, String> {
    let dir = super::openclaw_dir();
    let config_path = dir.join("openclaw.json");
    let backup_path = dir.join("openclaw.json.bak");
    let mut result = serde_json::Map::new();

    if config_path.exists() {
        result.insert("created".into(), Value::Bool(false));
        result.insert("message".into(), Value::String("配置文件已存在".into()));
        return Ok(Value::Object(result));
    }

    // 确保目录存在
    if !dir.exists() {
        std::fs::create_dir_all(&dir).map_err(|e| format!("创建目录失败: {e}"))?;
    }

    if backup_path.exists() {
        let backup_content =
            std::fs::read_to_string(&backup_path).map_err(|e| format!("读取配置备份失败: {e}"))?;
        serde_json::from_str::<Value>(&backup_content)
            .map_err(|e| format!("配置备份损坏，无法恢复: {e}"))?;
        std::fs::write(&config_path, backup_content)
            .map_err(|e| format!("恢复配置备份失败: {e}"))?;

        result.insert("created".into(), Value::Bool(false));
        result.insert("restored".into(), Value::Bool(true));
        result.insert(
            "message".into(),
            Value::String("已从 openclaw.json.bak 恢复配置文件".into()),
        );
        return Ok(Value::Object(result));
    }

    let default_config = strip_ui_fields(normalize_calibrated_config(build_calibration_baseline()));

    let content =
        serde_json::to_string_pretty(&default_config).map_err(|e| format!("序列化失败: {e}"))?;
    std::fs::write(&config_path, content).map_err(|e| format!("写入失败: {e}"))?;

    result.insert("created".into(), Value::Bool(true));
    result.insert("restored".into(), Value::Bool(false));
    result.insert("message".into(), Value::String("配置文件已创建".into()));
    Ok(Value::Object(result))
}

/// 检测 Node.js 是否已安装，返回版本号和检测到的路径
#[tauri::command]
pub fn write_env_file(path: String, config: String) -> Result<(), String> {
    let expanded = if let Some(stripped) = path.strip_prefix("~/") {
        dirs::home_dir().unwrap_or_default().join(stripped)
    } else {
        PathBuf::from(&path)
    };

    // 安全限制：只允许写入 ~/.openclaw/ 目录下的文件
    let openclaw_base = super::openclaw_dir();
    if !expanded.starts_with(&openclaw_base) {
        return Err(format!(
            "只允许写入 {} 目录下的文件",
            openclaw_base.display()
        ));
    }

    if let Some(parent) = expanded.parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::write(&expanded, &config).map_err(|e| format!("写入 .env 失败: {e}"))
}

// ===== 备份管理 =====

#[tauri::command]
pub fn list_backups() -> Result<Value, String> {
    let dir = backups_dir();
    if !dir.exists() {
        return Ok(Value::Array(vec![]));
    }
    let mut backups: Vec<Value> = vec![];
    let entries = fs::read_dir(&dir).map_err(|e| format!("读取备份目录失败: {e}"))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let meta = fs::metadata(&path).ok();
        let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
        // macOS 支持 created()，fallback 到 modified()
        let created = meta
            .and_then(|m| m.created().ok().or_else(|| m.modified().ok()))
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let mut obj = serde_json::Map::new();
        obj.insert("name".into(), Value::String(name));
        obj.insert("size".into(), Value::Number(size.into()));
        obj.insert("created_at".into(), Value::Number(created.into()));
        backups.push(Value::Object(obj));
    }
    // 按时间倒序
    backups.sort_by(|a, b| {
        let ta = a.get("created_at").and_then(|v| v.as_u64()).unwrap_or(0);
        let tb = b.get("created_at").and_then(|v| v.as_u64()).unwrap_or(0);
        tb.cmp(&ta)
    });
    Ok(Value::Array(backups))
}

#[tauri::command]
pub fn create_backup() -> Result<Value, String> {
    let dir = backups_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("创建备份目录失败: {e}"))?;

    let src = super::openclaw_dir().join("openclaw.json");
    if !src.exists() {
        return Err("openclaw.json 不存在".into());
    }

    let now = chrono::Local::now();
    let name = format!("openclaw-{}.json", now.format("%Y%m%d-%H%M%S"));
    let dest = dir.join(&name);
    fs::copy(&src, &dest).map_err(|e| format!("备份失败: {e}"))?;

    let size = fs::metadata(&dest).map(|m| m.len()).unwrap_or(0);
    let mut obj = serde_json::Map::new();
    obj.insert("name".into(), Value::String(name));
    obj.insert("size".into(), Value::Number(size.into()));
    Ok(Value::Object(obj))
}

/// 检查备份文件名是否安全
fn is_unsafe_backup_name(name: &str) -> bool {
    name.contains("..") || name.contains('/') || name.contains('\\')
}

#[tauri::command]
pub fn restore_backup(name: String) -> Result<(), String> {
    if is_unsafe_backup_name(&name) {
        return Err("非法文件名".into());
    }
    let backup_path = backups_dir().join(&name);
    if !backup_path.exists() {
        return Err(format!("备份文件不存在: {name}"));
    }
    let target = super::openclaw_dir().join("openclaw.json");

    // 恢复前先自动备份当前配置
    if target.exists() {
        let _ = create_backup();
    }

    fs::copy(&backup_path, &target).map_err(|e| format!("恢复失败: {e}"))?;
    Ok(())
}

#[tauri::command]
pub fn delete_backup(name: String) -> Result<(), String> {
    if is_unsafe_backup_name(&name) {
        return Err("非法文件名".into());
    }
    let path = backups_dir().join(&name);
    if !path.exists() {
        return Err(format!("备份文件不存在: {name}"));
    }
    fs::remove_file(&path).map_err(|e| format!("删除失败: {e}"))
}

/// 获取当前用户 UID（macOS/Linux 用 id -u，Windows 返回 0）
#[allow(dead_code)]
fn normalize_base_url(raw: &str) -> String {
    let mut base = raw.trim_end_matches('/').to_string();
    for suffix in &[
        "/api/chat",
        "/api/generate",
        "/api/tags",
        "/api",
        "/chat/completions",
        "/completions",
        "/responses",
        "/messages",
        "/models",
    ] {
        if base.ends_with(suffix) {
            base.truncate(base.len() - suffix.len());
            break;
        }
    }
    base = base.trim_end_matches('/').to_string();
    if base.ends_with(":11434") {
        return format!("{base}/v1");
    }
    base
}

fn normalize_model_api_type(raw: &str) -> &'static str {
    match raw.trim() {
        "anthropic" | "anthropic-messages" => "anthropic-messages",
        "google-gemini" | "google-generative-ai" => "google-gemini",
        "openai" | "openai-completions" | "openai-responses" | "" => "openai-completions",
        _ => "openai-completions",
    }
}

fn normalize_base_url_for_api(raw: &str, api_type: &str) -> String {
    let mut base = normalize_base_url(raw);
    match normalize_model_api_type(api_type) {
        "anthropic-messages" => {
            if !base.ends_with("/v1") {
                base.push_str("/v1");
            }
            base
        }
        "google-gemini" => base,
        _ => {
            // 不再强制追加 /v1，尊重用户填写的 URL（火山引擎等第三方用 /v3 等路径）
            // 仅 Ollama (端口 11434) 自动补 /v1
            base
        }
    }
}

fn is_valid_env_key(key: &str) -> bool {
    let mut chars = key.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first == '_' || first.is_ascii_alphabetic()) {
        return false;
    }
    chars.all(|c| c == '_' || c.is_ascii_alphanumeric())
}

fn model_api_key_env_ref(raw: &str) -> Result<Option<String>, String> {
    let value = raw.trim();
    if value.starts_with("${") && value.ends_with('}') {
        let key = &value[2..value.len() - 1];
        if is_valid_env_key(key) {
            return Ok(Some(key.to_string()));
        }
        return Err(format!("无效的环境变量引用: {value}"));
    }
    if let Some(key) = value.strip_prefix('$') {
        if !key.is_empty() && is_valid_env_key(key) {
            return Ok(Some(key.to_string()));
        }
    }
    Ok(None)
}

fn parse_dotenv_line(line: &str) -> Option<(String, String)> {
    let line = line.trim().trim_start_matches('\u{feff}');
    if line.is_empty() || line.starts_with('#') {
        return None;
    }
    let line = line.strip_prefix("export ").unwrap_or(line).trim();
    let (key, value) = line.split_once('=')?;
    let key = key.trim();
    if !is_valid_env_key(key) {
        return None;
    }
    let mut value = value.trim().to_string();
    if value.len() >= 2 {
        let bytes = value.as_bytes();
        if (bytes[0] == b'"' && bytes[value.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[value.len() - 1] == b'\'')
        {
            value = value[1..value.len() - 1].to_string();
        }
    }
    Some((key.to_string(), value))
}

fn model_env_values() -> HashMap<String, String> {
    let mut values = HashMap::new();
    if let Ok(cfg) = load_openclaw_json() {
        if let Some(env) = cfg.get("env").and_then(|v| v.as_object()) {
            for (key, value) in env {
                if !is_valid_env_key(key) {
                    continue;
                }
                if let Some(s) = value.as_str() {
                    values.insert(key.clone(), s.to_string());
                } else if value.is_number() || value.is_boolean() {
                    values.insert(key.clone(), value.to_string());
                }
            }
        }
    }
    let env_path = super::openclaw_dir().join(".env");
    if let Ok(content) = fs::read_to_string(env_path) {
        for line in content.lines() {
            if let Some((key, value)) = parse_dotenv_line(line) {
                values.entry(key).or_insert(value);
            }
        }
    }
    values
}

fn resolve_model_api_key(api_key: &str) -> Result<String, String> {
    let Some(key) = model_api_key_env_ref(api_key)? else {
        return Ok(api_key.to_string());
    };
    let values = model_env_values();
    if let Some(value) = values.get(&key).filter(|v| !v.is_empty()) {
        return Ok(value.clone());
    }
    if let Ok(value) = std::env::var(&key) {
        if !value.is_empty() {
            return Ok(value);
        }
    }
    Err(format!(
        "API Key 引用了环境变量 {key}，但未在 openclaw.json env、~/.openclaw/.env 或当前进程环境中找到"
    ))
}

fn extract_error_message(text: &str, status: reqwest::StatusCode) -> String {
    serde_json::from_str::<serde_json::Value>(text)
        .ok()
        .and_then(|v| {
            v.get("error")
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .map(String::from)
                .or_else(|| v.get("message").and_then(|m| m.as_str()).map(String::from))
        })
        .unwrap_or_else(|| format!("HTTP {status}"))
}

/// 测试模型连通性：向 provider 发送一个简单的 chat completion 请求
#[tauri::command]
pub async fn test_model(
    base_url: String,
    api_key: String,
    model_id: String,
    api_type: Option<String>,
) -> Result<String, String> {
    let api_type = normalize_model_api_type(api_type.as_deref().unwrap_or("openai-completions"));
    let base = normalize_base_url_for_api(&base_url, api_type);
    let api_key = resolve_model_api_key(&api_key)?;

    let client =
        crate::commands::build_http_client_no_proxy(std::time::Duration::from_secs(30), None)
            .map_err(|e| format!("创建 HTTP 客户端失败: {e}"))?;

    let resp = match api_type {
        "anthropic-messages" => {
            let url = format!("{}/messages", base);
            let body = json!({
                "model": model_id,
                "messages": [{"role": "user", "content": "Hi"}],
                "max_tokens": 16,
            });
            let mut req = client
                .post(&url)
                .header("anthropic-version", "2023-06-01")
                .json(&body);
            if !api_key.is_empty() {
                req = req.header("x-api-key", api_key.clone());
            }
            req.send()
        }
        "google-gemini" => {
            let url = format!(
                "{}/models/{}:generateContent?key={}",
                base, model_id, api_key
            );
            let body = json!({
                "contents": [{"role": "user", "parts": [{"text": "Hi"}]}]
            });
            client.post(&url).json(&body).send()
        }
        _ => {
            let url = format!("{}/chat/completions", base);
            let body = json!({
                "model": model_id,
                "messages": [{"role": "user", "content": "Hi"}],
                "max_tokens": 16,
                "stream": false
            });
            let mut req = client.post(&url).json(&body);
            if !api_key.is_empty() {
                req = req.header("Authorization", format!("Bearer {api_key}"));
            }
            req.send()
        }
    }
    .await
    .map_err(|e| {
        if e.is_timeout() {
            "请求超时 (30s)".to_string()
        } else if e.is_connect() {
            format!("连接失败: {e}")
        } else {
            format!("请求失败: {e}")
        }
    })?;

    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();

    if !status.is_success() {
        let msg = extract_error_message(&text, status);
        // 401/403 是认证错误，一定要报错
        if status.as_u16() == 401 || status.as_u16() == 403 {
            return Err(msg);
        }
        // 其他错误（400/422/429 等）：服务器可达、认证通过，仅模型对简单测试不兼容
        // 返回成功但带提示和完整错误信息，方便前端展示
        return Ok(format!(
            "⚠ 连接正常（API 返回 {status}，部分模型对简单测试不兼容，不影响实际使用）\n{msg}"
        ));
    }

    // 提取回复内容（兼容多种响应格式）
    let reply = serde_json::from_str::<serde_json::Value>(&text)
        .ok()
        .and_then(|v| {
            if let Some(arr) = v.get("content").and_then(|c| c.as_array()) {
                let text = arr
                    .iter()
                    .filter(|b| b.get("type").and_then(|t| t.as_str()) == Some("text"))
                    .filter_map(|b| b.get("text").and_then(|t| t.as_str()))
                    .collect::<Vec<_>>()
                    .join("");
                if !text.is_empty() {
                    return Some(text);
                }
            }
            if let Some(t) = v
                .get("candidates")
                .and_then(|c| c.get(0))
                .and_then(|c| c.get("content"))
                .and_then(|c| c.get("parts"))
                .and_then(|p| p.get(0))
                .and_then(|p| p.get("text"))
                .and_then(|t| t.as_str())
                .filter(|s| !s.is_empty())
            {
                return Some(t.to_string());
            }
            // 标准 OpenAI 格式: choices[0].message.content
            if let Some(msg) = v
                .get("choices")
                .and_then(|c| c.get(0))
                .and_then(|c| c.get("message"))
            {
                let content = msg.get("content").and_then(|c| c.as_str()).unwrap_or("");
                if !content.is_empty() {
                    return Some(content.to_string());
                }
                // reasoning 模型
                if let Some(rc) = msg
                    .get("reasoning_content")
                    .and_then(|c| c.as_str())
                    .filter(|s| !s.is_empty())
                {
                    return Some(format!("[reasoning] {rc}"));
                }
            }
            // DashScope 格式: output.text
            if let Some(t) = v
                .get("output")
                .and_then(|o| o.get("text"))
                .and_then(|t| t.as_str())
                .filter(|s| !s.is_empty())
            {
                return Some(t.to_string());
            }
            None
        })
        .unwrap_or_else(|| "（模型已响应）".into());

    Ok(reply)
}

/// 从 SSE 流文本中累积 OpenAI 风格的 delta.content / delta.reasoning_content
/// 格式示例：
///   data: {"choices":[{"delta":{"content":"你好"}}]}
///   data: {"choices":[{"delta":{"content":"，"}}]}
///   data: [DONE]
fn extract_sse_reply(text: &str) -> String {
    let mut content = String::new();
    let mut reasoning = String::new();
    let mut saw_data_line = false;
    for line in text.lines() {
        let data = if let Some(rest) = line.strip_prefix("data: ") {
            rest
        } else if let Some(rest) = line.strip_prefix("data:") {
            rest
        } else {
            continue;
        };
        saw_data_line = true;
        let data = data.trim();
        if data.is_empty() || data == "[DONE]" {
            continue;
        }
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(data) {
            // OpenAI / 兼容后端：choices[0].delta.content
            let delta = v
                .get("choices")
                .and_then(|c| c.get(0))
                .and_then(|c| c.get("delta"));
            if let Some(d) = delta {
                if let Some(c) = d.get("content").and_then(|c| c.as_str()) {
                    content.push_str(c);
                }
                if let Some(rc) = d.get("reasoning_content").and_then(|c| c.as_str()) {
                    reasoning.push_str(rc);
                }
            }
            // Anthropic streaming: {"type":"content_block_delta","delta":{"type":"text_delta","text":"..."}}
            if v.get("type").and_then(|t| t.as_str()) == Some("content_block_delta") {
                if let Some(c) = v
                    .get("delta")
                    .and_then(|d| d.get("text"))
                    .and_then(|t| t.as_str())
                {
                    content.push_str(c);
                }
            }
        }
    }
    if !saw_data_line {
        return String::new();
    }
    if !content.is_empty() {
        content
    } else if !reasoning.is_empty() {
        format!("[reasoning] {reasoning}")
    } else {
        String::new()
    }
}

/// 从单个 JSON 响应中提取 reply（兼容 OpenAI / Anthropic / Gemini / DashScope 非流式）
fn extract_single_json_reply(text: &str) -> String {
    serde_json::from_str::<serde_json::Value>(text)
        .ok()
        .and_then(|v| {
            if let Some(arr) = v.get("content").and_then(|c| c.as_array()) {
                let text = arr
                    .iter()
                    .filter(|b| b.get("type").and_then(|t| t.as_str()) == Some("text"))
                    .filter_map(|b| b.get("text").and_then(|t| t.as_str()))
                    .collect::<Vec<_>>()
                    .join("");
                if !text.is_empty() {
                    return Some(text);
                }
            }
            if let Some(t) = v
                .get("candidates")
                .and_then(|c| c.get(0))
                .and_then(|c| c.get("content"))
                .and_then(|c| c.get("parts"))
                .and_then(|p| p.get(0))
                .and_then(|p| p.get("text"))
                .and_then(|t| t.as_str())
                .filter(|s| !s.is_empty())
            {
                return Some(t.to_string());
            }
            if let Some(msg) = v
                .get("choices")
                .and_then(|c| c.get(0))
                .and_then(|c| c.get("message"))
            {
                let content = msg.get("content").and_then(|c| c.as_str()).unwrap_or("");
                if !content.is_empty() {
                    return Some(content.to_string());
                }
                if let Some(rc) = msg
                    .get("reasoning_content")
                    .and_then(|c| c.as_str())
                    .filter(|s| !s.is_empty())
                {
                    return Some(format!("[reasoning] {rc}"));
                }
            }
            if let Some(t) = v
                .get("output")
                .and_then(|o| o.get("text"))
                .and_then(|t| t.as_str())
                .filter(|s| !s.is_empty())
            {
                return Some(t.to_string());
            }
            None
        })
        .unwrap_or_default()
}

/// 测试模型（详细版 #Compat-1）：返回完整 req/resp 信息，供前端 debug 面板展示
///
/// 相比 test_model：
/// - 不会因 400/422/429 等吞掉错误返回"连接正常"，一律如实回传 status + body
/// - 返回结构化 JSON：success/status/req_url/req_body/resp_body/reply/error/elapsed_ms/used_api
/// - 前端拿到后可以直接渲染 debug 面板，无需在 webview 里走外部 fetch（规避 status 0）
/// - OpenAI 兼容路径使用 stream:true（绕开某些 new-api 后端的 non-streaming bug，
///   并与真实对话行为一致）
#[tauri::command]
pub async fn test_model_verbose(
    base_url: String,
    api_key: String,
    model_id: String,
    api_type: Option<String>,
) -> Result<serde_json::Value, String> {
    use std::time::Instant;
    let api_type_norm =
        normalize_model_api_type(api_type.as_deref().unwrap_or("openai-completions"));
    let base = normalize_base_url_for_api(&base_url, api_type_norm);
    let api_key = resolve_model_api_key(&api_key)?;
    let start = Instant::now();

    let client =
        crate::commands::build_http_client_no_proxy(std::time::Duration::from_secs(30), None)
            .map_err(|e| format!("创建 HTTP 客户端失败: {e}"))?;

    // 关键：显式 Accept-Encoding: identity 禁止响应压缩，避免：
    // - reqwest 未启用 brotli feature 时，provider 返回 Content-Encoding: br 导致 text() 失败
    // - 某些 CDN 会根据默认 UA 自动压缩响应
    // 测试请求的响应体很小（几百字节），不压缩的性能损失可忽略
    let (used_api, req_url, req_body_json, req_builder) = match api_type_norm {
        "anthropic-messages" => {
            let url = format!("{}/messages", base);
            let body = json!({
                "model": model_id,
                "messages": [{"role": "user", "content": "你好，请用一句话回复"}],
                "max_tokens": 200,
            });
            let mut req = client
                .post(&url)
                .header("anthropic-version", "2023-06-01")
                .header("Accept-Encoding", "identity")
                .json(&body);
            if !api_key.is_empty() {
                req = req.header("x-api-key", api_key.clone());
            }
            ("Anthropic Messages", url, body, req)
        }
        "google-gemini" => {
            let url_display = format!("{}/models/{}:generateContent?key=***", base, model_id);
            let url_real = format!(
                "{}/models/{}:generateContent?key={}",
                base, model_id, api_key
            );
            let body = json!({
                "contents": [{"role": "user", "parts": [{"text": "你好，请用一句话回复"}]}]
            });
            let req = client
                .post(&url_real)
                .header("Accept-Encoding", "identity")
                .json(&body);
            ("Gemini", url_display, body, req)
        }
        _ => {
            let url = format!("{}/chat/completions", base);
            // 关键：测试请求用 stream: true 而非 stream: false
            // 理由：部分兼容网关的 non-streaming 分支对某些模型会返回 200 + 空 body，
            // 而 streaming 分支是真实对话路径，所有 provider 都稳定支持。
            // 测试走 stream: true + SSE 累积，行为与真实对话一致。
            let body = json!({
                "model": model_id,
                "messages": [{"role": "user", "content": "你好，请用一句话回复"}],
                "max_tokens": 200,
                "stream": true
            });
            let mut req = client
                .post(&url)
                .header("Accept-Encoding", "identity")
                .header("Accept", "text/event-stream")
                .json(&body);
            if !api_key.is_empty() {
                req = req.header("Authorization", format!("Bearer {api_key}"));
            }
            ("Chat Completions (SSE)", url, body, req)
        }
    };

    let resp_result = req_builder.send().await;
    let elapsed_ms = start.elapsed().as_millis() as u64;

    let resp = match resp_result {
        Ok(r) => r,
        Err(e) => {
            let error = if e.is_timeout() {
                "请求超时 (30s)".to_string()
            } else if e.is_connect() {
                format!("连接失败: {e}")
            } else {
                format!("请求失败: {e}")
            };
            return Ok(json!({
                "success": false,
                "status": 0,
                "reqUrl": req_url,
                "reqBody": req_body_json,
                "respBody": "",
                "reply": "",
                "error": error,
                "elapsedMs": elapsed_ms,
                "usedApi": used_api,
            }));
        }
    };

    let status = resp.status();
    let status_code = status.as_u16();

    // 先抓取响应头（text() 会消耗 resp）—— 这是关键诊断信息：
    // Content-Encoding 告诉我们是否压缩、是 br/gzip/zstd 还是啥
    // Content-Type 告诉我们是否是 JSON / text
    // Content-Length 告诉我们服务器声明的响应体大小
    let resp_headers = {
        let mut map = serde_json::Map::new();
        for (k, v) in resp.headers().iter() {
            map.insert(
                k.to_string(),
                serde_json::Value::String(v.to_str().unwrap_or("<non-utf8>").to_string()),
            );
        }
        serde_json::Value::Object(map)
    };

    // 读取响应体：改用 bytes() 拿原始字节（reqwest 会按 Content-Encoding 自动解压），
    // 然后自己做 UTF-8 decode。这样：
    // 1. 失败时能给出更精确的错误分类（网络错误 vs 解压错误 vs UTF-8 错误）
    // 2. UTF-8 失败时能 fallback 到 hex dump + lossy string，方便诊断
    let bytes = match resp.bytes().await {
        Ok(b) => b,
        Err(e) => {
            let mut err_chain = format!("{e}");
            let mut src: Option<&dyn std::error::Error> = std::error::Error::source(&e);
            while let Some(s) = src {
                err_chain.push_str(&format!(" → {s}"));
                src = std::error::Error::source(s);
            }
            return Ok(json!({
                "success": false,
                "status": status_code,
                "reqUrl": req_url,
                "reqBody": req_body_json,
                "respHeaders": resp_headers,
                "respBody": "",
                "respRawHex": "",
                "respByteCount": 0,
                "reply": "",
                "error": format!("读取响应字节失败: {err_chain}"),
                "elapsedMs": elapsed_ms,
                "usedApi": used_api,
            }));
        }
    };
    let byte_count = bytes.len();

    // 前 200 字节的 hex dump（无论成功失败都附上，方便调试）
    let hex_preview = bytes
        .iter()
        .take(200)
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join(" ");

    // 尝试严格 UTF-8 decode；失败时 fallback 到 lossy 并在 error 里带上诊断
    let text = match std::str::from_utf8(&bytes) {
        Ok(s) => s.to_string(),
        Err(e) => {
            let lossy = String::from_utf8_lossy(&bytes).into_owned();
            let ascii_preview: String = bytes
                .iter()
                .take(80)
                .map(|&b| {
                    if (0x20..=0x7e).contains(&b) {
                        b as char
                    } else {
                        '.'
                    }
                })
                .collect();
            return Ok(json!({
                "success": false,
                "status": status_code,
                "reqUrl": req_url,
                "reqBody": req_body_json,
                "respHeaders": resp_headers,
                "respBody": lossy,
                "respRawHex": hex_preview,
                "respByteCount": byte_count,
                "reply": "",
                "error": format!("响应体 UTF-8 解码失败: {e} | 字节数={byte_count} | 前 80 字节 ASCII='{ascii_preview}'"),
                "elapsedMs": elapsed_ms,
                "usedApi": used_api,
            }));
        }
    };

    // 提取 reply 文本：同时兼容 SSE 流（stream:true）和单次 JSON（stream:false）
    // 优先尝试 SSE 解析（OpenAI 兼容路径现在用 stream:true），失败再回退到单 JSON
    let reply = {
        let sse_reply = extract_sse_reply(&text);
        if !sse_reply.is_empty() {
            sse_reply
        } else {
            extract_single_json_reply(&text)
        }
    };

    let success = status.is_success() && !reply.is_empty();
    let error = if !status.is_success() {
        Some(extract_error_message(&text, status))
    } else if reply.is_empty() {
        Some("API 已响应但未解析出内容".to_string())
    } else {
        None
    };

    Ok(json!({
        "success": success,
        "status": status_code,
        "reqUrl": req_url,
        "reqBody": req_body_json,
        "respHeaders": resp_headers,
        "respBody": text,
        "respRawHex": hex_preview,
        "respByteCount": byte_count,
        "reply": reply,
        "error": error,
        "elapsedMs": elapsed_ms,
        "usedApi": used_api,
    }))
}

/// 获取服务商的远程模型列表（调用 /models 接口）
#[tauri::command]
pub async fn list_remote_models(
    base_url: String,
    api_key: String,
    api_type: Option<String>,
) -> Result<Vec<String>, String> {
    let api_type = normalize_model_api_type(api_type.as_deref().unwrap_or("openai-completions"));
    let base = normalize_base_url_for_api(&base_url, api_type);
    let api_key = resolve_model_api_key(&api_key)?;

    let client =
        crate::commands::build_http_client_no_proxy(std::time::Duration::from_secs(15), None)
            .map_err(|e| format!("创建 HTTP 客户端失败: {e}"))?;

    let resp = match api_type {
        "anthropic-messages" => {
            let url = format!("{}/models", base);
            let mut req = client.get(&url).header("anthropic-version", "2023-06-01");
            if !api_key.is_empty() {
                req = req.header("x-api-key", api_key.clone());
            }
            req.send()
        }
        "google-gemini" => {
            let url = format!("{}/models?key={}", base, api_key);
            client.get(&url).send()
        }
        _ => {
            let url = format!("{}/models", base);
            let mut req = client.get(&url);
            if !api_key.is_empty() {
                req = req.header("Authorization", format!("Bearer {api_key}"));
            }
            req.send()
        }
    }
    .await
    .map_err(|e| {
        if e.is_timeout() {
            "请求超时 (15s)，该服务商可能不支持模型列表接口".to_string()
        } else if e.is_connect() {
            format!("连接失败，请检查接口地址是否正确: {e}")
        } else {
            format!("请求失败: {e}")
        }
    })?;

    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();

    if !status.is_success() {
        // 404/405/501 = 服务商不支持 /models 接口，给用户友好提示而非技术错误
        let code = status.as_u16();
        if code == 404 || code == 405 || code == 501 {
            return Err(
                "[NOT_SUPPORTED] 该服务商不支持自动获取模型列表，请手动输入模型 ID".to_string(),
            );
        }
        let msg = extract_error_message(&text, status);
        return Err(format!("获取模型列表失败: {msg}"));
    }

    // 解析 OpenAI / Anthropic / Gemini 格式的 /models 响应
    let ids = serde_json::from_str::<serde_json::Value>(&text)
        .ok()
        .map(|v| {
            let mut ids: Vec<String> = if let Some(data) = v.get("data").and_then(|d| d.as_array())
            {
                data.iter()
                    .filter_map(|m| m.get("id").and_then(|id| id.as_str()).map(String::from))
                    .collect()
            } else if let Some(data) = v.get("models").and_then(|d| d.as_array()) {
                data.iter()
                    .filter_map(|m| {
                        m.get("name")
                            .and_then(|id| id.as_str())
                            .map(|s| s.trim_start_matches("models/").to_string())
                    })
                    .collect()
            } else {
                vec![]
            };
            ids.sort();
            ids
        })
        .unwrap_or_default();

    if ids.is_empty() {
        return Err("该服务商返回了空的模型列表，可能不支持 /models 接口".to_string());
    }

    Ok(ids)
}

/// 安装 Gateway 服务（执行 openclaw gateway install）
/// 为 openclaw.json 中所有模型添加 input: ["text", "image"]，使 Gateway 识别模型支持图片输入
#[tauri::command]
pub fn patch_model_vision() -> Result<bool, String> {
    let path = super::openclaw_dir().join("openclaw.json");
    let content = fs::read_to_string(&path).map_err(|e| format!("读取配置失败: {e}"))?;
    let mut config: Value =
        serde_json::from_str(&content).map_err(|e| format!("解析 JSON 失败: {e}"))?;

    let vision_input = Value::Array(vec![
        Value::String("text".into()),
        Value::String("image".into()),
    ]);

    let mut changed = false;

    if let Some(obj) = config.as_object_mut() {
        if let Some(models_val) = obj.get_mut("models") {
            if let Some(models_obj) = models_val.as_object_mut() {
                if let Some(providers_val) = models_obj.get_mut("providers") {
                    if let Some(providers_obj) = providers_val.as_object_mut() {
                        for (_provider_name, provider_val) in providers_obj.iter_mut() {
                            if let Some(provider_obj) = provider_val.as_object_mut() {
                                if let Some(Value::Array(arr)) = provider_obj.get_mut("models") {
                                    for model in arr.iter_mut() {
                                        if let Some(mobj) = model.as_object_mut() {
                                            if !mobj.contains_key("input") {
                                                mobj.insert("input".into(), vision_input.clone());
                                                changed = true;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if changed {
        let bak = super::openclaw_dir().join("openclaw.json.bak");
        let _ = fs::copy(&path, &bak);
        let json = serde_json::to_string_pretty(&config).map_err(|e| format!("序列化失败: {e}"))?;
        fs::write(&path, json).map_err(|e| format!("写入失败: {e}"))?;
    }

    Ok(changed)
}

/// 检查 ClawPanel 自身是否有新版本（GitHub → Gitee 自动降级）
#[tauri::command]
pub async fn check_panel_update() -> Result<Value, String> {
    let client =
        crate::commands::build_http_client(std::time::Duration::from_secs(8), Some("ClawPanel"))
            .map_err(|e| format!("创建 HTTP 客户端失败: {e}"))?;

    // 先尝试 GitHub，失败后降级 Gitee
    let sources = [
        (
            "https://api.github.com/repos/qingchencloud/clawpanel/releases/latest",
            "https://github.com/qingchencloud/clawpanel/releases",
            "github",
        ),
        (
            "https://gitee.com/api/v5/repos/QtCodeCreators/clawpanel/releases/latest",
            "https://gitee.com/QtCodeCreators/clawpanel/releases",
            "gitee",
        ),
    ];

    let mut last_err = String::new();
    for (api_url, releases_url, source) in &sources {
        match client.get(*api_url).send().await {
            Ok(resp) if resp.status().is_success() => {
                let json: Value = resp
                    .json()
                    .await
                    .map_err(|e| format!("解析响应失败: {e}"))?;

                let tag = json
                    .get("tag_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .trim_start_matches('v')
                    .to_string();

                if tag.is_empty() {
                    last_err = format!("{source}: 未找到版本号");
                    continue;
                }

                let mut result = serde_json::Map::new();
                result.insert("latest".into(), Value::String(tag));
                result.insert(
                    "url".into(),
                    json.get("html_url")
                        .cloned()
                        .unwrap_or(Value::String(releases_url.to_string())),
                );
                result.insert("source".into(), Value::String(source.to_string()));
                result.insert(
                    "downloadUrl".into(),
                    Value::String("https://claw.qt.cool".into()),
                );
                return Ok(Value::Object(result));
            }
            Ok(resp) => {
                last_err = format!("{source}: HTTP {}", resp.status());
            }
            Err(e) => {
                last_err = format!("{source}: {e}");
            }
        }
    }

    Err(last_err)
}

// === 面板配置 (clawpanel.json) ===

/// 获取当前生效的 OpenClaw 配置目录路径
#[tauri::command]
pub fn get_openclaw_dir() -> Result<Value, String> {
    super::app_config::get_openclaw_dir()
}

#[tauri::command]
pub fn read_panel_config() -> Result<Value, String> {
    super::app_config::read_panel_config()
}

#[tauri::command]
pub fn write_panel_config(config: Value) -> Result<(), String> {
    super::app_config::write_panel_config(config)
}

/// 重启应用（用于设置变更后自动重启）
#[tauri::command]
pub async fn relaunch_app(app: tauri::AppHandle) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| format!("获取可执行文件路径失败: {e}"))?;
    std::process::Command::new(&exe)
        .spawn()
        .map_err(|e| format!("重启失败: {e}"))?;
    // 短暂延迟后退出当前进程
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    app.exit(0);
    Ok(())
}

/// 测试代理连通性：通过配置的代理访问指定 URL，返回状态码和耗时
#[tauri::command]
pub fn detect_legacy_config_migration() -> Result<Value, String> {
    super::app_config::detect_legacy_config_migration()
}

#[tauri::command]
pub fn apply_legacy_config_migration(action: String) -> Result<Value, String> {
    super::app_config::apply_legacy_config_migration(action)
}

#[tauri::command]
pub fn get_npm_registry() -> Result<String, String> {
    super::app_config::get_npm_registry()
}

#[tauri::command]
pub fn set_npm_registry(registry: String) -> Result<(), String> {
    super::app_config::set_npm_registry(registry)
}

/// 刷新 enhanced_path 缓存，使新设置的 Node.js 路径立即生效
#[tauri::command]
pub fn invalidate_path_cache() -> Result<(), String> {
    super::app_config::invalidate_path_cache()
}

use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Default)]
struct VersionPolicySource {
    recommended: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct VersionPolicyEntry {
    #[serde(default)]
    official: VersionPolicySource,
    #[serde(default)]
    chinese: VersionPolicySource,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Default)]
pub(crate) struct R2Config {
    #[serde(default)]
    #[serde(rename = "baseUrl")]
    pub(crate) base_url: Option<String>,
    #[serde(default)]
    pub(crate) enabled: bool,
}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct StandaloneConfig {
    #[serde(default)]
    #[serde(rename = "baseUrl")]
    pub(crate) base_url: Option<String>,
    #[serde(default)]
    pub(crate) enabled: bool,
}

#[derive(Debug, Deserialize, Default)]
struct VersionPolicy {
    #[serde(default)]
    standalone: StandaloneConfig,
    #[serde(default)]
    r2: R2Config,
    #[serde(default)]
    default: VersionPolicyEntry,
    #[serde(default)]
    panels: HashMap<String, VersionPolicyEntry>,
}

pub(crate) fn panel_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub(crate) fn npm_package_name(source: &str) -> &'static str {
    match source {
        "official" => "openclaw",
        _ => "@DeepAi助手/openclaw-zh",
    }
}

pub(crate) fn versions_match(cli_version: &str, recommended: &str) -> bool {
    if cli_version == recommended {
        return true;
    }
    base_version(cli_version) == base_version(recommended)
}

pub(crate) fn recommended_is_newer(recommended: &str, current: &str) -> bool {
    let recommended_parts = parse_version(&base_version(recommended));
    let current_parts = parse_version(&base_version(current));
    recommended_parts > current_parts
}

pub(crate) fn recommended_version_for(source: &str) -> Option<String> {
    let policy = load_version_policy();
    let panel_entry = find_panel_policy_entry(&policy, panel_version());
    match source {
        "official" => panel_entry
            .and_then(|entry| entry.official.recommended.clone())
            .or(policy.default.official.recommended),
        _ => panel_entry
            .and_then(|entry| entry.chinese.recommended.clone())
            .or(policy.default.chinese.recommended),
    }
}

#[allow(dead_code)]
pub(crate) fn r2_config() -> R2Config {
    load_version_policy().r2
}

pub(crate) fn standalone_config() -> StandaloneConfig {
    load_version_policy().standalone
}

fn load_version_policy() -> VersionPolicy {
    serde_json::from_str(include_str!("../../../openclaw-version-policy.json")).unwrap_or_default()
}

fn find_panel_policy_entry<'a>(
    policy: &'a VersionPolicy,
    current_version: &str,
) -> Option<&'a VersionPolicyEntry> {
    if let Some(entry) = policy.panels.get(current_version) {
        return Some(entry);
    }

    let current_parts = parse_version(current_version);
    if current_parts.len() < 2 {
        return None;
    }

    policy
        .panels
        .iter()
        .filter_map(|(version, entry)| {
            let parts = parse_version(version);
            if parts.len() < 2 {
                return None;
            }
            if parts[0] != current_parts[0] || parts[1] != current_parts[1] {
                return None;
            }
            if parts > current_parts {
                return None;
            }
            Some((parts, entry))
        })
        .max_by(|(left, _), (right, _)| left.cmp(right))
        .map(|(_, entry)| entry)
}

fn parse_version(value: &str) -> Vec<u32> {
    value
        .split(|c: char| !c.is_ascii_digit())
        .filter_map(|segment| segment.parse().ok())
        .collect()
}

fn base_version(value: &str) -> String {
    value.split('-').next().unwrap_or(value).to_string()
}

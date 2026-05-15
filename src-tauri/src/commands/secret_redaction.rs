use regex::{Captures, Regex};
use std::sync::LazyLock;

static BEARER_SECRET_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(authorization\s*[:=]\s*bearer\s+)([a-z0-9._~+/=-]{8,})")
        .expect("valid bearer secret redaction regex")
});

static URL_CREDENTIAL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b([a-z][a-z0-9+.-]*://)([^/\s:@]+):([^@\s/]+)@")
        .expect("valid url credential redaction regex")
});

static KEY_VALUE_SECRET_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?i)\b([a-z0-9_.-]*(?:api[_-]?key|auth[_-]?token|token|password|passwd|pwd|secret|access[_-]?key|refresh[_-]?token|client[_-]?secret)[a-z0-9_.-]*\s*[:=]\s*)("[^"]*"|'[^']*'|[^\s,;&]+)"#,
    )
    .expect("valid key-value secret redaction regex")
});

static WELL_KNOWN_TOKEN_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(sk-[a-zA-Z0-9_-]{12,}|ghp_[a-zA-Z0-9_]{12,}|github_pat_[a-zA-Z0-9_]{20,}|hf_[a-zA-Z0-9_]{12,}|xox[baprs]-[a-zA-Z0-9-]{12,})")
        .expect("valid well-known token redaction regex")
});

fn masked_value(raw: &str) -> &'static str {
    if raw.starts_with('"') && raw.ends_with('"') {
        "\"[REDACTED]\""
    } else if raw.starts_with('\'') && raw.ends_with('\'') {
        "'[REDACTED]'"
    } else {
        "[REDACTED]"
    }
}

pub(crate) fn redact_secrets(input: impl AsRef<str>) -> String {
    let input = input.as_ref();
    if input.is_empty() {
        return String::new();
    }

    let text = BEARER_SECRET_RE.replace_all(input, "${1}[REDACTED]");
    let text = URL_CREDENTIAL_RE.replace_all(&text, "${1}[REDACTED]@");
    let text = KEY_VALUE_SECRET_RE.replace_all(&text, |caps: &Captures| {
        format!("{}{}", &caps[1], masked_value(&caps[2]))
    });
    WELL_KNOWN_TOKEN_RE
        .replace_all(&text, "[REDACTED]")
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::redact_secrets;

    #[test]
    fn redacts_common_key_value_secrets() {
        let text = r#"token=abc123456 password: "secret-pass" api_key='key-12345'"#;
        let redacted = redact_secrets(text);

        assert!(redacted.contains("token=[REDACTED]"));
        assert!(redacted.contains(r#"password: "[REDACTED]""#));
        assert!(redacted.contains("api_key='[REDACTED]'"));
        assert!(!redacted.contains("abc123456"));
        assert!(!redacted.contains("secret-pass"));
        assert!(!redacted.contains("key-12345"));
    }

    #[test]
    fn redacts_bearer_headers_url_credentials_and_known_tokens() {
        let text = "Authorization: Bearer sk-testsecret0000 https://user:pass@example.com ghp_abcdefghijklmnop";
        let redacted = redact_secrets(text);

        assert!(redacted.contains("Authorization: Bearer [REDACTED]"));
        assert!(redacted.contains("https://[REDACTED]@example.com"));
        assert!(!redacted.contains("sk-testsecret0000"));
        assert!(!redacted.contains("user:pass"));
        assert!(!redacted.contains("ghp_abcdefghijklmnop"));
    }

    #[test]
    fn leaves_unrelated_text_readable() {
        let text = "gateway started on port 18789";
        assert_eq!(redact_secrets(text), text);
    }
}

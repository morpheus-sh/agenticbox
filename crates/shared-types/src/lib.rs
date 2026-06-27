use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type SessionId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub model_config: ModelConfig,
    pub permissions: PermissionSet,
    pub status: SessionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    pub name: String,
    pub model_config: ModelConfig,
    pub permissions: PermissionSet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionStatus {
    Creating,
    Running,
    Paused,
    Destroyed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub provider: String,
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            provider: "openai".into(),
            model: "gpt-4o".into(),
            api_key: None,
            base_url: None,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PermissionSet {
    pub terminal: bool,
    pub filesystem: FsPermission,
    pub browser: bool,
    pub network: NetworkPolicy,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FsPermission {
    #[default]
    Deny,
    ReadOnly,
    ReadWrite,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NetworkPolicy {
    #[default]
    Offline,
    Allowlist(Vec<String>),
    LocalhostOnly,
    Full,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub tool: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub call_id: String,
    pub output: serde_json::Value,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_results: Option<Vec<ToolResult>>,
}

// ── Network host matching (dependency-free, single source of truth) ───────
// Consumed by both `network-control` and `policy-engine`. The previous
// substring bug (`destination.contains(domain)`) existed because the same
// flawed logic was duplicated across two crates — this centralises it so it
// can only be fixed and tested in one place.

pub mod host {
    /// Extract the host from a URL-like string, handling `scheme://`,
    /// userinfo (`user:pass@`), ports, and bracketed IPv6 literals (`[::1]`).
    /// Pure string logic — no `url` crate dependency.
    pub fn extract_host(destination: &str) -> &str {
        // 1. strip scheme
        let after_scheme = match destination.find("://") {
            Some(i) => &destination[i + 3..],
            None => destination,
        };
        // 2. authority ends at the first path/query/fragment separator
        let authority_end = after_scheme
            .find(['/', '?', '#'])
            .unwrap_or(after_scheme.len());
        let authority = &after_scheme[..authority_end];
        // 3. drop userinfo (everything before the last '@')
        let host_port = authority.rsplit('@').next().unwrap_or(authority);
        // 4. keep bracketed IPv6 literal intact, else strip the port
        if let Some(rest) = host_port.strip_prefix('[') {
            if let Some(end) = rest.find(']') {
                return &host_port[..end + 1]; // e.g. "[::1]"
            }
        }
        match host_port.rfind(':') {
            Some(i) => &host_port[..i],
            None => host_port,
        }
    }

    /// True if `host` equals `allowed` or is a subdomain of `allowed`.
    /// Uses a `.suffix` match so lookalikes and path-spoofs are rejected:
    ///   - `evilgithub.com` does NOT match `github.com`
    ///   - `evil.com/github.com` does NOT match `github.com`
    ///   - `api.github.com` DOES match `github.com` (legit subdomain)
    pub fn host_matches_domain(host: &str, allowed: &str) -> bool {
        let host = host.to_ascii_lowercase();
        let allowed = allowed.to_ascii_lowercase();
        host == allowed || host.ends_with(&format!(".{allowed}"))
    }

    /// True if `host` is a loopback address.
    pub fn is_localhost(host: &str) -> bool {
        let h = host
            .trim_matches(|c| c == '[' || c == ']')
            .to_ascii_lowercase();
        h == "localhost" || h == "127.0.0.1" || h == "::1"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── host::extract_host / matching ──────────────────────

    #[test]
    fn host_extract_strips_scheme_port_path() {
        assert_eq!(
            host::extract_host("https://api.openai.com/v1/models"),
            "api.openai.com"
        );
        assert_eq!(host::extract_host("http://localhost:3000"), "localhost");
        assert_eq!(
            host::extract_host("https://github.com/repos/x"),
            "github.com"
        );
        assert_eq!(host::extract_host("127.0.0.1:8080"), "127.0.0.1");
    }

    #[test]
    fn host_extract_strips_userinfo() {
        assert_eq!(
            host::extract_host("https://user:pass@api.example.com/x"),
            "api.example.com"
        );
    }

    #[test]
    fn host_extract_no_scheme_bare_authority() {
        // path-spoof attempt: evil host with allowed domain in the path
        assert_eq!(host::extract_host("evil.com/github.com"), "evil.com");
        assert_eq!(host::extract_host("evil.com:443/github.com"), "evil.com");
    }

    #[test]
    fn host_match_exact_and_subdomain_only() {
        assert!(host::host_matches_domain("github.com", "github.com"));
        assert!(host::host_matches_domain("api.github.com", "github.com"));
        // lookalikes / spoofs must NOT match
        assert!(!host::host_matches_domain("evilgithub.com", "github.com"));
        assert!(!host::host_matches_domain("evil.com", "github.com"));
        assert!(!host::host_matches_domain("notgithub.com", "github.com"));
    }

    #[test]
    fn host_match_case_insensitive() {
        assert!(host::host_matches_domain("API.GitHub.com", "github.com"));
    }

    #[test]
    fn host_is_localhost() {
        assert!(host::is_localhost("localhost"));
        assert!(host::is_localhost("127.0.0.1"));
        assert!(host::is_localhost("[::1]"));
        assert!(!host::is_localhost("api.openai.com"));
        assert!(!host::is_localhost("evil-localhost.attacker.com"));
    }

    // ── ModelConfig ────────────────────────────────────────

    #[test]
    fn model_config_default_values() {
        let cfg = ModelConfig::default();
        assert_eq!(cfg.provider, "openai");
        assert_eq!(cfg.model, "gpt-4o");
        assert!(cfg.api_key.is_none());
        assert!(cfg.base_url.is_none());
    }

    #[test]
    fn model_config_serde_roundtrip() {
        let cfg = ModelConfig {
            provider: "anthropic".into(),
            model: "claude-sonnet-4-20250514".into(),
            api_key: Some("sk-test".into()),
            base_url: Some("https://api.anthropic.com".into()),
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let deserialized: ModelConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.provider, cfg.provider);
        assert_eq!(deserialized.model, cfg.model);
        assert_eq!(deserialized.api_key, cfg.api_key);
        assert_eq!(deserialized.base_url, cfg.base_url);
    }

    // ── FsPermission ───────────────────────────────────────

    #[test]
    fn fs_permission_default_is_deny() {
        let fs = FsPermission::default();
        assert!(matches!(fs, FsPermission::Deny));
    }

    #[test]
    fn fs_permission_serde_camel_case() {
        let json = serde_json::to_string(&FsPermission::ReadOnly).unwrap();
        assert_eq!(json, "\"readOnly\"");
        let json = serde_json::to_string(&FsPermission::ReadWrite).unwrap();
        assert_eq!(json, "\"readWrite\"");
        let json = serde_json::to_string(&FsPermission::Deny).unwrap();
        assert_eq!(json, "\"deny\"");
    }

    #[test]
    fn fs_permission_deserialize_camel_case() {
        let fs: FsPermission = serde_json::from_str("\"readOnly\"").unwrap();
        assert!(matches!(fs, FsPermission::ReadOnly));
        let fs: FsPermission = serde_json::from_str("\"readWrite\"").unwrap();
        assert!(matches!(fs, FsPermission::ReadWrite));
    }

    // ── NetworkPolicy ──────────────────────────────────────

    #[test]
    fn network_policy_default_is_offline() {
        let np = NetworkPolicy::default();
        assert!(matches!(np, NetworkPolicy::Offline));
    }

    #[test]
    fn network_policy_serde_camel_case() {
        let json = serde_json::to_string(&NetworkPolicy::Offline).unwrap();
        assert_eq!(json, "\"offline\"");
        let json = serde_json::to_string(&NetworkPolicy::Full).unwrap();
        assert_eq!(json, "\"full\"");
        let json = serde_json::to_string(&NetworkPolicy::LocalhostOnly).unwrap();
        assert_eq!(json, "\"localhostOnly\"");
    }

    #[test]
    fn network_policy_allowlist_serde_roundtrip() {
        let policy = NetworkPolicy::Allowlist(vec!["api.openai.com".into(), "github.com".into()]);
        let json = serde_json::to_string(&policy).unwrap();
        let deserialized: NetworkPolicy = serde_json::from_str(&json).unwrap();
        match deserialized {
            NetworkPolicy::Allowlist(domains) => {
                assert_eq!(domains.len(), 2);
                assert!(domains.contains(&"api.openai.com".to_string()));
                assert!(domains.contains(&"github.com".to_string()));
            }
            other => panic!("expected Allowlist, got {:?}", other),
        }
    }

    // ── PermissionSet ──────────────────────────────────────

    #[test]
    fn permission_set_default_values() {
        let ps = PermissionSet::default();
        assert!(!ps.terminal);
        assert!(matches!(ps.filesystem, FsPermission::Deny));
        assert!(!ps.browser);
        assert!(matches!(ps.network, NetworkPolicy::Offline));
    }

    #[test]
    fn permission_set_serde_roundtrip() {
        let ps = PermissionSet {
            terminal: true,
            filesystem: FsPermission::ReadWrite,
            browser: false,
            network: NetworkPolicy::Allowlist(vec!["api.openai.com".into()]),
        };
        let json = serde_json::to_string(&ps).unwrap();
        let deserialized: PermissionSet = serde_json::from_str(&json).unwrap();
        assert!(deserialized.terminal);
        assert!(matches!(deserialized.filesystem, FsPermission::ReadWrite));
        assert!(!deserialized.browser);
        match deserialized.network {
            NetworkPolicy::Allowlist(d) => assert_eq!(d, vec!["api.openai.com"]),
            other => panic!("expected Allowlist, got {:?}", other),
        }
    }

    // ── SessionStatus ──────────────────────────────────────

    #[test]
    fn session_status_serde_roundtrip() {
        // SessionStatus has no #[serde(rename_all)], so variants serialize
        // as their Rust names (PascalCase): "Running", "Paused", etc.
        let json = serde_json::to_string(&SessionStatus::Running).unwrap();
        assert_eq!(json, "\"Running\"");

        let status: SessionStatus = serde_json::from_str("\"Paused\"").unwrap();
        assert!(matches!(status, SessionStatus::Paused));
    }

    // ── Full Session roundtrip ─────────────────────────────

    #[test]
    fn session_full_serde_roundtrip() {
        let session = Session {
            id: Uuid::new_v4(),
            name: "test-agent".into(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            model_config: ModelConfig::default(),
            permissions: PermissionSet::default(),
            status: SessionStatus::Running,
        };
        let json = serde_json::to_string(&session).unwrap();
        let deserialized: Session = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "test-agent");
        assert_eq!(deserialized.id, session.id);
    }

    // ── ToolCall / ToolResult ──────────────────────────────

    #[test]
    fn tool_call_serde() {
        let call = ToolCall {
            id: "call_1".into(),
            tool: "terminal".into(),
            arguments: serde_json::json!({"command": "ls"}),
        };
        let json = serde_json::to_string(&call).unwrap();
        let deserialized: ToolCall = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.tool, "terminal");
    }
}

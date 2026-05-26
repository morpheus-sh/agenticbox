use serde::{Deserialize, Serialize};
use shared_types::{FsPermission, NetworkPolicy, PermissionSet};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRequest {
    pub action: String,
    pub resource: String,
    pub permissions: PermissionSet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyDecision {
    Allow,
    Deny(String),
}

pub struct PolicyEngine;

impl PolicyEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn evaluate(&self, req: PolicyRequest) -> PolicyDecision {
        info!("Evaluating policy: {} on {}", req.action, req.resource);
        match req.action.as_str() {
            "terminal:exec" => {
                if req.permissions.terminal {
                    PolicyDecision::Allow
                } else {
                    PolicyDecision::Deny("Terminal access not granted".into())
                }
            }
            "fs:read" => {
                match req.permissions.filesystem {
                    FsPermission::ReadOnly | FsPermission::ReadWrite => PolicyDecision::Allow,
                    _ => PolicyDecision::Deny("Filesystem read not granted".into()),
                }
            }
            "fs:write" => {
                match req.permissions.filesystem {
                    FsPermission::ReadWrite => PolicyDecision::Allow,
                    _ => PolicyDecision::Deny("Filesystem write not granted".into()),
                }
            }
            "browser:use" => {
                if req.permissions.browser {
                    PolicyDecision::Allow
                } else {
                    PolicyDecision::Deny("Browser access not granted".into())
                }
            }
            "network:outbound" => {
                match &req.permissions.network {
                    NetworkPolicy::Full => PolicyDecision::Allow,
                    NetworkPolicy::Allowlist(domains) => {
                        if domains.iter().any(|d| req.resource.contains(d)) {
                            PolicyDecision::Allow
                        } else {
                            PolicyDecision::Deny("Domain not in allowlist".into())
                        }
                    }
                    NetworkPolicy::LocalhostOnly => {
                        if req.resource.contains("localhost") || req.resource.contains("127.0.0.1") {
                            PolicyDecision::Allow
                        } else {
                            PolicyDecision::Deny("Only localhost allowed".into())
                        }
                    }
                    NetworkPolicy::Offline => PolicyDecision::Deny("Network is offline".into()),
                }
            }
            _ => PolicyDecision::Deny("Unknown action".into()),
        }
    }
}

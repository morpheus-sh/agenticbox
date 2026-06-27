use serde::{Deserialize, Serialize};
use shared_types::host;
use shared_types::{FsPermission, NetworkPolicy, PermissionSet};
use tracing::info;

/// A policy evaluation request.
///
/// `role` carries the agent's role (e.g. `"formatter"`, `"security-analyst"`).
/// It acts as a **deterministic privilege ceiling** — see [`RoleProfile`].
/// Effective permissions are the intersection of the session's granted
/// `permissions` and the role's ceiling; the ceiling is never bypassed. A
/// manifest may grant a formatter full network access, but the formatter role
/// ceiling strips it. That floor is what makes the system auditable and
/// sellable to enterprises.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRequest {
    #[serde(default)]
    pub role: String,
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

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl PolicyEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn evaluate(&self, req: PolicyRequest) -> PolicyDecision {
        info!(
            "Evaluating policy: role={} {} on {}",
            req.role, req.action, req.resource
        );

        // ── Layer 0: apply the deterministic role ceiling ──────────────
        // Effective = min(granted, ceiling). This runs before the permission
        // match, so a role can only ever NARROW what a session was granted —
        // never expand it.
        let ceiling = RoleProfile::for_role(&req.role).ceiling;
        let effective = cap_permissions(&req.permissions, &ceiling);

        match req.action.as_str() {
            "terminal:exec" => {
                if effective.terminal {
                    PolicyDecision::Allow
                } else {
                    PolicyDecision::Deny(bool_reason(
                        "Terminal access",
                        req.permissions.terminal,
                        effective.terminal,
                        &req.role,
                    ))
                }
            }
            "fs:read" => match effective.filesystem {
                FsPermission::ReadOnly | FsPermission::ReadWrite => PolicyDecision::Allow,
                _ => PolicyDecision::Deny(fs_reason(
                    "Filesystem read",
                    &req.permissions.filesystem,
                    &effective.filesystem,
                    &req.role,
                )),
            },
            "fs:write" => match effective.filesystem {
                FsPermission::ReadWrite => PolicyDecision::Allow,
                _ => PolicyDecision::Deny(fs_reason(
                    "Filesystem write",
                    &req.permissions.filesystem,
                    &effective.filesystem,
                    &req.role,
                )),
            },
            "browser:use" => {
                if effective.browser {
                    PolicyDecision::Allow
                } else {
                    PolicyDecision::Deny(bool_reason(
                        "Browser access",
                        req.permissions.browser,
                        effective.browser,
                        &req.role,
                    ))
                }
            }
            "network:outbound" => {
                let h = host::extract_host(&req.resource);
                match &effective.network {
                    NetworkPolicy::Full => PolicyDecision::Allow,
                    NetworkPolicy::Allowlist(domains) => {
                        if domains.iter().any(|d| host::host_matches_domain(h, d)) {
                            PolicyDecision::Allow
                        } else {
                            PolicyDecision::Deny(format!("{h} not in allowlist"))
                        }
                    }
                    NetworkPolicy::LocalhostOnly => {
                        if host::is_localhost(h) {
                            PolicyDecision::Allow
                        } else {
                            PolicyDecision::Deny(format!("{h} not localhost"))
                        }
                    }
                    NetworkPolicy::Offline => PolicyDecision::Deny(net_offline_reason(
                        &req.permissions.network,
                        &req.role,
                    )),
                }
            }
            _ => PolicyDecision::Deny("Unknown action".into()),
        }
    }
}

// ── Role profiles (deterministic privilege ceilings) ────────────────────
//
// A role is a privilege ceiling: the maximum capabilities that role may EVER
// possess, no matter what a session/manifest grants. Effective policy =
// min(session permissions, role ceiling). The ceiling is the deterministic
// floor — it is never bypassed, which is what makes the system auditable and
// sellable to enterprises. The intelligence layer (LLM) sits ON TOP of this
// floor; it can only further restrict, never expand it.
//
// Unknown roles get an unrestricted ceiling (backward-compatible behaviour).
// For a hardened deployment, flip the default arm to deny-all and register
// every role explicitly.

#[derive(Debug, Clone)]
pub struct RoleProfile {
    pub name: String,
    pub ceiling: PermissionSet,
}

impl RoleProfile {
    pub fn for_role(role: &str) -> RoleProfile {
        match role {
            // A formatter rewrites text in the workspace. It needs read+write
            // to files but NEVER network, terminal, or browser. Even if a
            // manifest grants network: full, the ceiling strips it to offline.
            "formatter" => RoleProfile {
                name: "formatter".into(),
                ceiling: PermissionSet {
                    terminal: false,
                    filesystem: FsPermission::ReadWrite,
                    browser: false,
                    network: NetworkPolicy::Offline,
                },
            },
            // A security analyst reads code/logs and may fetch threat intel.
            // Read-only filesystem (never mutates evidence), scoped network,
            // terminal for grep/ripgrep. Write is capped away.
            "security-analyst" => RoleProfile {
                name: "security-analyst".into(),
                ceiling: PermissionSet {
                    terminal: true,
                    filesystem: FsPermission::ReadOnly,
                    browser: true,
                    network: NetworkPolicy::Allowlist(vec![
                        "cve.circl.lu".into(),
                        "api.github.com".into(),
                        "services.nvd.nist.gov".into(),
                    ]),
                },
            },
            // Unregistered role: no ceiling (unrestricted). Register a profile
            // above to constrain a role; flip to deny-all to harden by default.
            other => RoleProfile {
                name: other.into(),
                ceiling: PermissionSet {
                    terminal: true,
                    filesystem: FsPermission::ReadWrite,
                    browser: true,
                    network: NetworkPolicy::Full,
                },
            },
        }
    }
}

/// Effective permissions = the intersection of what was granted and what the
/// role ceiling permits. Each capability is reduced to its minimum.
fn cap_permissions(granted: &PermissionSet, ceiling: &PermissionSet) -> PermissionSet {
    PermissionSet {
        terminal: granted.terminal && ceiling.terminal,
        filesystem: min_fs(&granted.filesystem, &ceiling.filesystem),
        browser: granted.browser && ceiling.browser,
        network: min_network(&granted.network, &ceiling.network),
    }
}

fn min_fs(a: &FsPermission, b: &FsPermission) -> FsPermission {
    fn rank(f: &FsPermission) -> u8 {
        match f {
            FsPermission::Deny => 0,
            FsPermission::ReadOnly => 1,
            FsPermission::ReadWrite => 2,
        }
    }
    if rank(a) <= rank(b) {
        a.clone()
    } else {
        b.clone()
    }
}

/// Network policy lattice, most → least restrictive:
///   `Offline < LocalhostOnly < Allowlist(domains) < Full`
/// `min()` returns the more restrictive of the two.
fn min_network(a: &NetworkPolicy, b: &NetworkPolicy) -> NetworkPolicy {
    use NetworkPolicy::*;
    match (a, b) {
        (Offline, _) | (_, Offline) => Offline,
        (Full, other) | (other, Full) => other.clone(),
        (LocalhostOnly, LocalhostOnly) => LocalhostOnly,
        (LocalhostOnly, Allowlist(_)) | (Allowlist(_), LocalhostOnly) => LocalhostOnly,
        (Allowlist(da), Allowlist(db)) => {
            let inter: Vec<String> = da
                .iter()
                .filter(|d| db.iter().any(|e| e == *d))
                .cloned()
                .collect();
            Allowlist(inter)
        }
    }
}

// ── Deny reasons that name the binding constraint ──────────────────────
// When the role ceiling (not the session grant) is what blocked an action,
// the reason says so — this is the auditable trail.

fn bool_reason(capability: &str, granted: bool, effective: bool, role: &str) -> String {
    if granted && !effective {
        format!("{capability} denied by role '{role}' ceiling")
    } else {
        format!("{capability} not granted")
    }
}

fn fs_reason(
    capability: &str,
    granted: &FsPermission,
    effective: &FsPermission,
    role: &str,
) -> String {
    if granted != effective {
        format!("{capability} denied by role '{role}' ceiling")
    } else {
        format!("{capability} not granted")
    }
}

fn net_offline_reason(granted: &NetworkPolicy, role: &str) -> String {
    if !matches!(granted, NetworkPolicy::Offline) {
        format!("Network denied by role '{role}' ceiling")
    } else {
        "Network is offline".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine() -> PolicyEngine {
        PolicyEngine::new()
    }

    fn perms() -> PermissionSet {
        PermissionSet {
            terminal: true,
            filesystem: FsPermission::ReadWrite,
            browser: true,
            network: NetworkPolicy::Allowlist(vec!["api.openai.com".into(), "github.com".into()]),
        }
    }

    // `test-runner` is unregistered → unrestricted ceiling, so these tests
    // exercise the raw permission logic unchanged from before the role work.

    // ── terminal:exec ──────────────────────────────────────

    #[test]
    fn terminal_exec_allowed_when_granted() {
        let req = PolicyRequest {
            role: "test-runner".into(),
            action: "terminal:exec".into(),
            resource: "ls -la".into(),
            permissions: perms(),
        };
        assert!(matches!(engine().evaluate(req), PolicyDecision::Allow));
    }

    #[test]
    fn terminal_exec_denied_when_not_granted() {
        let mut p = perms();
        p.terminal = false;
        let req = PolicyRequest {
            role: "test-runner".into(),
            action: "terminal:exec".into(),
            resource: "rm -rf /".into(),
            permissions: p,
        };
        match engine().evaluate(req) {
            PolicyDecision::Deny(msg) => assert!(msg.contains("Terminal")),
            other => panic!("expected Deny, got {:?}", other),
        }
    }

    // ── fs:read ────────────────────────────────────────────

    #[test]
    fn fs_read_allowed_with_readonly() {
        let mut p = perms();
        p.filesystem = FsPermission::ReadOnly;
        let req = PolicyRequest {
            role: "test-runner".into(),
            action: "fs:read".into(),
            resource: "/workspace/main.rs".into(),
            permissions: p,
        };
        assert!(matches!(engine().evaluate(req), PolicyDecision::Allow));
    }

    #[test]
    fn fs_read_allowed_with_readwrite() {
        let req = PolicyRequest {
            role: "test-runner".into(),
            action: "fs:read".into(),
            resource: "/workspace/main.rs".into(),
            permissions: perms(),
        };
        assert!(matches!(engine().evaluate(req), PolicyDecision::Allow));
    }

    #[test]
    fn fs_read_denied_when_deny() {
        let mut p = perms();
        p.filesystem = FsPermission::Deny;
        let req = PolicyRequest {
            role: "test-runner".into(),
            action: "fs:read".into(),
            resource: "/workspace/main.rs".into(),
            permissions: p,
        };
        assert!(matches!(engine().evaluate(req), PolicyDecision::Deny(_)));
    }

    // ── fs:write ───────────────────────────────────────────

    #[test]
    fn fs_write_allowed_with_readwrite() {
        let req = PolicyRequest {
            role: "test-runner".into(),
            action: "fs:write".into(),
            resource: "/workspace/output.txt".into(),
            permissions: perms(),
        };
        assert!(matches!(engine().evaluate(req), PolicyDecision::Allow));
    }

    #[test]
    fn fs_write_denied_with_readonly() {
        let mut p = perms();
        p.filesystem = FsPermission::ReadOnly;
        let req = PolicyRequest {
            role: "test-runner".into(),
            action: "fs:write".into(),
            resource: "/workspace/output.txt".into(),
            permissions: p,
        };
        match engine().evaluate(req) {
            PolicyDecision::Deny(msg) => assert!(msg.contains("write")),
            other => panic!("expected Deny, got {:?}", other),
        }
    }

    #[test]
    fn fs_write_denied_with_deny() {
        let mut p = perms();
        p.filesystem = FsPermission::Deny;
        let req = PolicyRequest {
            role: "test-runner".into(),
            action: "fs:write".into(),
            resource: "/workspace/output.txt".into(),
            permissions: p,
        };
        assert!(matches!(engine().evaluate(req), PolicyDecision::Deny(_)));
    }

    // ── browser:use ────────────────────────────────────────

    #[test]
    fn browser_allowed_when_granted() {
        let req = PolicyRequest {
            role: "test-runner".into(),
            action: "browser:use".into(),
            resource: "https://example.com".into(),
            permissions: perms(),
        };
        assert!(matches!(engine().evaluate(req), PolicyDecision::Allow));
    }

    #[test]
    fn browser_denied_when_not_granted() {
        let mut p = perms();
        p.browser = false;
        let req = PolicyRequest {
            role: "test-runner".into(),
            action: "browser:use".into(),
            resource: "https://example.com".into(),
            permissions: p,
        };
        assert!(matches!(engine().evaluate(req), PolicyDecision::Deny(_)));
    }

    // ── network:outbound ───────────────────────────────────

    #[test]
    fn network_allowlist_permits_listed_domain() {
        let req = PolicyRequest {
            role: "test-runner".into(),
            action: "network:outbound".into(),
            resource: "https://api.openai.com/v1/chat".into(),
            permissions: perms(),
        };
        assert!(matches!(engine().evaluate(req), PolicyDecision::Allow));
    }

    #[test]
    fn network_allowlist_blocks_unlisted_domain() {
        let req = PolicyRequest {
            role: "test-runner".into(),
            action: "network:outbound".into(),
            resource: "https://evil.attacker.com".into(),
            permissions: perms(),
        };
        match engine().evaluate(req) {
            PolicyDecision::Deny(msg) => assert!(msg.contains("allowlist")),
            other => panic!("expected Deny, got {:?}", other),
        }
    }

    #[test]
    fn network_allowlist_blocks_path_spoof() {
        // substring bug regression: evil host, allowed domain in the path
        let req = PolicyRequest {
            role: "test-runner".into(),
            action: "network:outbound".into(),
            resource: "evil.com/github.com".into(),
            permissions: perms(),
        };
        assert!(matches!(engine().evaluate(req), PolicyDecision::Deny(_)));
    }

    #[test]
    fn network_full_allows_any() {
        let mut p = perms();
        p.network = NetworkPolicy::Full;
        let req = PolicyRequest {
            role: "test-runner".into(),
            action: "network:outbound".into(),
            resource: "https://evil.attacker.com".into(),
            permissions: p,
        };
        assert!(matches!(engine().evaluate(req), PolicyDecision::Allow));
    }

    #[test]
    fn network_localhost_only_allows_localhost() {
        let mut p = perms();
        p.network = NetworkPolicy::LocalhostOnly;
        let req = PolicyRequest {
            role: "test-runner".into(),
            action: "network:outbound".into(),
            resource: "http://localhost:3000".into(),
            permissions: p,
        };
        assert!(matches!(engine().evaluate(req), PolicyDecision::Allow));
    }

    #[test]
    fn network_localhost_only_blocks_external() {
        let mut p = perms();
        p.network = NetworkPolicy::LocalhostOnly;
        let req = PolicyRequest {
            role: "test-runner".into(),
            action: "network:outbound".into(),
            resource: "https://api.openai.com".into(),
            permissions: p,
        };
        assert!(matches!(engine().evaluate(req), PolicyDecision::Deny(_)));
    }

    #[test]
    fn network_offline_blocks_all() {
        let mut p = perms();
        p.network = NetworkPolicy::Offline;
        let req = PolicyRequest {
            role: "test-runner".into(),
            action: "network:outbound".into(),
            resource: "http://localhost".into(),
            permissions: p,
        };
        assert!(matches!(engine().evaluate(req), PolicyDecision::Deny(_)));
    }

    // ── Unknown action ─────────────────────────────────────

    #[test]
    fn unknown_action_denied() {
        let req = PolicyRequest {
            role: "test-runner".into(),
            action: "admin:shutdown".into(),
            resource: "system".into(),
            permissions: perms(),
        };
        match engine().evaluate(req) {
            PolicyDecision::Deny(msg) => assert!(msg.contains("Unknown")),
            other => panic!("expected Deny, got {:?}", other),
        }
    }

    // ── Attack scenarios (integration-style) ───────────────

    #[test]
    fn attack_ssh_key_read_blocked_by_fs_deny() {
        let mut p = perms();
        p.filesystem = FsPermission::ReadOnly;
        // Even with fs:read allowed, /root/.ssh is outside /workspace.
        // The policy engine checks permissions, FsGuard checks paths — both
        // layers matter.
        let req = PolicyRequest {
            role: "test-runner".into(),
            action: "fs:read".into(),
            resource: "/root/.ssh/id_rsa".into(),
            permissions: p,
        };
        // Policy engine allows because permission level is ReadOnly, but
        // FsGuard would block the actual access — two layers of defence.
        assert!(matches!(engine().evaluate(req), PolicyDecision::Allow));
    }

    #[test]
    fn attack_cron_persistence_blocked_by_fs_write() {
        let mut p = perms();
        p.filesystem = FsPermission::ReadOnly;
        let req = PolicyRequest {
            role: "test-runner".into(),
            action: "fs:write".into(),
            resource: "/etc/cron.d/persist".into(),
            permissions: p,
        };
        assert!(matches!(engine().evaluate(req), PolicyDecision::Deny(_)));
    }

    #[test]
    fn attack_data_exfiltration_blocked_by_network() {
        let req = PolicyRequest {
            role: "test-runner".into(),
            action: "network:outbound".into(),
            resource: "https://evil.attacker.com/exfil?data=secret".into(),
            permissions: perms(),
        };
        assert!(matches!(engine().evaluate(req), PolicyDecision::Deny(_)));
    }

    // ══ Role ceiling concept ════════════════════════════════════════════
    //
    // The role is a deterministic privilege floor. These tests prove the
    // ceiling strips capabilities even when the session grants them in full,
    // and that the deny reason names the role as the binding constraint.

    #[test]
    fn formatter_denies_network_even_when_granted_full() {
        let mut p = perms();
        p.network = NetworkPolicy::Full; // session grants everything
        let req = PolicyRequest {
            role: "formatter".into(),
            action: "network:outbound".into(),
            resource: "https://api.openai.com/v1/chat".into(),
            permissions: p,
        };
        match engine().evaluate(req) {
            PolicyDecision::Deny(msg) => assert!(msg.contains("formatter"), "msg: {msg}"),
            other => panic!("formatter must not reach the network, got {:?}", other),
        }
    }

    #[test]
    fn formatter_denies_terminal_even_when_granted() {
        let req = PolicyRequest {
            role: "formatter".into(),
            action: "terminal:exec".into(),
            resource: "ls".into(),
            permissions: perms(),
        };
        match engine().evaluate(req) {
            PolicyDecision::Deny(msg) => assert!(msg.contains("formatter")),
            other => panic!("expected Deny, got {:?}", other),
        }
    }

    #[test]
    fn formatter_allows_fs_write_in_workspace() {
        let req = PolicyRequest {
            role: "formatter".into(),
            action: "fs:write".into(),
            resource: "/workspace/formatted.rs".into(),
            permissions: perms(),
        };
        assert!(matches!(engine().evaluate(req), PolicyDecision::Allow));
    }

    #[test]
    fn security_analyst_caps_fs_write_to_readonly() {
        // granted ReadWrite, ceiling ReadOnly → effective ReadOnly → write blocked
        let req = PolicyRequest {
            role: "security-analyst".into(),
            action: "fs:write".into(),
            resource: "/workspace/report.md".into(),
            permissions: perms(),
        };
        match engine().evaluate(req) {
            PolicyDecision::Deny(msg) => assert!(msg.contains("security-analyst")),
            other => panic!("analyst must not mutate evidence, got {:?}", other),
        }
    }

    #[test]
    fn security_analyst_allows_fs_read() {
        let mut p = perms();
        p.filesystem = FsPermission::ReadOnly;
        let req = PolicyRequest {
            role: "security-analyst".into(),
            action: "fs:read".into(),
            resource: "/workspace/app.py".into(),
            permissions: p,
        };
        assert!(matches!(engine().evaluate(req), PolicyDecision::Allow));
    }

    #[test]
    fn security_analyst_network_scoped_by_ceiling() {
        // session grants full network; ceiling narrows to a threat-intel allowlist
        let mut p = perms();
        p.network = NetworkPolicy::Full;
        let allowed = PolicyRequest {
            role: "security-analyst".into(),
            action: "network:outbound".into(),
            resource: "https://cve.circl.lu/api/cve/CVE-2024-1".into(),
            permissions: p.clone(),
        };
        assert!(matches!(engine().evaluate(allowed), PolicyDecision::Allow));

        let blocked = PolicyRequest {
            role: "security-analyst".into(),
            action: "network:outbound".into(),
            resource: "https://evil.attacker.com/exfil".into(),
            permissions: p,
        };
        assert!(matches!(
            engine().evaluate(blocked),
            PolicyDecision::Deny(_)
        ));
    }

    #[test]
    fn unregistered_role_is_unrestricted_backward_compat() {
        let mut p = perms();
        p.network = NetworkPolicy::Full;
        let req = PolicyRequest {
            role: "some-custom-agent".into(),
            action: "network:outbound".into(),
            resource: "https://anything.evil.com".into(),
            permissions: p,
        };
        assert!(matches!(engine().evaluate(req), PolicyDecision::Allow));
    }

    #[test]
    fn ceiling_intersection_narrows_allowlist() {
        // session allowlist ∩ ceiling allowlist = intersection
        let mut p = perms();
        p.network = NetworkPolicy::Allowlist(vec![
            "api.openai.com".into(),
            "cve.circl.lu".into(), // also in the analyst ceiling
        ]);
        let req = PolicyRequest {
            role: "security-analyst".into(),
            action: "network:outbound".into(),
            resource: "https://cve.circl.lu/api".into(),
            permissions: p,
        };
        assert!(matches!(engine().evaluate(req), PolicyDecision::Allow));

        // api.openai.com is NOT in the analyst ceiling → intersection drops it
        let req2 = PolicyRequest {
            role: "security-analyst".into(),
            action: "network:outbound".into(),
            resource: "https://api.openai.com/v1/chat".into(),
            permissions: perms(), // allowlist has api.openai.com
        };
        assert!(matches!(engine().evaluate(req2), PolicyDecision::Deny(_)));
    }

    // ── permission lattice helpers ─────────────────────────

    #[test]
    fn min_fs_picks_more_restrictive() {
        assert!(matches!(
            min_fs(&FsPermission::ReadWrite, &FsPermission::ReadOnly),
            FsPermission::ReadOnly
        ));
        assert!(matches!(
            min_fs(&FsPermission::ReadOnly, &FsPermission::Deny),
            FsPermission::Deny
        ));
    }

    #[test]
    fn min_network_offline_dominates() {
        assert!(matches!(
            min_network(&NetworkPolicy::Full, &NetworkPolicy::Offline),
            NetworkPolicy::Offline
        ));
    }

    #[test]
    fn min_network_intersect_allowlists() {
        let r = min_network(
            &NetworkPolicy::Allowlist(vec!["a.com".into(), "b.com".into()]),
            &NetworkPolicy::Allowlist(vec!["b.com".into(), "c.com".into()]),
        );
        match r {
            NetworkPolicy::Allowlist(d) => assert_eq!(d, vec!["b.com".to_string()]),
            other => panic!("expected Allowlist, got {:?}", other),
        }
    }
}

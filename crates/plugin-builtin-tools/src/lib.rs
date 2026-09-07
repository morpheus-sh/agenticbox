//! Builtin tool plugins for AgenticBox: filesystem, network, exec.
//!
//! Each plugin wraps the existing guard crates (fs-guard, network-control,
//! policy-engine) behind the harness-core `ToolPlugin` trait. Behavior is
//! identical to the pre-plugin agent-loop implementations — this is an
//! extraction, not a rewrite.

use harness_core::{HarnessContext, ToolCall, ToolOutcome, ToolPlugin};
use policy_engine::PolicyEngine;
use shared_types::{FsPermission, NetworkPolicy, PermissionSet};

// ─── Filesystem tools ─────────────────────────────────────────

/// `read_file` / `write_file`, guarded by `FsGuard` against the workspace root.
pub struct FsPlugin;

impl ToolPlugin for FsPlugin {
    fn tool_names(&self) -> Vec<String> {
        vec!["read_file".into(), "write_file".into()]
    }

    fn handle(&self, call: ToolCall<'_>, ctx: &HarnessContext) -> anyhow::Result<ToolOutcome> {
        let guard = fs_guard::FsGuard::new(vec![ctx.workspace.clone()]);
        match call.name {
            "read_file" => {
                let path = call.args.get("path").and_then(|v| v.as_str()).unwrap_or("");
                match guard.resolve(path) {
                    Ok(resolved) => match std::fs::read_to_string(&resolved) {
                        Ok(content) => Ok(ToolOutcome::allowed("within allowed roots", content)),
                        Err(e) => Ok(ToolOutcome::blocked(format!("read error: {e}"))),
                    },
                    Err(e) => Ok(ToolOutcome::blocked(format!("filesystem: {e}"))),
                }
            }
            "write_file" => {
                let path = call.args.get("path").and_then(|v| v.as_str()).unwrap_or("");
                let content = call
                    .args
                    .get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                match guard.resolve(path) {
                    Ok(resolved) => match std::fs::write(&resolved, content) {
                        Ok(()) => Ok(ToolOutcome::allowed(
                            "within allowed roots",
                            "File written successfully",
                        )),
                        Err(e) => Ok(ToolOutcome::blocked(format!("write error: {e}"))),
                    },
                    Err(e) => Ok(ToolOutcome::blocked(format!("filesystem: {e}"))),
                }
            }
            other => Ok(ToolOutcome::blocked(format!(
                "fs plugin cannot handle {other}"
            ))),
        }
    }
}

// ─── Network tool ─────────────────────────────────────────────

/// `http_request`, domain-checked by `NetworkGuard` against the allowlist.
pub struct NetworkPlugin;

impl ToolPlugin for NetworkPlugin {
    fn tool_names(&self) -> Vec<String> {
        vec!["http_request".into()]
    }

    fn handle(&self, call: ToolCall<'_>, ctx: &HarnessContext) -> anyhow::Result<ToolOutcome> {
        let url = call.args.get("url").and_then(|v| v.as_str()).unwrap_or("");
        let _method = call
            .args
            .get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("GET");
        let guard = network_control::NetworkGuard::new(NetworkPolicy::Allowlist(
            ctx.network_allowlist.clone(),
        ));
        match guard.check(url) {
            Ok(()) => Ok(ToolOutcome::allowed(
                "domain in allowlist",
                format!("HTTP 200 OK (simulated — {url} is allowlisted)"),
            )),
            Err(e) => Ok(ToolOutcome::blocked(format!("network: {e}"))),
        }
    }
}

// ─── Exec tool ────────────────────────────────────────────────

/// `exec`, gated by the `PolicyEngine` terminal permission, executed via the
/// system shell (cmd on Windows, sh elsewhere) — matching the pre-plugin
/// `execute_exec` behavior, pipes and `&&` included.
///
/// ponytail: guards are rebuilt per call; fine while PolicyEngine is a unit
/// struct, but hoist into the plugin if it gains construction-time config.
pub struct ExecPlugin;

impl ExecPlugin {
    fn evaluate(&self, command: &str) -> policy_engine::PolicyDecision {
        let engine = PolicyEngine::new();
        let req = policy_engine::PolicyRequest {
            action: "terminal:exec".into(),
            resource: command.into(),
            permissions: PermissionSet {
                terminal: true,
                filesystem: FsPermission::ReadWrite,
                browser: false,
                network: NetworkPolicy::Allowlist(vec![]),
            },
        };
        engine.evaluate(req)
    }
}

impl ToolPlugin for ExecPlugin {
    fn tool_names(&self) -> Vec<String> {
        vec!["exec".into()]
    }

    fn handle(&self, call: ToolCall<'_>, _ctx: &HarnessContext) -> anyhow::Result<ToolOutcome> {
        let command = call
            .args
            .get("command")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        match self.evaluate(command) {
            policy_engine::PolicyDecision::Allow => {
                // Use shell to handle pipes, paths, && — Windows uses cmd, Unix uses sh
                #[cfg(windows)]
                let (shell, flag) = ("cmd.exe", "/C");
                #[cfg(not(windows))]
                let (shell, flag) = ("sh", "-c");

                let output = std::process::Command::new(shell)
                    .arg(flag)
                    .arg(command)
                    .output();
                match output {
                    Ok(out) => {
                        let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                        let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                        let combined = if stdout.is_empty() { stderr } else { stdout };
                        Ok(ToolOutcome::allowed("terminal access granted", combined))
                    }
                    Err(e) => Ok(ToolOutcome::blocked(format!("exec error: {e}"))),
                }
            }
            policy_engine::PolicyDecision::Deny(reason) => {
                Ok(ToolOutcome::blocked(format!("terminal: {reason}")))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use harness_core::Harness;

    fn ctx() -> HarnessContext {
        // Canonicalize to match FsGuard's internal canonicalization (Windows
        // extended-length `\\?\` prefixes otherwise break starts_with).
        let dir =
            std::env::temp_dir().join(format!("agenticbox-plugin-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let dir = dir.canonicalize().unwrap_or(dir);
        HarnessContext {
            workspace: dir,
            network_allowlist: vec!["api.github.com".into()],
        }
    }

    #[test]
    fn read_write_roundtrip_inside_workspace() {
        let c = ctx();
        let mut h = Harness::new(c.clone());
        h.register_tool(std::sync::Arc::new(FsPlugin));

        let args = serde_json::json!({"path": "hello.txt", "content": "world"});
        let out = h.dispatch(ToolCall {
            name: "write_file",
            args: &args,
        });
        assert!(out.allowed, "write blocked: {}", out.reason);

        let args = serde_json::json!({"path": "hello.txt"});
        let out = h.dispatch(ToolCall {
            name: "read_file",
            args: &args,
        });
        assert!(out.allowed);
        assert_eq!(out.output, "world");
    }

    #[test]
    fn read_outside_workspace_blocked() {
        let c = ctx();
        let mut h = Harness::new(c);
        h.register_tool(std::sync::Arc::new(FsPlugin));
        // `..` escape: outside the workspace on every platform, and the block
        // must come from the guard (reason mentions the filesystem), not from
        // a file-not-found error.
        let args = serde_json::json!({"path": "../agenticbox-escape-probe.txt"});
        let out = h.dispatch(ToolCall {
            name: "read_file",
            args: &args,
        });
        assert!(!out.allowed);
        assert!(
            out.reason.contains("filesystem"),
            "block must come from the guard, got: {}",
            out.reason
        );
    }

    #[test]
    fn network_allowlist_enforced() {
        let c = ctx();
        let mut h = Harness::new(c);
        h.register_tool(std::sync::Arc::new(NetworkPlugin));

        let ok = serde_json::json!({"url": "https://api.github.com/x", "method": "GET"});
        assert!(
            h.dispatch(ToolCall {
                name: "http_request",
                args: &ok
            })
            .allowed
        );

        let bad = serde_json::json!({"url": "https://evil.example.com/x", "method": "GET"});
        assert!(
            !h.dispatch(ToolCall {
                name: "http_request",
                args: &bad
            })
            .allowed
        );
    }

    #[test]
    fn exec_runs_and_returns_output() {
        let c = ctx();
        let mut h = Harness::new(c);
        h.register_tool(std::sync::Arc::new(ExecPlugin));
        // Platform-portable: echo is a shell builtin everywhere.
        #[cfg(windows)]
        let (cmd, expect) = ("echo ponytail", "ponytail");
        #[cfg(not(windows))]
        let (cmd, expect) = ("echo ponytail", "ponytail");
        let args = serde_json::json!({"command": cmd});
        let out = h.dispatch(ToolCall {
            name: "exec",
            args: &args,
        });
        assert!(out.allowed, "exec blocked: {}", out.reason);
        assert!(
            out.output.contains(expect),
            "exec must return real shell output, got: {:?}",
            out.output
        );
    }
}

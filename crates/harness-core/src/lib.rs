//! AgenticBox microkernel.
//!
//! The harness is a small core (`Harness`) that owns a plugin registry.
//! Everything else — filesystem access, network access, exec — is a
//! plugin implementing the [`ToolPlugin`] trait. The core knows nothing
//! about any specific capability; it routes and reports.
//!
//! Tool names are matched first-registration-wins; unknown tools fail
//! closed in the core, so plugins can never be bypassed by a
//! hallucinated tool name.

use anyhow::Result;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Outcome of a tool plugin handling a call.
#[derive(Debug, Clone)]
pub struct ToolOutcome {
    /// Whether the action was permitted and (attempted) executed.
    pub allowed: bool,
    /// Human-readable reason (shown in the action log / audit trail).
    pub reason: String,
    /// Output fed back to the model on success.
    pub output: String,
}

impl ToolOutcome {
    pub fn allowed(reason: impl Into<String>, output: impl Into<String>) -> Self {
        Self {
            allowed: true,
            reason: reason.into(),
            output: output.into(),
        }
    }
    pub fn blocked(reason: impl Into<String>) -> Self {
        Self {
            allowed: false,
            reason: reason.into(),
            output: String::new(),
        }
    }
}

/// A single agent tool call, already parsed from JSON arguments.
#[derive(Debug, Clone)]
pub struct ToolCall<'a> {
    pub name: &'a str,
    pub args: &'a serde_json::Value,
}

/// A plugin that handles one or more agent tools.
pub trait ToolPlugin: Send + Sync {
    /// Tool names this plugin handles (used for routing).
    fn tool_names(&self) -> Vec<String>;

    /// Handle a tool call. `ctx` gives read access to shared harness config.
    ///
    /// Implementations report failure through [`ToolOutcome::blocked`]; the
    /// `Result` exists so a malfunctioning plugin (I/O error building its
    /// response, poisoned lock) fails closed instead of panicking.
    fn handle(&self, call: ToolCall<'_>, ctx: &HarnessContext) -> Result<ToolOutcome>;
}

/// Read-only context handed to tool plugins.
#[derive(Debug, Clone, Default)]
pub struct HarnessContext {
    /// Workspace root the agent is allowed to touch.
    pub workspace: std::path::PathBuf,
    /// Network allowlist (domains).
    pub network_allowlist: Vec<String>,
}

type Tool = Arc<dyn ToolPlugin>;

/// The microkernel. Small on purpose: it routes tool calls to plugins.
/// All capability logic lives in plugins.
#[derive(Default)]
pub struct Harness {
    tools: BTreeMap<String, Tool>,
    ctx: HarnessContext,
}

impl Harness {
    pub fn new(ctx: HarnessContext) -> Self {
        Self {
            tools: BTreeMap::new(),
            ctx,
        }
    }

    /// Register a tool plugin. First registration of a name wins.
    pub fn register_tool(&mut self, plugin: Arc<dyn ToolPlugin>) {
        for name in plugin.tool_names() {
            self.tools.entry(name).or_insert(plugin.clone());
        }
    }

    /// Route one tool call.
    ///
    /// Unknown tools are blocked by the core (fail-closed); a plugin
    /// returning `Err` is also converted to a blocked outcome rather
    /// than aborting the agent session.
    pub fn dispatch(&self, call: ToolCall<'_>) -> ToolOutcome {
        match self.tools.get(call.name) {
            Some(plugin) => plugin
                .handle(call.clone(), &self.ctx)
                .unwrap_or_else(|e| ToolOutcome::blocked(format!("plugin error: {e}"))),
            None => ToolOutcome::blocked(format!("unknown tool: {}", call.name)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    struct EchoPlugin {
        calls: AtomicU32,
    }

    impl ToolPlugin for EchoPlugin {
        fn tool_names(&self) -> Vec<String> {
            vec!["echo".into()]
        }
        fn handle(&self, _call: ToolCall<'_>, _ctx: &HarnessContext) -> Result<ToolOutcome> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(ToolOutcome::allowed("echoed", "ok"))
        }
    }

    #[test]
    fn dispatch_routes_to_plugin() {
        let mut h = Harness::new(HarnessContext::default());
        h.register_tool(Arc::new(EchoPlugin {
            calls: AtomicU32::new(0),
        }));
        let args = serde_json::json!({"text": "hi"});
        let out = h.dispatch(ToolCall {
            name: "echo",
            args: &args,
        });
        assert!(out.allowed);
        assert_eq!(h.tools.len(), 1);
    }

    #[test]
    fn unknown_tool_fails_closed() {
        let h = Harness::new(HarnessContext::default());
        let args = serde_json::json!({});
        let out = h.dispatch(ToolCall {
            name: "nope",
            args: &args,
        });
        assert!(!out.allowed);
        assert!(out.reason.contains("unknown tool"));
    }
}

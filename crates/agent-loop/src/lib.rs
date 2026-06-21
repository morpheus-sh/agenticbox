use anyhow::{Context, Result};
use console::style;
use fs_guard::FsGuard;
use network_control::NetworkGuard;
use policy_engine::{PolicyDecision, PolicyEngine, PolicyRequest};
use serde::{Deserialize, Serialize};
use shared_types::{FsPermission, NetworkPolicy, PermissionSet};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

// ─── Public types ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentLoopConfig {
    pub api_base: String,
    pub model: String,
    pub workspace: PathBuf,
    pub network_allowlist: Vec<String>,
    pub max_iterations: usize,
    pub system_prompt: String,
    pub user_task: String,
}

impl Default for AgentLoopConfig {
    fn default() -> Self {
        Self {
            api_base: "http://localhost:1234/v1".into(),
            model: "qwen3.6-35b-a3b".into(),
            workspace: PathBuf::from("./workspace"),
            network_allowlist: vec!["api.github.com".into(), "registry.npmjs.org".into()],
            max_iterations: 15,
            system_prompt: String::new(),
            user_task: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentLoopResult {
    pub allowed: u32,
    pub blocked: u32,
    pub history: Vec<DecisionLog>,
    pub final_message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionLog {
    pub timestamp: String,
    pub tool: String,
    pub args: String,
    pub allowed: bool,
    pub reason: String,
    pub agent_message: Option<String>,
}

// ─── OpenAI-compatible API types ──────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: FunctionCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    tools: Vec<ToolDefinition>,
    temperature: f32,
}

#[derive(Debug, Serialize)]
struct ToolDefinition {
    #[serde(rename = "type")]
    def_type: String,
    function: ToolFunction,
}

#[derive(Debug, Serialize)]
struct ToolFunction {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

// ─── Tool definitions (OpenAI function-calling schema) ────────

fn tool_definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            def_type: "function".into(),
            function: ToolFunction {
                name: "read_file".into(),
                description: "Read the contents of a file. The path must be within the allowed workspace directory.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Absolute or relative path to the file" }
                    },
                    "required": ["path"]
                }),
            },
        },
        ToolDefinition {
            def_type: "function".into(),
            function: ToolFunction {
                name: "write_file".into(),
                description: "Write content to a file. The path must be within the allowed workspace directory.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Path to write to" },
                        "content": { "type": "string", "description": "Content to write" }
                    },
                    "required": ["path", "content"]
                }),
            },
        },
        ToolDefinition {
            def_type: "function".into(),
            function: ToolFunction {
                name: "http_request".into(),
                description: "Make an HTTP request to a URL. Only allowlisted domains are permitted.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "url": { "type": "string", "description": "Full URL to request" },
                        "method": { "type": "string", "description": "HTTP method (GET, POST, etc.)" }
                    },
                    "required": ["url", "method"]
                }),
            },
        },
        ToolDefinition {
            def_type: "function".into(),
            function: ToolFunction {
                name: "exec".into(),
                description: "Execute a shell command in the sandbox.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "command": { "type": "string", "description": "Command to execute" }
                    },
                    "required": ["command"]
                }),
            },
        },
    ]
}

// ─── Helpers ──────────────────────────────────────────────────

fn ts() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    format!(
        "{:02}:{:02}:{:02}",
        (secs % 86400) / 3600,
        (secs % 3600) / 60,
        secs % 60
    )
}

fn print_step(tool: &str, args: &str) {
    println!(
        "{} {} {}",
        style(&ts()).dim(),
        style("AGENT →").cyan().bold(),
        style(format!("{tool}({args})")).yellow()
    );
}

fn print_allowed(reason: &str) {
    println!(
        "  {} {}",
        style("✓ ALLOWED").green().bold(),
        style(reason).dim()
    );
}

fn print_blocked(reason: &str) {
    println!(
        "  {} {}",
        style("✗ BLOCKED").red().bold(),
        style(reason).dim()
    );
}

fn print_snippet(data: &str) {
    let snippet: String = data.chars().take(200).collect();
    let ellipsis = if data.len() > 200 { "..." } else { "" };
    for line in snippet.lines().take(8) {
        println!("  {} {}", style("│").dim(), style(line).dim());
    }
    if !ellipsis.is_empty() {
        println!("  {} {}", style("│").dim(), style(ellipsis).dim());
    }
}

// ─── Tool execution (through real guards) ─────────────────────

fn execute_read_file(
    path: &str,
    guard: &FsGuard,
) -> (bool, String, String) {
    match guard.resolve(path) {
        Ok(resolved) => match std::fs::read_to_string(&resolved) {
            Ok(content) => (true, "within allowed roots".into(), content),
            Err(e) => (false, format!("read error: {e}"), String::new()),
        },
        Err(e) => (false, format!("filesystem: {e}"), String::new()),
    }
}

fn execute_write_file(
    path: &str,
    content: &str,
    guard: &FsGuard,
) -> (bool, String, String) {
    match guard.resolve(path) {
        Ok(resolved) => match std::fs::write(&resolved, content) {
            Ok(()) => (true, "within allowed roots".into(), "File written successfully".into()),
            Err(e) => (false, format!("write error: {e}"), String::new()),
        },
        Err(e) => (false, format!("filesystem: {e}"), String::new()),
    }
}

fn execute_http_request(
    url: &str,
    _method: &str,
    guard: &NetworkGuard,
) -> (bool, String, String) {
    match guard.check(url) {
        Ok(()) => {
            // URL is allowed — attempt real request, but we may be offline
            // Return simulated response for demo purposes (the guard ran)
            (true, "domain in allowlist".into(), format!("HTTP 200 OK (simulated — {url} is allowlisted)"))
        }
        Err(e) => (false, format!("network: {e}"), String::new()),
    }
}

fn execute_exec(
    command: &str,
    engine: &PolicyEngine,
) -> (bool, String, String) {
    let perms = PermissionSet {
        terminal: true,
        filesystem: FsPermission::ReadWrite,
        browser: false,
        network: NetworkPolicy::Allowlist(vec![]),
    };
    let req = PolicyRequest {
        action: "terminal:exec".into(),
        resource: command.into(),
        permissions: perms,
    };
    match engine.evaluate(req) {
        PolicyDecision::Allow => {
            // Actually run the command
            let parts: Vec<&str> = command.split_whitespace().collect();
            if parts.is_empty() {
                return (false, "empty command".into(), String::new());
            }
            let output = std::process::Command::new(parts[0])
                .args(&parts[1..])
                .output();
            match output {
                Ok(out) => {
                    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                    let combined = if stdout.is_empty() { stderr } else { stdout };
                    (true, "terminal access granted".into(), combined)
                }
                Err(e) => (false, format!("exec error: {e}"), String::new()),
            }
        }
        PolicyDecision::Deny(msg) => (false, format!("policy: {msg}"), String::new()),
    }
}

// ─── The agent loop ───────────────────────────────────────────

pub async fn run_agent_loop(config: AgentLoopConfig) -> Result<AgentLoopResult> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()?;

    // Initialize real guards
    let fs_guard = FsGuard::new(vec![config.workspace.clone()]);
    let net_guard = NetworkGuard::new(NetworkPolicy::Allowlist(
        config.network_allowlist.clone(),
    ));
    let policy_engine = PolicyEngine::new();

    let mut allowed: u32 = 0;
    let mut blocked: u32 = 0;
    let mut history: Vec<DecisionLog> = Vec::new();
    let mut final_message = String::new();

    // Build initial messages
    let mut messages = vec![
        ChatMessage {
            role: "system".into(),
            content: Some(config.system_prompt.clone()),
            tool_calls: None,
            tool_call_id: None,
        },
        ChatMessage {
            role: "user".into(),
            content: Some(config.user_task.clone()),
            tool_calls: None,
            tool_call_id: None,
        },
    ];

    let endpoint = format!("{}/chat/completions", config.api_base.trim_end_matches('/'));

    for iteration in 0..config.max_iterations {
        // Call the LLM
        let req_body = ChatRequest {
            model: config.model.clone(),
            messages: messages.clone(),
            tools: tool_definitions(),
            temperature: 0.7,
        };

        let resp = client.post(&endpoint).json(&req_body).send().await;

        let resp = match resp {
            Ok(r) => r,
            Err(e) => {
                anyhow::bail!(
                    "Failed to reach LLM at {}: {}. Is LM Studio or another OpenAI-compatible server running?",
                    endpoint, e
                );
            }
        };

        let chat_resp: ChatResponse = resp.json().await.context("Failed to parse LLM response")?;

        let choice = chat_resp
            .choices
            .into_iter()
            .next()
            .context("No choices in LLM response")?;

        let assistant_msg = choice.message;

        // Print content if the LLM said something
        if let Some(content) = &assistant_msg.content {
            if !content.is_empty() {
                println!();
                for line in content.lines() {
                    println!("  {} {}", style("💬").dim(), style(line).white());
                }
                final_message = content.clone();
            }
        }

        let tool_calls = match &assistant_msg.tool_calls {
            Some(calls) if !calls.is_empty() => calls.clone(),
            _ => {
                // No tool calls — agent is done
                println!(
                    "\n{} Agent finished after {} iteration(s).",
                    style("✓").green().bold(),
                    iteration + 1
                );
                break;
            }
        };

        // Add assistant message to history
        messages.push(assistant_msg.clone());

        // Execute each tool call through the guards
        for tc in &tool_calls {
            let args: serde_json::Value = serde_json::from_str(&tc.function.arguments)
                .unwrap_or(serde_json::json!({}));

            let tool_name = &tc.function.name;
            let args_str = serde_json::to_string(&args).unwrap_or_default();

            print_step(tool_name, &args_str);

            let (is_allowed, reason, output) = match tool_name.as_str() {
                "read_file" => {
                    let path = args.get("path")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    execute_read_file(path, &fs_guard)
                }
                "write_file" => {
                    let path = args.get("path")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let content = args.get("content")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    execute_write_file(path, content, &fs_guard)
                }
                "http_request" => {
                    let url = args.get("url")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let method = args.get("method")
                        .and_then(|v| v.as_str())
                        .unwrap_or("GET");
                    execute_http_request(url, method, &net_guard)
                }
                "exec" => {
                    let command = args.get("command")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    execute_exec(command, &policy_engine)
                }
                _ => {
                    (false, format!("unknown tool: {tool_name}"), String::new())
                }
            };

            if is_allowed {
                allowed += 1;
                print_allowed(&reason);
                if !output.is_empty() {
                    print_snippet(&output);
                }
            } else {
                blocked += 1;
                print_blocked(&reason);
            }

            history.push(DecisionLog {
                timestamp: ts(),
                tool: tool_name.clone(),
                args: args_str,
                allowed: is_allowed,
                reason: reason.clone(),
                agent_message: assistant_msg.content.clone(),
            });

            // Add tool result message
            let tool_result = if is_allowed {
                output.clone()
            } else {
                format!("BLOCKED: {reason}")
            };

            messages.push(ChatMessage {
                role: "tool".into(),
                content: Some(tool_result),
                tool_calls: None,
                tool_call_id: Some(tc.id.clone()),
            });
        }
    }

    Ok(AgentLoopResult {
        allowed,
        blocked,
        history,
        final_message,
    })
}

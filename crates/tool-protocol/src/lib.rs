use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInvocation {
    pub id: String,
    pub name: String,
    pub arguments: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolOutput {
    pub id: String,
    pub result: Result<Value, String>,
}

#[async_trait]
pub trait Tool: Send + Sync {
    fn definition(&self) -> ToolDefinition;
    async fn invoke(&self, args: Value) -> Result<Value>;
}

pub struct TerminalTool;

#[async_trait]
impl Tool for TerminalTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "terminal".into(),
            description: "Execute a shell command".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": { "type": "string" }
                },
                "required": ["command"]
            }),
        }
    }

    async fn invoke(&self, args: Value) -> Result<Value> {
        let _cmd = args["command"].as_str().unwrap_or_default();
        Ok(serde_json::json!({ "stdout": "", "stderr": "", "exit_code": 0 }))
    }
}

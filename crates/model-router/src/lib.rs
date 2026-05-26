use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Value>,
    pub stream: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub id: String,
    pub object: String,
    pub choices: Vec<Choice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    pub index: usize,
    pub message: Option<Value>,
    pub delta: Option<Value>,
    pub finish_reason: Option<String>,
}

#[async_trait]
pub trait ModelProvider: Send + Sync {
    async fn chat(&self, req: ChatRequest) -> Result<ChatResponse>;
    async fn models(&self) -> Result<Vec<String>>;
}

pub struct OpenAIProvider {
    client: Client,
    api_key: String,
    base_url: String,
}

impl OpenAIProvider {
    pub fn new(api_key: String, base_url: Option<String>) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: base_url.unwrap_or_else(|| "https://api.openai.com/v1".into()),
        }
    }
}

#[async_trait]
impl ModelProvider for OpenAIProvider {
    async fn chat(&self, req: ChatRequest) -> Result<ChatResponse> {
        info!("Sending chat request to OpenAI-compatible endpoint");
        let url = format!("{}/chat/completions", self.base_url);
        let resp = self.client.post(&url)
            .bearer_auth(&self.api_key)
            .json(&req)
            .send()
            .await?
            .json::<ChatResponse>()
            .await?;
        Ok(resp)
    }

    async fn models(&self) -> Result<Vec<String>> {
        let url = format!("{}/models", self.base_url);
        let resp = self.client.get(&url)
            .bearer_auth(&self.api_key)
            .send()
            .await?
            .json::<Value>()
            .await?;
        let models = resp["data"].as_array()
            .map(|arr| arr.iter().filter_map(|m| m["id"].as_str().map(String::from)).collect())
            .unwrap_or_default();
        Ok(models)
    }
}

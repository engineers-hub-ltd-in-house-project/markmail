use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::AIProviderConfig;
use crate::ai::{AIProvider, ChatMessage, MessageRole};

/// Anthropic Claude API プロバイダー
pub struct AnthropicProvider {
    client: Client,
    api_key: String,
    model: String,
    max_retries: u32,
}

impl AnthropicProvider {
    pub fn new(config: AIProviderConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()?;

        Ok(Self {
            client,
            api_key: config.api_key,
            model: config.model,
            max_retries: config.max_retries,
        })
    }

    async fn make_request<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        body: &T,
    ) -> Result<R> {
        let mut retries = 0;

        loop {
            let response = self
                .client
                .post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", &self.api_key)
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .json(body)
                .send()
                .await?;

            match response.status() {
                StatusCode::OK => {
                    return response
                        .json::<R>()
                        .await
                        .map_err(|e| anyhow!("Failed to parse response: {}", e));
                }
                StatusCode::TOO_MANY_REQUESTS => {
                    if retries < self.max_retries {
                        retries += 1;
                        tokio::time::sleep(Duration::from_secs(2u64.pow(retries))).await;
                        continue;
                    }
                    return Err(anyhow!(
                        "Rate limit exceeded after {} retries",
                        self.max_retries
                    ));
                }
                status => {
                    let error_text = response.text().await.unwrap_or_default();
                    return Err(anyhow!("Anthropic API error ({}): {}", status, error_text));
                }
            }
        }
    }
}

#[async_trait]
impl AIProvider for AnthropicProvider {
    async fn generate_text(&self, prompt: &str, max_tokens: Option<u32>) -> Result<String> {
        let messages = vec![Message {
            role: "user".to_string(),
            content: prompt.to_string(),
        }];

        let request = MessagesRequest {
            model: self.model.clone(),
            messages,
            max_tokens: max_tokens.unwrap_or(1000) as i32,
            temperature: Some(0.7),
            system: None,
        };

        let response: MessagesResponse = self.make_request(&request).await?;

        response
            .content
            .first()
            .map(|c| c.text.clone())
            .ok_or_else(|| anyhow!("No response from Anthropic"))
    }

    async fn chat(&self, messages: Vec<ChatMessage>, max_tokens: Option<u32>) -> Result<String> {
        let (system_message, chat_messages): (Option<String>, Vec<Message>) = {
            let mut system = None;
            let mut msgs = Vec::new();

            for msg in messages {
                match msg.role {
                    MessageRole::System => system = Some(msg.content),
                    MessageRole::User => msgs.push(Message {
                        role: "user".to_string(),
                        content: msg.content,
                    }),
                    MessageRole::Assistant => msgs.push(Message {
                        role: "assistant".to_string(),
                        content: msg.content,
                    }),
                }
            }

            (system, msgs)
        };

        let request = MessagesRequest {
            model: self.model.clone(),
            messages: chat_messages,
            max_tokens: max_tokens.unwrap_or(1000) as i32,
            temperature: Some(0.7),
            system: system_message,
        };

        let response: MessagesResponse = self.make_request(&request).await?;

        response
            .content
            .first()
            .map(|c| c.text.clone())
            .ok_or_else(|| anyhow!("No response from Anthropic"))
    }

    fn count_tokens(&self, text: &str) -> Result<usize> {
        // Anthropicのトークンカウントは概算
        // 平均的に4文字で1トークンとして計算
        Ok(text.len() / 4)
    }
}

/// Anthropic Messages API リクエスト
#[derive(Debug, Serialize)]
struct MessagesRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
}

/// Anthropic メッセージ
#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

/// Anthropic Messages API レスポンス
#[derive(Debug, Deserialize)]
struct MessagesResponse {
    content: Vec<Content>,
}

/// Anthropic コンテンツ
#[derive(Debug, Deserialize)]
struct Content {
    text: String,
}

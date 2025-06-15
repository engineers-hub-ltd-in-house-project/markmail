use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tiktoken_rs::p50k_base;

use super::AIProviderConfig;
use crate::ai::{AIProvider, ChatMessage, MessageRole};

/// OpenAI API プロバイダー
pub struct OpenAIProvider {
    client: Client,
    api_key: String,
    model: String,
    max_retries: u32,
}

impl OpenAIProvider {
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
        endpoint: &str,
        body: &T,
    ) -> Result<R> {
        let mut retries = 0;

        loop {
            let response = self
                .client
                .post(format!("https://api.openai.com/v1/{}", endpoint))
                .header("Authorization", format!("Bearer {}", self.api_key))
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
                    return Err(anyhow!("OpenAI API error ({}): {}", status, error_text));
                }
            }
        }
    }
}

#[async_trait]
impl AIProvider for OpenAIProvider {
    async fn generate_text(&self, prompt: &str, max_tokens: Option<u32>) -> Result<String> {
        let messages = vec![ChatMessage {
            role: MessageRole::User,
            content: prompt.to_string(),
        }];

        self.chat(messages, max_tokens).await
    }

    async fn chat(&self, messages: Vec<ChatMessage>, max_tokens: Option<u32>) -> Result<String> {
        let request = ChatCompletionRequest {
            model: self.model.clone(),
            messages: messages
                .into_iter()
                .map(|m| Message {
                    role: match m.role {
                        MessageRole::System => "system".to_string(),
                        MessageRole::User => "user".to_string(),
                        MessageRole::Assistant => "assistant".to_string(),
                    },
                    content: m.content,
                })
                .collect(),
            max_tokens: max_tokens.map(|t| t as i32),
            temperature: Some(0.7),
        };

        let response: ChatCompletionResponse =
            self.make_request("chat/completions", &request).await?;

        response
            .choices
            .first()
            .map(|choice| choice.message.content.clone())
            .ok_or_else(|| anyhow!("No response from OpenAI"))
    }

    fn count_tokens(&self, text: &str) -> Result<usize> {
        let bpe = p50k_base()?;
        let tokens = bpe.encode_with_special_tokens(text);
        Ok(tokens.len())
    }
}

/// OpenAI Chat Completion リクエスト
#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

/// OpenAI メッセージ
#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

/// OpenAI Chat Completion レスポンス
#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
}

/// OpenAI Choice
#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

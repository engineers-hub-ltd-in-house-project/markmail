pub mod models;
pub mod providers;
pub mod services;

use anyhow::Result;
use async_trait::async_trait;

/// AI プロバイダーの共通トレイト
#[async_trait]
pub trait AIProvider: Send + Sync {
    /// テキスト生成
    async fn generate_text(&self, prompt: &str, max_tokens: Option<u32>) -> Result<String>;

    /// チャット形式での生成
    async fn chat(&self, messages: Vec<ChatMessage>, max_tokens: Option<u32>) -> Result<String>;

    /// トークン数のカウント
    fn count_tokens(&self, text: &str) -> Result<usize>;
}

/// チャットメッセージ
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
}

/// メッセージロール
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

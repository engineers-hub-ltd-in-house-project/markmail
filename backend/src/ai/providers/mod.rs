pub mod anthropic;
pub mod openai;

use crate::ai::AIProvider;
use anyhow::Result;
use std::sync::Arc;

/// AI プロバイダーの設定
#[derive(Debug, Clone)]
pub struct AIProviderConfig {
    pub provider_type: AIProviderType,
    pub api_key: String,
    pub model: String,
    pub max_retries: u32,
    pub timeout_seconds: u64,
}

/// AI プロバイダータイプ
#[derive(Debug, Clone, PartialEq)]
pub enum AIProviderType {
    OpenAI,
    Anthropic,
}

/// AI プロバイダーファクトリー
pub fn create_ai_provider(config: AIProviderConfig) -> Result<Arc<dyn AIProvider>> {
    match config.provider_type {
        AIProviderType::OpenAI => {
            let provider = openai::OpenAIProvider::new(config)?;
            Ok(Arc::new(provider))
        }
        AIProviderType::Anthropic => {
            let provider = anthropic::AnthropicProvider::new(config)?;
            Ok(Arc::new(provider))
        }
    }
}

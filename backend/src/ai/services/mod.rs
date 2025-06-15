pub mod content_generator;
pub mod scenario_builder;

use crate::ai::AIProvider;
use std::sync::Arc;

/// AI サービスの基底構造体
pub struct AIService {
    provider: Arc<dyn AIProvider>,
}

impl AIService {
    pub fn new(provider: Arc<dyn AIProvider>) -> Self {
        Self { provider }
    }

    /// プロバイダーへの参照を取得
    pub fn provider(&self) -> &Arc<dyn AIProvider> {
        &self.provider
    }
}

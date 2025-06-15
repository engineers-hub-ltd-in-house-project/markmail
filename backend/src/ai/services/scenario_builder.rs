use anyhow::{anyhow, Result};
use serde_json;
use std::sync::Arc;

use crate::ai::models::prompts::{generate_scenario_user_prompt, get_scenario_system_prompt};
use crate::ai::models::{
    GenerateScenarioRequest, GenerateScenarioResponse, GeneratedForm, GeneratedFormField,
};
use crate::ai::{AIProvider, ChatMessage, MessageRole};

/// シナリオビルダーサービス
pub struct ScenarioBuilderService {
    provider: Arc<dyn AIProvider>,
}

impl ScenarioBuilderService {
    pub fn new(provider: Arc<dyn AIProvider>) -> Self {
        Self { provider }
    }

    /// マーケティングシナリオを自動生成
    pub async fn generate_scenario(
        &self,
        request: GenerateScenarioRequest,
    ) -> Result<GenerateScenarioResponse> {
        // 言語の決定（デフォルトは日本語）
        let language = request.language.unwrap_or_default();

        // プロンプトの構築
        let user_prompt = generate_scenario_user_prompt(
            &request.industry,
            &request.target_audience,
            &request.goal,
            request.additional_context.as_deref(),
            &language,
        );

        let system_prompt = get_scenario_system_prompt(&language);

        let messages = vec![
            ChatMessage {
                role: MessageRole::System,
                content: system_prompt.to_string(),
            },
            ChatMessage {
                role: MessageRole::User,
                content: user_prompt,
            },
        ];

        // AI からレスポンスを取得
        let ai_response = self.provider.chat(messages, Some(2000)).await?;

        // JSON をパース
        let response: GenerateScenarioResponse =
            serde_json::from_str(&ai_response).map_err(|e| {
                anyhow!(
                    "Failed to parse AI response: {}. Response: {}",
                    e,
                    ai_response
                )
            })?;

        // 検証とデフォルト値の設定
        let validated_response = self.validate_and_enhance_response(response)?;

        Ok(validated_response)
    }

    /// レスポンスの検証と拡張
    fn validate_and_enhance_response(
        &self,
        mut response: GenerateScenarioResponse,
    ) -> Result<GenerateScenarioResponse> {
        // シーケンスの検証
        if response.sequence.steps.is_empty() {
            return Err(anyhow!("Generated sequence has no steps"));
        }

        // テンプレートの検証
        if response.templates.is_empty() {
            return Err(anyhow!("No email templates generated"));
        }

        // フォームの検証と拡張
        if response.forms.is_empty() {
            // デフォルトのリードキャプチャフォームを追加
            response.forms.push(self.create_default_lead_form());
        }

        // ステップの遅延時間を正規化
        for step in &mut response.sequence.steps {
            if step.delay_value < 0 {
                step.delay_value = 0;
            }
            if !["minutes", "hours", "days"].contains(&step.delay_unit.as_str()) {
                step.delay_unit = "hours".to_string();
            }
        }

        Ok(response)
    }

    /// デフォルトのリードキャプチャフォームを作成
    fn create_default_lead_form(&self) -> GeneratedForm {
        GeneratedForm {
            name: "リードキャプチャフォーム".to_string(),
            description: "メールアドレスと基本情報を収集するフォーム".to_string(),
            fields: vec![
                GeneratedFormField {
                    field_type: "email".to_string(),
                    name: "email".to_string(),
                    label: "メールアドレス".to_string(),
                    required: true,
                    options: None,
                },
                GeneratedFormField {
                    field_type: "text".to_string(),
                    name: "name".to_string(),
                    label: "お名前".to_string(),
                    required: true,
                    options: None,
                },
                GeneratedFormField {
                    field_type: "text".to_string(),
                    name: "company".to_string(),
                    label: "会社名".to_string(),
                    required: false,
                    options: None,
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_lead_form_creation() {
        use crate::ai::providers::{create_ai_provider, AIProviderConfig, AIProviderType};

        let config = AIProviderConfig {
            provider_type: AIProviderType::OpenAI,
            api_key: "test-key".to_string(),
            model: "gpt-4".to_string(),
            max_retries: 3,
            timeout_seconds: 30,
        };

        let provider = create_ai_provider(config).unwrap();
        let service = ScenarioBuilderService::new(provider);

        let form = service.create_default_lead_form();
        assert_eq!(form.fields.len(), 3);
        assert_eq!(form.fields[0].field_type, "email");
        assert!(form.fields[0].required);
    }
}

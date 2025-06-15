use anyhow::{anyhow, Result};
use regex::Regex;
use std::sync::Arc;

use crate::ai::models::ai_responses::{
    ContentMetadata, GenerateContentRequest, GenerateContentResponse, OptimizeSubjectRequest,
    OptimizeSubjectResponse, SubjectVariation,
};
use crate::ai::models::prompts::{
    generate_subject_optimization_prompt, get_content_generation_system_prompt,
};
use crate::ai::{AIProvider, ChatMessage, MessageRole};

/// コンテンツジェネレーターサービス
pub struct ContentGeneratorService {
    provider: Arc<dyn AIProvider>,
    variable_regex: Regex,
}

impl ContentGeneratorService {
    pub fn new(provider: Arc<dyn AIProvider>) -> Self {
        Self {
            provider,
            variable_regex: Regex::new(r"\{\{([^}]+)\}\}").unwrap(),
        }
    }

    /// コンテンツを生成
    pub async fn generate_content(
        &self,
        request: GenerateContentRequest,
    ) -> Result<GenerateContentResponse> {
        let prompt = self.build_content_prompt(&request);

        // 言語の決定（デフォルトは日本語）
        let language = request.context.language.unwrap_or_default();
        let system_prompt = get_content_generation_system_prompt(&language);

        let messages = vec![
            ChatMessage {
                role: MessageRole::System,
                content: system_prompt.to_string(),
            },
            ChatMessage {
                role: MessageRole::User,
                content: prompt,
            },
        ];

        let ai_response = self.provider.chat(messages, Some(1000)).await?;

        // 生成されたコンテンツから変数を抽出
        let suggested_variables = self.extract_variables(&ai_response);

        // メタデータを計算
        let metadata = self.calculate_metadata(&ai_response, &suggested_variables);

        Ok(GenerateContentResponse {
            content: ai_response,
            variations: None,
            suggested_variables,
            metadata,
        })
    }

    /// 件名を最適化
    pub async fn optimize_subject(
        &self,
        request: OptimizeSubjectRequest,
    ) -> Result<OptimizeSubjectResponse> {
        // 言語の決定（デフォルトは日本語）
        let language = request.language.unwrap_or_default();

        let prompt = generate_subject_optimization_prompt(
            &request.original_subject,
            &request.target_audience,
            &language,
        );

        let system_message = match language {
            crate::ai::models::Language::Japanese => "あなたはメールマーケティングの専門家です。開封率を最大化する件名を提案してください。",
            crate::ai::models::Language::English => "You are an email marketing expert. Suggest subject lines that maximize open rates.",
        };

        let messages = vec![
            ChatMessage {
                role: MessageRole::System,
                content: system_message.to_string(),
            },
            ChatMessage {
                role: MessageRole::User,
                content: prompt,
            },
        ];

        let ai_response = self.provider.chat(messages, Some(500)).await?;

        // レスポンスをパースして件名バリエーションを抽出
        let variations = self.parse_subject_variations(&ai_response)?;

        // 最も良いと思われるものを選択（最初のものをデフォルトとする）
        let best_pick = 0;

        Ok(OptimizeSubjectResponse {
            optimized_subjects: variations,
            best_pick,
        })
    }

    /// コンテンツプロンプトを構築
    fn build_content_prompt(&self, request: &GenerateContentRequest) -> String {
        let language = request.context.language.unwrap_or_default();

        match language {
            crate::ai::models::Language::Japanese => {
                let mut prompt = format!("{}を作成してください。\n\n", request.content_type);

                if let Some(industry) = &request.context.industry {
                    prompt.push_str(&format!("業界: {}\n", industry));
                }

                if let Some(audience) = &request.context.target_audience {
                    prompt.push_str(&format!("ターゲット層: {}\n", audience));
                }

                if let Some(tone) = &request.context.tone {
                    prompt.push_str(&format!("トーン: {:?}\n", tone));
                }

                if let Some(existing) = &request.context.existing_content {
                    prompt.push_str(&format!(
                        "\n既存のコンテンツを改善してください:\n{}\n",
                        existing
                    ));
                }

                prompt.push_str("\n重要：すべての出力を日本語で生成してください。");
                prompt
            }
            crate::ai::models::Language::English => {
                let mut prompt = format!("Create a {} content.\n\n", request.content_type);

                if let Some(industry) = &request.context.industry {
                    prompt.push_str(&format!("Industry: {}\n", industry));
                }

                if let Some(audience) = &request.context.target_audience {
                    prompt.push_str(&format!("Target Audience: {}\n", audience));
                }

                if let Some(tone) = &request.context.tone {
                    prompt.push_str(&format!("Tone: {:?}\n", tone));
                }

                if let Some(existing) = &request.context.existing_content {
                    prompt.push_str(&format!(
                        "\nImprove the following existing content:\n{}\n",
                        existing
                    ));
                }

                prompt
            }
        }
    }

    /// 変数を抽出
    fn extract_variables(&self, content: &str) -> Vec<String> {
        self.variable_regex
            .captures_iter(content)
            .map(|cap| cap[1].to_string())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect()
    }

    /// メタデータを計算
    fn calculate_metadata(&self, content: &str, variables: &[String]) -> ContentMetadata {
        let word_count = content.split_whitespace().count();
        let estimated_reading_time = (word_count as f32 / 200.0 * 60.0) as u32; // 200 words per minute
        let personalization_score = (variables.len() as f32 / 5.0).min(1.0); // 5変数以上で満点
        let clarity_score = 0.8; // TODO: 実際の可読性スコアを計算

        ContentMetadata {
            estimated_reading_time,
            word_count,
            personalization_score,
            clarity_score,
        }
    }

    /// 件名バリエーションをパース
    fn parse_subject_variations(&self, response: &str) -> Result<Vec<SubjectVariation>> {
        // シンプルな実装：各行を件名として扱う
        let variations: Vec<SubjectVariation> = response
            .lines()
            .filter(|line| !line.trim().is_empty())
            .enumerate()
            .map(|(i, line)| SubjectVariation {
                subject: line.trim().to_string(),
                predicted_open_rate: 0.20 + (0.05 * (5 - i) as f32), // 仮の開封率
                reasoning: format!("バリエーション{}: ターゲット層に響く表現を使用", i + 1),
            })
            .take(5)
            .collect();

        if variations.is_empty() {
            return Err(anyhow!("No subject variations generated"));
        }

        Ok(variations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_variables() {
        use crate::ai::providers::{create_ai_provider, AIProviderConfig, AIProviderType};

        let config = AIProviderConfig {
            provider_type: AIProviderType::OpenAI,
            api_key: "test-key".to_string(),
            model: "gpt-4".to_string(),
            max_retries: 3,
            timeout_seconds: 30,
        };

        let provider = create_ai_provider(config).unwrap();
        let service = ContentGeneratorService::new(provider);

        let content = "こんにちは、{{name}}さん。{{company}}へようこそ！";
        let variables = service.extract_variables(content);

        assert_eq!(variables.len(), 2);
        assert!(variables.contains(&"name".to_string()));
        assert!(variables.contains(&"company".to_string()));
    }

    #[test]
    fn test_calculate_metadata() {
        use crate::ai::providers::{create_ai_provider, AIProviderConfig, AIProviderType};

        let config = AIProviderConfig {
            provider_type: AIProviderType::OpenAI,
            api_key: "test-key".to_string(),
            model: "gpt-4".to_string(),
            max_retries: 3,
            timeout_seconds: 30,
        };

        let provider = create_ai_provider(config).unwrap();
        let service = ContentGeneratorService::new(provider);

        let content = "This is a test email with some content.";
        let variables = vec!["name".to_string(), "company".to_string()];
        let metadata = service.calculate_metadata(content, &variables);

        assert_eq!(metadata.word_count, 8);
        assert!(metadata.estimated_reading_time > 0);
        assert!(metadata.personalization_score > 0.0);
    }
}

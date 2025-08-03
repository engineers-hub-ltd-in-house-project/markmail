use serde::{Deserialize, Serialize};

/// コンテンツ生成リクエスト
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateContentRequest {
    pub content_type: String,
    pub context: ContentContext,
    pub options: Option<GenerationOptions>,
}

use crate::ai::models::Language;

/// コンテンツコンテキスト
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentContext {
    pub industry: Option<String>,
    pub target_audience: Option<String>,
    pub tone: Option<ContentTone>,
    pub language: Option<Language>,
    pub existing_content: Option<String>,
}

/// コンテンツトーン
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentTone {
    Formal,
    Casual,
    Professional,
    Friendly,
    Urgent,
}

/// 生成オプション
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationOptions {
    pub variations: Option<u8>,
    pub max_length: Option<usize>,
    pub include_personalization: Option<bool>,
}

/// コンテンツ生成レスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateContentResponse {
    pub content: String,
    pub variations: Option<Vec<String>>,
    pub suggested_variables: Vec<String>,
    pub metadata: ContentMetadata,
}

/// コンテンツメタデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentMetadata {
    pub estimated_reading_time: u32, // 秒
    pub word_count: usize,
    pub personalization_score: f32, // 0.0 - 1.0
    pub clarity_score: f32,         // 0.0 - 1.0
}

/// 件名最適化リクエスト
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizeSubjectRequest {
    pub original_subject: String,
    pub target_audience: String,
    pub campaign_goal: Option<String>,
    pub variations_count: Option<u8>,
    pub language: Option<Language>,
}

/// 件名最適化レスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizeSubjectResponse {
    pub optimized_subjects: Vec<SubjectVariation>,
    pub best_pick: usize,
}

/// 件名バリエーション
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectVariation {
    pub subject: String,
    pub predicted_open_rate: f32,
    pub reasoning: String,
}

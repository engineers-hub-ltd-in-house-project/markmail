pub mod ai_responses;
pub mod prompts;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// AI生成コンテンツ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIGeneratedContent {
    pub id: Uuid,
    pub user_id: Uuid,
    pub content_type: ContentType,
    pub prompt: String,
    pub generated_content: String,
    pub model_used: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// コンテンツタイプ
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentType {
    EmailTemplate,
    Subject,
    Sequence,
    Form,
    Scenario,
}

/// AIシナリオテンプレート
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIScenarioTemplate {
    pub id: Uuid,
    pub name: String,
    pub industry: String,
    pub goal: String,
    pub template_data: serde_json::Value,
    pub success_metrics: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

/// シナリオ生成リクエスト
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateScenarioRequest {
    pub industry: String,
    pub target_audience: String,
    pub goal: String,
    pub additional_context: Option<String>,
}

/// シナリオ生成レスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateScenarioResponse {
    pub scenario_name: String,
    pub description: String,
    pub sequence: GeneratedSequence,
    pub forms: Vec<GeneratedForm>,
    pub templates: Vec<GeneratedTemplate>,
}

/// 生成されたシーケンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedSequence {
    pub name: String,
    pub description: String,
    pub trigger_type: String,
    pub steps: Vec<GeneratedSequenceStep>,
}

/// 生成されたシーケンスステップ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedSequenceStep {
    pub name: String,
    pub step_type: String,
    pub delay_value: i32,
    pub delay_unit: String,
    pub template_index: Option<usize>, // templatesベクトルへのインデックス
    pub conditions: Option<serde_json::Value>,
}

/// 生成されたフォーム
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedForm {
    pub name: String,
    pub description: String,
    pub fields: Vec<GeneratedFormField>,
}

/// 生成されたフォームフィールド
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedFormField {
    pub field_type: String,
    pub name: String,
    pub label: String,
    pub required: bool,
    pub options: Option<Vec<String>>,
}

/// 生成されたテンプレート
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedTemplate {
    pub name: String,
    pub subject: String,
    pub content: String,
    pub variables: Vec<String>,
}

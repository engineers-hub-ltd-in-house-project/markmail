use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AiUsageLog {
    pub id: Uuid,
    pub user_id: Uuid,
    pub feature_type: String,
    pub prompt: Option<String>,
    pub response: Option<String>,
    pub tokens_used: Option<i32>,
    pub model_used: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAiUsageLog {
    pub user_id: Uuid,
    pub feature_type: String,
    pub prompt: Option<String>,
    pub response: Option<String>,
    pub tokens_used: Option<i32>,
    pub model_used: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiUsageStats {
    pub total_usage: i64,
    pub scenario_usage: i64,
    pub content_usage: i64,
    pub subject_usage: i64,
}

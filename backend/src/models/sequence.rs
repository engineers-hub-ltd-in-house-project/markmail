use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Sequence {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub trigger_type: String,
    pub trigger_config: JsonValue,
    pub status: String,
    pub active_subscribers: i32,
    pub completed_subscribers: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSequenceRequest {
    pub name: String,
    pub description: Option<String>,
    pub trigger_type: String,
    pub trigger_config: Option<JsonValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSequenceRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub trigger_type: Option<String>,
    pub trigger_config: Option<JsonValue>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SequenceStep {
    pub id: Uuid,
    pub sequence_id: Uuid,
    pub name: String,
    pub step_order: i32,
    pub step_type: String,
    pub delay_value: i32,
    pub delay_unit: String,
    pub template_id: Option<Uuid>,
    pub subject: Option<String>,
    pub conditions: JsonValue,
    pub action_config: JsonValue,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSequenceStepRequest {
    pub name: String,
    pub step_order: i32,
    pub step_type: String,
    pub delay_value: Option<i32>,
    pub delay_unit: Option<String>,
    pub template_id: Option<Uuid>,
    pub subject: Option<String>,
    pub conditions: Option<JsonValue>,
    pub action_config: Option<JsonValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSequenceStepRequest {
    pub name: Option<String>,
    pub step_order: Option<i32>,
    pub step_type: Option<String>,
    pub delay_value: Option<i32>,
    pub delay_unit: Option<String>,
    pub template_id: Option<Uuid>,
    pub subject: Option<String>,
    pub conditions: Option<JsonValue>,
    pub action_config: Option<JsonValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SequenceEnrollment {
    pub id: Uuid,
    pub sequence_id: Uuid,
    pub subscriber_id: Uuid,
    pub current_step_id: Option<Uuid>,
    pub status: String,
    pub enrolled_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub next_step_at: Option<DateTime<Utc>>,
    pub metadata: JsonValue,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequenceWithSteps {
    #[serde(flatten)]
    pub sequence: Sequence,
    pub steps: Vec<SequenceStep>,
}

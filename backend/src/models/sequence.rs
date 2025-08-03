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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequenceStepWithTemplate {
    #[serde(flatten)]
    pub step: SequenceStep,
    pub template: Option<crate::models::template::Template>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequenceWithStepsAndTemplates {
    #[serde(flatten)]
    pub sequence: Sequence,
    pub steps: Vec<SequenceStepWithTemplate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SequenceStepLog {
    pub id: Uuid,
    pub enrollment_id: Uuid,
    pub step_id: Uuid,
    pub status: String,
    pub error_message: Option<String>,
    pub executed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSequenceEnrollmentRequest {
    pub subscriber_id: Uuid,
    pub trigger_data: Option<JsonValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerType {
    Manual,
    SubscriberCreated,
    FormSubmission,
    TagAdded,
}

impl TriggerType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TriggerType::Manual => "manual",
            TriggerType::SubscriberCreated => "subscriber_created",
            TriggerType::FormSubmission => "form_submission",
            TriggerType::TagAdded => "tag_added",
        }
    }
}

impl From<String> for TriggerType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "manual" => TriggerType::Manual,
            "subscriber_created" => TriggerType::SubscriberCreated,
            "form_submission" => TriggerType::FormSubmission,
            "tag_added" => TriggerType::TagAdded,
            _ => TriggerType::Manual,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepType {
    Email,
    Wait,
    Condition,
    Tag,
}

impl StepType {
    pub fn as_str(&self) -> &'static str {
        match self {
            StepType::Email => "email",
            StepType::Wait => "wait",
            StepType::Condition => "condition",
            StepType::Tag => "tag",
        }
    }
}

impl From<String> for StepType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "email" => StepType::Email,
            "wait" => StepType::Wait,
            "condition" => StepType::Condition,
            "tag" => StepType::Tag,
            _ => StepType::Email,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SequenceStatus {
    Draft,
    Active,
    Paused,
    Archived,
}

impl SequenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            SequenceStatus::Draft => "draft",
            SequenceStatus::Active => "active",
            SequenceStatus::Paused => "paused",
            SequenceStatus::Archived => "archived",
        }
    }
}

impl From<String> for SequenceStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "draft" => SequenceStatus::Draft,
            "active" => SequenceStatus::Active,
            "paused" => SequenceStatus::Paused,
            "archived" => SequenceStatus::Archived,
            _ => SequenceStatus::Draft,
        }
    }
}

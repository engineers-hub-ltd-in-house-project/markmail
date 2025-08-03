use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Form {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub slug: String,
    pub markdown_content: String,
    pub form_fields: JsonValue,
    pub settings: JsonValue,
    pub status: String,
    pub submission_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFormRequest {
    pub name: String,
    pub description: Option<String>,
    pub slug: Option<String>,
    pub markdown_content: String,
    pub form_fields: Option<JsonValue>,
    pub settings: Option<JsonValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateFormRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub markdown_content: Option<String>,
    pub form_fields: Option<JsonValue>,
    pub settings: Option<JsonValue>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormField {
    pub field_type: String,
    pub name: String,
    pub label: String,
    pub placeholder: Option<String>,
    pub required: bool,
    pub validation_rules: Option<JsonValue>,
    pub options: Option<JsonValue>,
    pub display_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FormSubmission {
    pub id: Uuid,
    pub form_id: Uuid,
    pub subscriber_id: Option<Uuid>,
    pub data: JsonValue,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub referrer: Option<String>,
    pub confirmation_token: Option<String>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFormSubmissionRequest {
    pub data: JsonValue,
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Template {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub subject_template: String,
    pub markdown_content: String,
    pub html_content: Option<String>,
    pub variables: Value, // JSONB型
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct CreateTemplateRequest {
    #[validate(length(
        min = 1,
        max = 200,
        message = "テンプレート名は1文字以上200文字以下である必要があります"
    ))]
    pub name: String,

    #[validate(length(
        min = 1,
        max = 200,
        message = "件名テンプレートは1文字以上200文字以下である必要があります"
    ))]
    pub subject_template: String,

    #[validate(length(
        min = 1,
        max = 50000,
        message = "マークダウンコンテンツは1文字以上50000文字以下である必要があります"
    ))]
    pub markdown_content: String,

    pub variables: Option<Value>,
    pub is_public: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct UpdateTemplateRequest {
    #[validate(length(
        min = 1,
        max = 200,
        message = "テンプレート名は1文字以上200文字以下である必要があります"
    ))]
    pub name: Option<String>,

    #[validate(length(
        min = 1,
        max = 200,
        message = "件名テンプレートは1文字以上200文字以下である必要があります"
    ))]
    pub subject_template: Option<String>,

    #[validate(length(
        min = 1,
        max = 50000,
        message = "マークダウンコンテンツは1文字以上50000文字以下である必要があります"
    ))]
    pub markdown_content: Option<String>,

    pub html_content: Option<String>,
    pub variables: Option<Value>,
    pub is_public: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct TemplateResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub subject_template: String,
    pub markdown_content: String,
    pub html_content: Option<String>,
    pub variables: Value,
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Template> for TemplateResponse {
    fn from(template: Template) -> Self {
        Self {
            id: template.id,
            user_id: template.user_id,
            name: template.name,
            subject_template: template.subject_template,
            markdown_content: template.markdown_content,
            html_content: template.html_content,
            variables: template.variables,
            is_public: template.is_public,
            created_at: template.created_at,
            updated_at: template.updated_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TemplateListResponse {
    pub templates: Vec<TemplateResponse>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PreviewTemplateRequest {
    pub variables: Value,
}

#[derive(Debug, Serialize)]
pub struct PreviewTemplateResponse {
    pub html: String,
    pub subject: String,
}

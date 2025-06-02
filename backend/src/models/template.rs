use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Template {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub markdown_content: String,
    pub html_content: String,
    pub subject_template: String,
    pub variables: Vec<String>, // JSON array
    pub tags: Vec<String>,      // JSON array
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[allow(dead_code)]
pub struct CreateTemplateRequest {
    #[validate(length(
        min = 1,
        max = 200,
        message = "テンプレート名は1文字以上200文字以下である必要があります"
    ))]
    pub name: String,

    #[validate(length(max = 500, message = "説明は500文字以下である必要があります"))]
    pub description: Option<String>,

    #[validate(length(
        min = 1,
        max = 50000,
        message = "コンテンツは1文字以上50000文字以下である必要があります"
    ))]
    pub markdown_content: String,

    #[validate(length(
        min = 1,
        max = 200,
        message = "件名テンプレートは1文字以上200文字以下である必要があります"
    ))]
    pub subject_template: String,

    pub tags: Vec<String>,
    pub is_public: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[allow(dead_code)]
pub struct UpdateTemplateRequest {
    #[validate(length(
        min = 1,
        max = 200,
        message = "テンプレート名は1文字以上200文字以下である必要があります"
    ))]
    pub name: Option<String>,

    #[validate(length(max = 500, message = "説明は500文字以下である必要があります"))]
    pub description: Option<String>,

    #[validate(length(
        min = 1,
        max = 50000,
        message = "コンテンツは1文字以上50000文字以下である必要があります"
    ))]
    pub markdown_content: Option<String>,

    #[validate(length(
        min = 1,
        max = 200,
        message = "件名テンプレートは1文字以上200文字以下である必要があります"
    ))]
    pub subject_template: Option<String>,

    pub tags: Option<Vec<String>>,
    pub is_public: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct TemplateResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub markdown_content: String,
    pub html_content: String,
    pub subject_template: String,
    pub variables: Vec<String>,
    pub tags: Vec<String>,
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Template> for TemplateResponse {
    fn from(template: Template) -> Self {
        Self {
            id: template.id,
            name: template.name,
            description: template.description,
            markdown_content: template.markdown_content,
            html_content: template.html_content,
            subject_template: template.subject_template,
            variables: template.variables,
            tags: template.tags,
            is_public: template.is_public,
            created_at: template.created_at,
            updated_at: template.updated_at,
        }
    }
}

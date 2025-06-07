use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Subscriber {
    pub id: Uuid,
    pub user_id: Uuid,
    pub email: String,
    pub name: Option<String>,
    pub status: SubscriberStatus,
    pub tags: Vec<String>,
    pub custom_fields: serde_json::Value,
    pub subscribed_at: DateTime<Utc>,
    pub unsubscribed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, Copy)]
#[sqlx(type_name = "subscriber_status", rename_all = "lowercase")]
pub enum SubscriberStatus {
    Active,
    Unsubscribed,
    Bounced,
    Complained,
}

// 購読者作成リクエスト
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateSubscriberRequest {
    #[validate(email(message = "有効なメールアドレスを入力してください"))]
    pub email: String,
    pub name: Option<String>,
    pub status: Option<SubscriberStatus>,
    pub tags: Option<Vec<String>>,
    pub custom_fields: Option<serde_json::Value>,
}

// 購読者更新リクエスト
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateSubscriberRequest {
    #[validate(email(message = "有効なメールアドレスを入力してください"))]
    pub email: Option<String>,
    pub name: Option<String>,
    pub status: Option<SubscriberStatus>,
    pub tags: Option<Vec<String>>,
    pub custom_fields: Option<serde_json::Value>,
}

// CSVインポートリクエスト
#[derive(Debug, Serialize, Deserialize)]
pub struct ImportSubscribersRequest {
    pub csv_content: String,
    pub has_header: Option<bool>,
    pub column_mapping: ColumnMapping,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ColumnMapping {
    pub email: usize,
    pub name: Option<usize>,
    pub tags: Option<Vec<usize>>,
    pub custom_fields: Option<Vec<CustomFieldMapping>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomFieldMapping {
    pub name: String,
    pub column: usize,
}

// 購読者一覧リクエストオプション
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ListSubscriberOptions {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub search: Option<String>,
    pub tag: Option<String>,
    pub status: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

// 購読者一覧レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriberListResponse {
    pub subscribers: Vec<Subscriber>,
    pub total: i64,
    pub available_tags: Vec<String>,
}

// インポート結果レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct ImportSubscribersResponse {
    pub imported_count: u32,
    pub errors: Vec<String>,
}

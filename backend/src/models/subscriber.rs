use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

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

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "subscriber_status", rename_all = "lowercase")]
pub enum SubscriberStatus {
    Active,
    Unsubscribed,
    Bounced,
    Complained,
}

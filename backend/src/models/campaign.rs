use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Campaign {
    pub id: Uuid,
    pub user_id: Uuid,
    pub template_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub subject: String,
    pub status: CampaignStatus,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub sent_at: Option<DateTime<Utc>>,
    pub recipient_count: i32,
    pub sent_count: i32,
    pub opened_count: i32,
    pub clicked_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "campaign_status", rename_all = "lowercase")]
pub enum CampaignStatus {
    Draft,
    Scheduled,
    Sending,
    Sent,
    Paused,
    Cancelled,
}

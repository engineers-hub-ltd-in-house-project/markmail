use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::fmt;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CampaignStatus {
    Draft,
    Scheduled,
    Sending,
    Sent,
    Paused,
    Cancelled,
    Error,
}

impl fmt::Display for CampaignStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            CampaignStatus::Draft => "draft",
            CampaignStatus::Scheduled => "scheduled",
            CampaignStatus::Sending => "sending",
            CampaignStatus::Sent => "sent",
            CampaignStatus::Paused => "paused",
            CampaignStatus::Cancelled => "cancelled",
            CampaignStatus::Error => "error",
        };
        write!(f, "{}", s)
    }
}

impl sqlx::Type<sqlx::Postgres> for CampaignStatus {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for CampaignStatus {
    fn decode(
        value: sqlx::postgres::PgValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let s = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match s {
            "draft" => Ok(CampaignStatus::Draft),
            "scheduled" => Ok(CampaignStatus::Scheduled),
            "sending" => Ok(CampaignStatus::Sending),
            "sent" => Ok(CampaignStatus::Sent),
            "paused" => Ok(CampaignStatus::Paused),
            "cancelled" => Ok(CampaignStatus::Cancelled),
            "error" => Ok(CampaignStatus::Error),
            other => Err(format!("Unknown campaign status: {}", other).into()),
        }
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for CampaignStatus {
    fn encode_by_ref(
        &self,
        buf: &mut <sqlx::Postgres as sqlx::database::HasArguments<'q>>::ArgumentBuffer,
    ) -> sqlx::encode::IsNull {
        let s = self.to_string();
        <String as sqlx::Encode<sqlx::Postgres>>::encode(s, buf)
    }
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateCampaignRequest {
    #[validate(length(
        min = 1,
        max = 255,
        message = "キャンペーン名は1〜255文字で指定してください"
    ))]
    pub name: String,

    pub description: Option<String>,

    #[validate(length(min = 1, max = 255, message = "件名は1〜255文字で指定してください"))]
    pub subject: String,

    pub template_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateCampaignRequest {
    #[validate(length(
        min = 1,
        max = 255,
        message = "キャンペーン名は1〜255文字で指定してください"
    ))]
    pub name: Option<String>,

    pub description: Option<String>,

    #[validate(length(min = 1, max = 255, message = "件名は1〜255文字で指定してください"))]
    pub subject: Option<String>,

    pub template_id: Option<Uuid>,

    pub status: Option<CampaignStatus>,

    pub scheduled_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScheduleCampaignRequest {
    pub scheduled_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListCampaignOptions {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub status: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CampaignListResponse {
    pub campaigns: Vec<CampaignResponse>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CampaignResponse {
    pub id: Uuid,
    pub template_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub subject: String,
    pub status: String,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub sent_at: Option<DateTime<Utc>>,
    pub stats: CampaignStats,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CampaignStats {
    pub recipient_count: i32,
    pub sent_count: i32,
    pub opened_count: i32,
    pub clicked_count: i32,
    pub open_rate: f32,
    pub click_rate: f32,
}

impl From<Campaign> for CampaignResponse {
    fn from(campaign: Campaign) -> Self {
        let open_rate = if campaign.sent_count > 0 {
            campaign.opened_count as f32 / campaign.sent_count as f32
        } else {
            0.0
        };

        let click_rate = if campaign.sent_count > 0 {
            campaign.clicked_count as f32 / campaign.sent_count as f32
        } else {
            0.0
        };

        Self {
            id: campaign.id,
            template_id: campaign.template_id,
            name: campaign.name,
            description: campaign.description,
            subject: campaign.subject,
            status: campaign.status.to_string(),
            scheduled_at: campaign.scheduled_at,
            sent_at: campaign.sent_at,
            stats: CampaignStats {
                recipient_count: campaign.recipient_count,
                sent_count: campaign.sent_count,
                opened_count: campaign.opened_count,
                clicked_count: campaign.clicked_count,
                open_rate,
                click_rate,
            },
            created_at: campaign.created_at,
            updated_at: campaign.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_campaign_status_display() {
        assert_eq!(CampaignStatus::Draft.to_string(), "draft");
        assert_eq!(CampaignStatus::Scheduled.to_string(), "scheduled");
        assert_eq!(CampaignStatus::Sending.to_string(), "sending");
        assert_eq!(CampaignStatus::Sent.to_string(), "sent");
        assert_eq!(CampaignStatus::Paused.to_string(), "paused");
        assert_eq!(CampaignStatus::Cancelled.to_string(), "cancelled");
    }

    #[test]
    fn test_campaign_response_conversion() {
        let now = Utc::now();
        let campaign = Campaign {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            template_id: Uuid::new_v4(),
            name: "Test Campaign".to_string(),
            description: Some("Test Description".to_string()),
            subject: "Test Subject".to_string(),
            status: CampaignStatus::Draft,
            scheduled_at: Some(now + Duration::days(1)),
            sent_at: None,
            recipient_count: 100,
            sent_count: 50,
            opened_count: 25,
            clicked_count: 10,
            created_at: now - Duration::hours(1),
            updated_at: now,
        };

        let response: CampaignResponse = campaign.clone().into();

        assert_eq!(response.id, campaign.id);
        assert_eq!(response.template_id, campaign.template_id);
        assert_eq!(response.name, campaign.name);
        assert_eq!(response.description, campaign.description);
        assert_eq!(response.subject, campaign.subject);
        assert_eq!(response.status, campaign.status.to_string());
        assert_eq!(response.scheduled_at, campaign.scheduled_at);
        assert_eq!(response.sent_at, campaign.sent_at);
        assert_eq!(response.stats.recipient_count, campaign.recipient_count);
        assert_eq!(response.stats.sent_count, campaign.sent_count);
        assert_eq!(response.stats.opened_count, campaign.opened_count);
        assert_eq!(response.stats.clicked_count, campaign.clicked_count);
        assert_eq!(response.stats.open_rate, 0.5); // 25/50 = 0.5
        assert_eq!(response.stats.click_rate, 0.2); // 10/50 = 0.2
        assert_eq!(response.created_at, campaign.created_at);
        assert_eq!(response.updated_at, campaign.updated_at);
    }

    #[test]
    fn test_zero_division_handling() {
        let now = Utc::now();
        let campaign = Campaign {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            template_id: Uuid::new_v4(),
            name: "Test Campaign".to_string(),
            description: None,
            subject: "Test Subject".to_string(),
            status: CampaignStatus::Draft,
            scheduled_at: None,
            sent_at: None,
            recipient_count: 100,
            sent_count: 0, // 送信数0
            opened_count: 0,
            clicked_count: 0,
            created_at: now,
            updated_at: now,
        };

        let response: CampaignResponse = campaign.into();

        // 0除算が発生しないことを確認
        assert_eq!(response.stats.open_rate, 0.0);
        assert_eq!(response.stats.click_rate, 0.0);
    }

    #[test]
    fn test_create_campaign_request_validation() {
        // 有効なリクエスト
        let valid_request = CreateCampaignRequest {
            name: "Valid Campaign".to_string(),
            description: Some("Valid description".to_string()),
            subject: "Valid subject".to_string(),
            template_id: Uuid::new_v4(),
        };
        assert!(valid_request.validate().is_ok());

        // 無効なリクエスト (空の名前)
        let invalid_name_request = CreateCampaignRequest {
            name: "".to_string(),
            description: Some("Valid description".to_string()),
            subject: "Valid subject".to_string(),
            template_id: Uuid::new_v4(),
        };
        assert!(invalid_name_request.validate().is_err());

        // 無効なリクエスト (空の件名)
        let invalid_subject_request = CreateCampaignRequest {
            name: "Valid Campaign".to_string(),
            description: Some("Valid description".to_string()),
            subject: "".to_string(),
            template_id: Uuid::new_v4(),
        };
        assert!(invalid_subject_request.validate().is_err());
    }
}

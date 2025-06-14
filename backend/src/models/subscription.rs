use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// サブスクリプションプラン
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SubscriptionPlan {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub price: i32,
    pub billing_interval: String,
    // 制限項目
    pub contact_limit: i32,
    pub monthly_email_limit: i32,
    pub campaign_limit: i32,
    pub template_limit: i32,
    pub sequence_limit: i32,
    pub sequence_step_limit: i32,
    pub form_limit: i32,
    pub form_submission_limit: i32,
    pub user_limit: i32,
    pub webhook_limit: i32,
    // 機能フラグ
    pub custom_markdown_components: bool,
    pub ai_features: bool,
    pub advanced_analytics: bool,
    pub ab_testing: bool,
    pub api_access: bool,
    pub priority_support: bool,
    pub custom_domain: bool,
    pub white_label: bool,
    // メタデータ
    pub sort_order: i32,
    pub is_active: bool,
    pub features: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl SubscriptionPlan {
    /// 指定されたメトリクスの制限値を取得
    pub fn get_limit(&self, metric_type: &str) -> i32 {
        match metric_type {
            "contacts" => self.contact_limit,
            "emails_sent" => self.monthly_email_limit,
            "campaigns" => self.campaign_limit,
            "templates" => self.template_limit,
            "sequences" => self.sequence_limit,
            "sequence_steps" => self.sequence_step_limit,
            "forms" => self.form_limit,
            "form_submissions" => self.form_submission_limit,
            "users" => self.user_limit,
            "webhooks" => self.webhook_limit,
            _ => -1,
        }
    }

    /// 指定された機能が利用可能かチェック
    pub fn has_feature(&self, feature: &str) -> bool {
        match feature {
            "custom_markdown_components" => self.custom_markdown_components,
            "ai_features" => self.ai_features,
            "advanced_analytics" => self.advanced_analytics,
            "ab_testing" => self.ab_testing,
            "api_access" => self.api_access,
            "priority_support" => self.priority_support,
            "custom_domain" => self.custom_domain,
            "white_label" => self.white_label,
            _ => false,
        }
    }
}

/// プラン一覧レスポンス
#[derive(Debug, Serialize)]
pub struct PlansResponse {
    pub plans: Vec<SubscriptionPlan>,
}

/// ユーザーのサブスクリプション
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserSubscription {
    pub id: Uuid,
    pub user_id: Uuid,
    pub plan_id: Uuid,
    pub status: String, // active, canceled, past_due, trialing
    pub current_period_start: DateTime<Utc>,
    pub current_period_end: DateTime<Utc>,
    pub cancel_at: Option<DateTime<Utc>>,
    pub canceled_at: Option<DateTime<Utc>>,
    pub trial_end: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
    pub stripe_subscription_id: Option<String>,
    pub stripe_customer_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// サブスクリプションステータス
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum SubscriptionStatus {
    #[serde(rename = "active")]
    #[sqlx(rename = "active")]
    Active,
    #[serde(rename = "canceled")]
    #[sqlx(rename = "canceled")]
    Canceled,
    #[serde(rename = "past_due")]
    #[sqlx(rename = "past_due")]
    PastDue,
    #[serde(rename = "trialing")]
    #[sqlx(rename = "trialing")]
    Trialing,
}

/// サブスクリプション詳細レスポンス
#[derive(Debug, Serialize)]
pub struct SubscriptionDetailsResponse {
    pub subscription: UserSubscription,
    pub plan: SubscriptionPlan,
    pub usage: UsageSummary,
}

/// 使用量記録
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UsageRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub metric_type: String,
    pub usage_count: i32,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 支払い履歴
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PaymentHistory {
    pub id: Uuid,
    pub user_id: Uuid,
    pub subscription_id: Option<Uuid>,
    pub amount: i32,
    pub currency: String,
    pub status: String, // succeeded, failed, pending, refunded
    pub description: Option<String>,
    pub stripe_payment_intent_id: Option<String>,
    pub stripe_invoice_id: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub paid_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// 支払いステータス
#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "text")]
pub enum PaymentStatus {
    #[serde(rename = "succeeded")]
    #[sqlx(rename = "succeeded")]
    Succeeded,
    #[serde(rename = "failed")]
    #[sqlx(rename = "failed")]
    Failed,
    #[serde(rename = "pending")]
    #[sqlx(rename = "pending")]
    Pending,
    #[serde(rename = "refunded")]
    #[sqlx(rename = "refunded")]
    Refunded,
}

/// 使用量アラート設定
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UsageAlert {
    pub id: Uuid,
    pub user_id: Uuid,
    pub metric_type: String,
    pub threshold_percentage: i32,
    pub is_enabled: bool,
    pub last_alerted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// アップグレードリクエスト
#[derive(Debug, Deserialize)]
pub struct UpgradeRequest {
    pub plan_id: Uuid,
}

/// キャンセルリクエスト
#[derive(Debug, Deserialize)]
pub struct CancelRequest {
    pub cancel_at_period_end: bool,
}

/// 使用量サマリー
#[derive(Debug, Serialize)]
pub struct UsageSummary {
    pub contacts: UsageMetric,
    pub emails_sent: UsageMetric,
    pub campaigns: UsageMetric,
    pub templates: UsageMetric,
    pub sequences: UsageMetric,
    pub forms: UsageMetric,
    pub form_submissions: UsageMetric,
}

/// 使用量メトリクス
#[derive(Debug, Serialize)]
pub struct UsageMetric {
    pub current: i32,
    pub limit: i32,
    pub percentage: f32,
}

impl UsageMetric {
    pub fn new(current: i32, limit: i32) -> Self {
        let percentage = if limit <= 0 {
            0.0
        } else {
            (current as f32 / limit as f32) * 100.0
        };

        Self {
            current,
            limit,
            percentage,
        }
    }

    pub fn is_at_limit(&self) -> bool {
        self.limit > 0 && self.current >= self.limit
    }

    pub fn is_near_limit(&self, threshold: f32) -> bool {
        self.limit > 0 && self.percentage >= threshold
    }
}

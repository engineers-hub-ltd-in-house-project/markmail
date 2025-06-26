use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

use crate::models::subscriber::Subscriber;

/// CRMプロバイダーの種類
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CrmProviderType {
    Salesforce,
    // 将来的な拡張用
    // HubSpot,
    // Pipedrive,
}

impl CrmProviderType {
    pub fn as_str(&self) -> &str {
        match self {
            CrmProviderType::Salesforce => "salesforce",
        }
    }
}

/// CRM連絡先データ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrmContact {
    pub id: Option<String>, // CRM側のID
    pub markmail_id: Uuid,  // MarkMail側のsubscriber ID
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub company: Option<String>,
    pub phone: Option<String>,
    pub tags: Vec<String>,
    pub custom_fields: HashMap<String, Value>,
    pub last_sync_at: Option<DateTime<Utc>>,
}

impl CrmContact {
    /// SubscriberからCrmContactを作成
    pub fn from_subscriber(subscriber: &Subscriber) -> Self {
        // 名前を分割（簡易版）
        let (first_name, last_name) = if let Some(full_name) = &subscriber.name {
            let parts: Vec<&str> = full_name.splitn(2, ' ').collect();
            match parts.len() {
                0 => (None, None),
                1 => (Some(parts[0].to_string()), None),
                _ => (Some(parts[0].to_string()), Some(parts[1].to_string())),
            }
        } else {
            (None, None)
        };

        // カスタムフィールドの変換
        let custom_fields = if subscriber.custom_fields.is_object() {
            subscriber
                .custom_fields
                .as_object()
                .unwrap()
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        } else {
            HashMap::new()
        };

        Self {
            id: None, // CRM側のIDはまだない
            markmail_id: subscriber.id,
            email: subscriber.email.clone(),
            first_name,
            last_name,
            company: None, // 購読者モデルには会社情報がない
            phone: None,   // 購読者モデルには電話番号がない
            tags: subscriber.tags.clone(),
            custom_fields,
            last_sync_at: None,
        }
    }
}

/// CRMキャンペーンデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrmCampaign {
    pub id: Option<String>, // CRM側のID
    pub markmail_id: Uuid,  // MarkMail側のcampaign ID
    pub name: String,
    pub status: String,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub member_count: i32,
    pub email_stats: CrmEmailStats,
}

/// メール統計情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrmEmailStats {
    pub sent: i32,
    pub opened: i32,
    pub clicked: i32,
    pub bounced: i32,
    pub unsubscribed: i32,
}

/// 同期結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrmSyncResult {
    pub entity_type: String,
    pub markmail_id: Uuid,
    pub crm_id: String,
    pub success: bool,
    pub error_message: Option<String>,
    pub synced_at: DateTime<Utc>,
}

/// 一括同期結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrmBulkSyncResult {
    pub total: usize,
    pub success: usize,
    pub failed: usize,
    pub results: Vec<CrmSyncResult>,
}

/// メールアクティビティ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrmEmailActivity {
    pub subscriber_id: Uuid,
    pub campaign_id: Option<Uuid>,
    pub activity_type: EmailActivityType,
    pub occurred_at: DateTime<Utc>,
    pub metadata: Option<HashMap<String, Value>>,
}

/// メールアクティビティの種類
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmailActivityType {
    Sent,
    Opened,
    Clicked,
    Bounced,
    Unsubscribed,
}

/// CRMリスト（Salesforceキャンペーンメンバーシップ）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrmList {
    pub id: Option<String>,
    pub name: String,
    pub member_ids: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// カスタムフィールド定義
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrmCustomField {
    pub name: String,
    pub label: String,
    pub field_type: CrmFieldType,
    pub required: bool,
    pub options: Option<Vec<String>>, // For picklist fields
}

/// フィールドタイプ
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CrmFieldType {
    Text,
    Number,
    Date,
    Boolean,
    Picklist,
    MultiPicklist,
    Email,
    Phone,
    Url,
}

/// フィールドマッピング
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrmFieldMapping {
    pub markmail_field: String,
    pub crm_field: String,
    pub field_type: CrmFieldType,
    pub sync_direction: SyncDirection,
}

/// 同期方向
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncDirection {
    ToCrm,         // MarkMail → CRM
    FromCrm,       // CRM → MarkMail
    Bidirectional, // 双方向
}

/// CRM機能フラグ
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CrmFeature {
    ContactSync,
    CampaignSync,
    CustomFields,
    BulkOperations,
    WebhookSupport,
    RealTimeSync,
}

/// CRM統合設定（データベース保存用）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CrmIntegrationSettings {
    pub sync_enabled: bool,
    pub sync_interval_minutes: i32,
    pub batch_size: usize,
    pub field_mappings: Vec<CrmFieldMapping>,
}

/// Salesforce固有の設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalesforceSettings {
    pub instance_url: String,
    pub api_version: String,
    pub sync_custom_objects: bool,
    pub campaign_member_status_mapping: HashMap<String, String>,
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

use crate::models::subscriber::Subscriber;

// Salesforce Lead カスタムフィールドマッピング定数
pub mod salesforce_field_mapping {
    // プログラミング言語スキルフィールドマッピング
    pub const PROGRAMMING_LANGUAGE_FIELDS: &[(&str, &str)] = &[
        ("java", "00NIR00000FTrIJ"),
        ("python", "00NIR00000FTrIO"),
        ("javascript_typescript", "00NIR00000FTrIT"),
        ("c_cpp", "00NIR00000FTrIY"),
        ("csharp", "00NIR00000FTrId"),
        ("php", "00NIR00000FTrIi"),
        ("go", "00NIR00000FTrIn"),
        ("ruby", "00NIR00000FTrIs"),
        ("swift", "00NIR00000FTrIx"),
        ("kotlin", "00NIR00000FTrJ2"),
    ];

    // フレームワーク・技術スタックフィールドマッピング
    pub const TECH_STACK_FIELDS: &[(&str, &str)] = &[
        ("react", "00NIR00000FTrNZ"),
        ("nextjs", "00NIR00000FTrNe"),
        ("django", "00NIR00000FTrNj"),
        ("ruby_on_rails", "00NIR00000FTrNo"),
        ("react_native", "00NIR00000FTrNt"),
        ("postgresql", "00NIR00000FTrNy"),
        ("sql_server", "00NIR00000FTrO3"),
        ("kubernetes", "00NIR00000FTrO8"),
        ("azure", "00NIR00000FTrOD"),
        ("vue_js", "00NIR00000FTrOI"),
        ("svelte", "00NIR00000FTrON"),
        ("flask", "00NIR00000FTrOS"),
        ("laravel", "00NIR00000FTrOX"),
        ("flutter", "00NIR00000FTrOc"),
        ("mongodb", "00NIR00000FTrOh"),
        ("redis", "00NIR00000FTrOm"),
        ("aws", "00NIR00000FTrOr"),
        ("jenkins", "00NIR00000FTrOw"),
        ("angular", "00NIR00000FTrP1"),
        ("spring", "00NIR00000FTrP6"),
        ("express", "00NIR00000FTrPB"),
        ("asp_net", "00NIR00000FTrPG"),
        ("mysql", "00NIR00000FTrPL"),
        ("oracle", "00NIR00000FTrPQ"),
        ("docker", "00NIR00000FTrPV"),
        ("gcp", "00NIR00000FTrPa"),
        ("github_actions", "00NIR00000FTrPf"),
    ];

    // その他のカスタムフィールドマッピング
    pub const OTHER_FIELDS: &[(&str, &str)] = &[
        ("github_url", "00NIR00000FTrJC"),
        ("portfolio_url", "00NIR00000FTrJH"),
        ("experience", "00NIR00000FTrJM"),
        ("newsletter_opt_in", "00NIR00000FTrVO"),
        ("state", "State"), // 標準フィールド
    ];
}

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

/// CRMリードデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrmLead {
    pub id: Option<String>,                   // CRM側のID
    pub markmail_form_id: Uuid,               // MarkMail側のform ID
    pub markmail_submission_id: Option<Uuid>, // MarkMail側のform submission ID
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub company: Option<String>,
    pub phone: Option<String>,
    pub title: Option<String>,
    pub website: Option<String>,
    pub lead_source: String,    // e.g., "Web Form", "MarkMail Form"
    pub status: Option<String>, // e.g., "New", "Contacted", "Qualified"
    pub description: Option<String>,
    pub custom_fields: HashMap<String, Value>,
    pub created_at: DateTime<Utc>,
}

impl CrmLead {
    /// フォーム送信データからCrmLeadを作成
    pub fn from_form_submission(
        form_id: Uuid,
        submission_id: Option<Uuid>,
        form_data: &Value,
        form_fields: &Value,
        form_name: &str,
    ) -> Self {
        let mut lead = Self {
            id: None,
            markmail_form_id: form_id,
            markmail_submission_id: submission_id,
            email: String::new(),
            first_name: None,
            last_name: None,
            company: None,
            phone: None,
            title: None,
            website: None,
            lead_source: format!("MarkMail Form: {}", form_name),
            status: Some("New".to_string()),
            description: None,
            custom_fields: HashMap::new(),
            created_at: Utc::now(),
        };

        // フォームデータから各フィールドを抽出
        if let (Value::Object(data), Value::Array(fields)) = (form_data, form_fields) {
            for field in fields {
                if let Value::Object(field_obj) = field {
                    if let (Some(Value::String(field_type)), Some(Value::String(field_name))) =
                        (field_obj.get("field_type"), field_obj.get("name"))
                    {
                        if let Some(value) = data.get(field_name) {
                            match field_type.as_str() {
                                "email" => {
                                    if let Value::String(email) = value {
                                        lead.email = email.clone();
                                    }
                                }
                                "text" => {
                                    if let Value::String(text) = value {
                                        // フィールド名に基づいて適切なフィールドに設定
                                        if field_name.contains("first") || field_name.contains("名")
                                        {
                                            lead.first_name = Some(text.clone());
                                        } else if field_name.contains("last")
                                            || field_name.contains("姓")
                                        {
                                            lead.last_name = Some(text.clone());
                                        } else if field_name.contains("company")
                                            || field_name.contains("会社")
                                        {
                                            lead.company = Some(text.clone());
                                        } else if field_name.contains("title")
                                            || field_name.contains("役職")
                                        {
                                            lead.title = Some(text.clone());
                                        } else if field_name.contains("name")
                                            || field_name.contains("名前")
                                        {
                                            // フルネームの場合は分割
                                            let parts: Vec<&str> = text.splitn(2, ' ').collect();
                                            if parts.len() == 2 {
                                                lead.first_name = Some(parts[0].to_string());
                                                lead.last_name = Some(parts[1].to_string());
                                            } else {
                                                lead.first_name = Some(text.clone());
                                            }
                                        } else {
                                            // その他のテキストフィールドはカスタムフィールドとして保存
                                            lead.custom_fields
                                                .insert(field_name.clone(), value.clone());
                                        }
                                    }
                                }
                                "phone" => {
                                    if let Value::String(phone) = value {
                                        lead.phone = Some(phone.clone());
                                    }
                                }
                                "url" => {
                                    if let Value::String(url) = value {
                                        if field_name.contains("website")
                                            || field_name.contains("web")
                                        {
                                            lead.website = Some(url.clone());
                                        } else {
                                            lead.custom_fields
                                                .insert(field_name.clone(), value.clone());
                                        }
                                    }
                                }
                                _ => {
                                    // その他のフィールドタイプはカスタムフィールドとして保存
                                    lead.custom_fields.insert(field_name.clone(), value.clone());
                                }
                            }
                        }
                    }
                }
            }

            // Salesforceカスタムフィールドマッピングの処理
            use crate::models::crm::salesforce_field_mapping::{
                OTHER_FIELDS, PROGRAMMING_LANGUAGE_FIELDS, TECH_STACK_FIELDS,
            };

            // プログラミング言語フィールドのマッピング
            for (field_name, sf_field_id) in PROGRAMMING_LANGUAGE_FIELDS {
                if let Some(value) = data.get(*field_name) {
                    if !value.is_null() {
                        lead.custom_fields
                            .insert(sf_field_id.to_string(), value.clone());
                    }
                }
            }

            // 技術スタックフィールドのマッピング
            for (field_name, sf_field_id) in TECH_STACK_FIELDS {
                if let Some(value) = data.get(*field_name) {
                    if !value.is_null() {
                        lead.custom_fields
                            .insert(sf_field_id.to_string(), value.clone());
                    }
                }
            }

            // その他のカスタムフィールドのマッピング
            for (field_name, sf_field_id) in OTHER_FIELDS {
                if let Some(value) = data.get(*field_name) {
                    if !value.is_null() {
                        // 都道府県は標準フィールドなので特別扱い
                        if *field_name == "state" {
                            // Stateフィールドは後でSalesforceプロバイダーで処理
                            lead.custom_fields
                                .insert("State".to_string(), value.clone());
                        } else {
                            lead.custom_fields
                                .insert(sf_field_id.to_string(), value.clone());
                        }
                    }
                }
            }
        }

        // フォーム送信の詳細を説明に追加
        lead.description = Some(format!(
            "Lead created from MarkMail form submission.\nForm: {}\nSubmission ID: {}\nSubmitted at: {}",
            form_name,
            submission_id.map(|id| id.to_string()).unwrap_or_else(|| "N/A".to_string()),
            lead.created_at.format("%Y-%m-%d %H:%M:%S UTC")
        ));

        lead
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

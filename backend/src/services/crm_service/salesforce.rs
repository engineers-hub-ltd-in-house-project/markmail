use async_trait::async_trait;
use chrono::Utc;
use rustforce::response::QueryResponse;
use rustforce::Client as SalesforceClient;
use salesforce_bulk_api::{BulkClient, JobConfig, Operation as BulkOperation};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::models::crm::{
    CrmBulkSyncResult, CrmCampaign, CrmContact, CrmCustomField, CrmEmailActivity, CrmFeature,
    CrmFieldMapping, CrmLead, CrmList, CrmSyncResult, EmailActivityType,
};

use super::{CrmError, CrmProvider, SalesforceConfig};

/// Salesforce Attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalesforceAttributes {
    pub url: String,
    #[serde(rename = "type")]
    pub sobject_type: String,
}

/// Salesforce Contact オブジェクト
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SalesforceContact {
    #[serde(rename = "attributes")]
    pub attributes: Option<SalesforceAttributes>,
    #[serde(rename = "Id")]
    pub id: Option<String>,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub account_id: Option<String>,
    pub phone: Option<String>,
    pub mailing_street: Option<String>,
    pub mailing_city: Option<String>,
    pub mailing_state: Option<String>,
    pub mailing_postal_code: Option<String>,
    pub mailing_country: Option<String>,
    #[serde(rename = "MarkMail_ID__c")]
    pub markmail_id: Option<String>,
}

/// Salesforce Lead オブジェクト
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SalesforceLead {
    #[serde(rename = "attributes")]
    pub attributes: Option<SalesforceAttributes>,
    #[serde(rename = "Id")]
    pub id: Option<String>,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: String, // Required in Salesforce
    pub company: String,   // Required in Salesforce
    pub phone: Option<String>,
    pub title: Option<String>,
    pub website: Option<String>,
    pub lead_source: Option<String>,
    pub status: Option<String>,
    // pub description: Option<String>, // Not available in this Salesforce org
    #[serde(rename = "MarkMail_Form_ID__c")]
    pub markmail_form_id: Option<String>,
    #[serde(rename = "MarkMail_Submission_ID__c")]
    pub markmail_submission_id: Option<String>,
}

/// Salesforce Campaign オブジェクト
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SalesforceCampaign {
    #[serde(rename = "attributes")]
    pub attributes: Option<SalesforceAttributes>,
    #[serde(rename = "Id")]
    pub id: Option<String>,
    pub name: String,
    pub status: String,
    pub r#type: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub number_of_contacts: Option<i32>,
    pub number_of_responses: Option<i32>,
    #[serde(rename = "MarkMail_Campaign_ID__c")]
    pub markmail_campaign_id: Option<String>,
}

/// Salesforce Task オブジェクト（メールアクティビティ用）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SalesforceTask {
    #[serde(rename = "attributes")]
    pub attributes: Option<SalesforceAttributes>,
    #[serde(rename = "Id")]
    pub id: Option<String>,
    pub subject: String,
    pub status: String,
    pub priority: String,
    pub who_id: Option<String>,  // Contact/Lead ID
    pub what_id: Option<String>, // Campaign ID など
    pub activity_date: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "Type")]
    pub task_type: Option<String>,
}

/// CSV形式のコンタクトデータ
#[derive(Debug, Serialize, Deserialize)]
pub struct ContactCsvRecord {
    #[serde(rename = "FirstName")]
    pub first_name: String,
    #[serde(rename = "LastName")]
    pub last_name: String,
    #[serde(rename = "Email")]
    pub email: String,
    #[serde(rename = "Phone")]
    pub phone: String,
    #[serde(rename = "MarkMail_ID__c")]
    pub markmail_id: String,
}

/// Salesforceプロバイダー
pub struct SalesforceProvider {
    config: SalesforceConfig,
    bulk_client: Option<BulkClient>,
}

impl SalesforceProvider {
    /// 新しいSalesforceプロバイダーを作成
    pub async fn new(config: SalesforceConfig) -> Result<Self, CrmError> {
        // Bulk APIクライアントを作成（アクセストークンがある場合のみ）
        let bulk_client = if !config.access_token.is_empty() {
            match BulkClient::new(&config.instance_url, &config.access_token) {
                Ok(client) => Some(client),
                Err(e) => {
                    tracing::warn!("Failed to create Bulk API client: {}", e);
                    None
                }
            }
        } else {
            None
        };

        Ok(Self {
            config,
            bulk_client,
        })
    }

    /// Salesforceクライアントを取得
    fn get_client(&self) -> Result<SalesforceClient, CrmError> {
        // rustforceクライアントを作成
        let mut client = SalesforceClient::new(
            // OAuth2クライアントIDとシークレットは通常環境変数から取得
            Some(
                std::env::var("SALESFORCE_CLIENT_ID")
                    .unwrap_or_else(|_| "dummy_client_id".to_string()),
            ),
            Some(
                std::env::var("SALESFORCE_CLIENT_SECRET")
                    .unwrap_or_else(|_| "dummy_client_secret".to_string()),
            ),
        );

        // アクセストークンを直接設定（SF CLIから取得済みの場合）
        client.set_access_token(&self.config.access_token);
        client.set_instance_url(&self.config.instance_url);

        Ok(client)
    }

    /// MarkMail ContactをSalesforce Contactに変換
    fn to_salesforce_contact(&self, contact: &CrmContact) -> SalesforceContact {
        SalesforceContact {
            attributes: None, // 作成/更新時は不要
            id: contact.id.clone(),
            email: contact.email.clone(),
            first_name: contact.first_name.clone(),
            last_name: contact.last_name.clone(),
            account_id: None,
            phone: contact.phone.clone(),
            mailing_street: None,
            mailing_city: None,
            mailing_state: None,
            mailing_postal_code: None,
            mailing_country: None,
            markmail_id: Some(contact.markmail_id.to_string()),
        }
    }

    /// CrmLeadをSalesforceLeadに変換
    fn to_salesforce_lead(&self, lead: &CrmLead) -> SalesforceLead {
        SalesforceLead {
            attributes: None, // 作成/更新時は不要
            id: lead.id.clone(),
            email: lead.email.clone(),
            first_name: lead.first_name.clone(),
            // Salesforceでは姓は必須なので、なければ"Unknown"を設定
            last_name: lead
                .last_name
                .clone()
                .unwrap_or_else(|| "Unknown".to_string()),
            // Salesforceでは会社名は必須なので、なければ"Unknown"を設定
            company: lead
                .company
                .clone()
                .unwrap_or_else(|| "Unknown Company".to_string()),
            phone: lead.phone.clone(),
            title: lead.title.clone(),
            website: lead.website.clone(),
            lead_source: Some(lead.lead_source.clone()),
            status: lead.status.clone(),
            // description: lead.description.clone(), // Not available in this Salesforce org
            markmail_form_id: Some(lead.markmail_form_id.to_string()),
            markmail_submission_id: lead.markmail_submission_id.map(|id| id.to_string()),
        }
    }

    /// カスタムフィールドを含むリードパラメータを構築
    fn build_lead_params(&self, lead: &CrmLead) -> Result<Value, CrmError> {
        let sf_lead = self.to_salesforce_lead(lead);
        let mut params =
            serde_json::to_value(&sf_lead).map_err(|e| CrmError::DataConversion(e.to_string()))?;

        // カスタムフィールドを追加
        if let Value::Object(ref mut map) = params {
            for (field_id, value) in &lead.custom_fields {
                // Stateは標準フィールドなので特別扱い
                if field_id == "State" {
                    map.insert("State".to_string(), value.clone());
                } else if field_id.starts_with("00NIR") {
                    // カスタムフィールドIDのマッピング
                    // APIでは__cサフィックスが必要
                    let field_name = match field_id.as_str() {
                        "00NIR00000FTrIJ" => "Java__c",
                        "00NIR00000FTrIO" => "Python__c",
                        "00NIR00000FTrIT" => "JavaScript_TypeScript__c",
                        "00NIR00000FTrIY" => "C_C__c",
                        "00NIR00000FTrId" => "C__c",
                        "00NIR00000FTrIi" => "PHP__c",
                        "00NIR00000FTrIn" => "Go__c",
                        "00NIR00000FTrIs" => "Ruby__c",
                        "00NIR00000FTrIx" => "Swift__c",
                        "00NIR00000FTrJ2" => "Kotlin__c",
                        "00NIR00000FTrNZ" => "React__c",
                        "00NIR00000FTrNe" => "Next_js__c",
                        "00NIR00000FTrNj" => "Django__c",
                        "00NIR00000FTrNo" => "Ruby_on_Rails__c",
                        "00NIR00000FTrNt" => "React_Native__c",
                        "00NIR00000FTrNy" => "PostgreSQL__c",
                        "00NIR00000FTrO3" => "SQL_Server__c",
                        "00NIR00000FTrO8" => "Kubernetes__c",
                        "00NIR00000FTrOD" => "Azure__c",
                        "00NIR00000FTrOI" => "Vue_js__c",
                        "00NIR00000FTrON" => "Svelte__c",
                        "00NIR00000FTrOS" => "Flask__c",
                        "00NIR00000FTrOX" => "Laravel__c",
                        "00NIR00000FTrOc" => "Flutter__c",
                        "00NIR00000FTrOh" => "MongoDB__c",
                        "00NIR00000FTrOm" => "Redis__c",
                        "00NIR00000FTrOr" => "AWS__c",
                        "00NIR00000FTrOw" => "Jenkins__c",
                        "00NIR00000FTrP1" => "Angular__c",
                        "00NIR00000FTrP6" => "Spring__c",
                        "00NIR00000FTrPB" => "Express__c",
                        "00NIR00000FTrPG" => "ASP_NET__c",
                        "00NIR00000FTrPL" => "MySQL__c",
                        "00NIR00000FTrPQ" => "Oracle__c",
                        "00NIR00000FTrPV" => "Docker__c",
                        "00NIR00000FTrPa" => "GCP__c",
                        "00NIR00000FTrPf" => "GitHub_Actions__c",
                        "00NIR00000FTrJC" => "GitHub_URL__c",
                        "00NIR00000FTrJH" => "URL__c",
                        "00NIR00000FTrJM" => "PR__c",
                        "00NIR00000FTrVO" => "Co_Ltd__c",
                        _ => field_id,
                    };
                    map.insert(field_name.to_string(), value.clone());
                }
            }
        }

        Ok(params)
    }

    /// Salesforce ContactをMarkMail CrmContactに変換
    fn salesforce_contact_to_crm(
        &self,
        sf_contact: &SalesforceContact,
        markmail_id: Option<Uuid>,
    ) -> CrmContact {
        CrmContact {
            id: sf_contact.id.clone(),
            markmail_id: markmail_id.unwrap_or_else(|| {
                // MarkMail IDがカスタムフィールドに保存されていれば使用
                sf_contact
                    .markmail_id
                    .as_ref()
                    .and_then(|id| Uuid::parse_str(id).ok())
                    .unwrap_or_else(Uuid::new_v4)
            }),
            email: sf_contact.email.clone(),
            first_name: sf_contact.first_name.clone(),
            last_name: sf_contact.last_name.clone(),
            company: None, // AccountからCompany名を取得する必要がある
            phone: sf_contact.phone.clone(),
            tags: Vec::new(),
            custom_fields: std::collections::HashMap::new(),
            last_sync_at: Some(Utc::now()),
        }
    }

    /// メールアクティビティタイプをSalesforce Taskタイプに変換
    fn activity_type_to_task_type(&self, activity_type: &EmailActivityType) -> &'static str {
        match activity_type {
            EmailActivityType::Sent => "Email Sent",
            EmailActivityType::Opened => "Email Opened",
            EmailActivityType::Clicked => "Email Clicked",
            EmailActivityType::Bounced => "Email Bounced",
            EmailActivityType::Unsubscribed => "Email Unsubscribed",
        }
    }
}

#[async_trait]
impl CrmProvider for SalesforceProvider {
    async fn sync_contact(&self, contact: &CrmContact) -> Result<CrmSyncResult, CrmError> {
        let client = self.get_client()?;
        let sf_contact = self.to_salesforce_contact(contact);

        let result = if let Some(id) = &contact.id {
            // 既存のContactを更新
            let params = serde_json::to_value(&sf_contact)
                .map_err(|e| CrmError::DataConversion(e.to_string()))?;

            client
                .update("Contact", id, params)
                .await
                .map_err(|e| CrmError::ApiError(e.to_string()))?;

            CrmSyncResult {
                entity_type: "Contact".to_string(),
                markmail_id: contact.markmail_id,
                crm_id: id.clone(),
                success: true,
                error_message: None,
                synced_at: Utc::now(),
            }
        } else {
            // 新規Contactを作成
            let params = serde_json::to_value(&sf_contact)
                .map_err(|e| CrmError::DataConversion(e.to_string()))?;

            let create_result = client
                .create("Contact", params)
                .await
                .map_err(|e| CrmError::ApiError(e.to_string()))?;

            CrmSyncResult {
                entity_type: "Contact".to_string(),
                markmail_id: contact.markmail_id,
                crm_id: create_result.id,
                success: true,
                error_message: None,
                synced_at: Utc::now(),
            }
        };

        Ok(result)
    }

    async fn get_contact(&self, email: &str) -> Result<Option<CrmContact>, CrmError> {
        let client = self.get_client()?;

        let query = format!(
            "SELECT Id, Email, FirstName, LastName, Phone, MarkMail_ID__c \
             FROM Contact WHERE Email = '{}'",
            email
        );

        let result: QueryResponse<SalesforceContact> = client
            .query(&query)
            .await
            .map_err(|e| CrmError::ApiError(e.to_string()))?;

        if let Some(sf_contact) = result.records.into_iter().next() {
            Ok(Some(self.salesforce_contact_to_crm(&sf_contact, None)))
        } else {
            Ok(None)
        }
    }

    async fn update_contact(&self, contact: &CrmContact) -> Result<CrmSyncResult, CrmError> {
        if contact.id.is_none() {
            return Err(CrmError::DataConversion("Contact IDが必要です".to_string()));
        }

        self.sync_contact(contact).await
    }

    async fn delete_contact(&self, id: &str) -> Result<(), CrmError> {
        let client = self.get_client()?;

        client
            .destroy("Contact", id)
            .await
            .map_err(|e| CrmError::ApiError(e.to_string()))?;

        Ok(())
    }

    async fn bulk_sync_contacts(
        &self,
        contacts: Vec<CrmContact>,
    ) -> Result<CrmBulkSyncResult, CrmError> {
        let _total = contacts.len();

        // Bulk APIクライアントが利用可能な場合は使用
        if let Some(bulk_client) = &self.bulk_client {
            // ContactをCSV形式に変換
            let mut csv_data = String::from("FirstName,LastName,Email,Phone,MarkMail_ID__c\n");

            for contact in &contacts {
                csv_data.push_str(&format!(
                    "{},{},{},{},{}\n",
                    contact.first_name.as_deref().unwrap_or(""),
                    contact.last_name.as_deref().unwrap_or(""),
                    contact.email,
                    contact.phone.as_deref().unwrap_or(""),
                    contact.markmail_id
                ));
            }

            // Bulk API 2.0を使用した同期処理
            match self
                .execute_bulk_sync(bulk_client, "Contact", &csv_data, &contacts)
                .await
            {
                Ok(results) => Ok(results),
                Err(e) => {
                    tracing::error!(
                        "Bulk API sync failed, falling back to individual sync: {}",
                        e
                    );
                    // Bulk APIが失敗した場合は個別同期にフォールバック
                    self.sync_contacts_individually(contacts).await
                }
            }
        } else {
            // Bulk APIクライアントがない場合は個別同期
            self.sync_contacts_individually(contacts).await
        }
    }

    async fn sync_campaign(&self, campaign: &CrmCampaign) -> Result<CrmSyncResult, CrmError> {
        let client = self.get_client()?;

        let sf_campaign = SalesforceCampaign {
            attributes: None,
            id: campaign.id.clone(),
            name: campaign.name.clone(),
            status: campaign.status.clone(),
            r#type: Some("Email".to_string()),
            start_date: campaign
                .start_date
                .map(|d| d.format("%Y-%m-%d").to_string()),
            end_date: campaign.end_date.map(|d| d.format("%Y-%m-%d").to_string()),
            number_of_contacts: Some(campaign.member_count),
            number_of_responses: Some(campaign.email_stats.opened),
            markmail_campaign_id: Some(campaign.markmail_id.to_string()),
        };

        let result = if let Some(id) = &campaign.id {
            // 既存のCampaignを更新
            let params = serde_json::to_value(&sf_campaign)
                .map_err(|e| CrmError::DataConversion(e.to_string()))?;

            client
                .update("Campaign", id, params)
                .await
                .map_err(|e| CrmError::ApiError(e.to_string()))?;

            CrmSyncResult {
                entity_type: "Campaign".to_string(),
                markmail_id: campaign.markmail_id,
                crm_id: id.clone(),
                success: true,
                error_message: None,
                synced_at: Utc::now(),
            }
        } else {
            // 新規Campaignを作成
            let params = serde_json::to_value(&sf_campaign)
                .map_err(|e| CrmError::DataConversion(e.to_string()))?;

            let create_result = client
                .create("Campaign", params)
                .await
                .map_err(|e| CrmError::ApiError(e.to_string()))?;

            CrmSyncResult {
                entity_type: "Campaign".to_string(),
                markmail_id: campaign.markmail_id,
                crm_id: create_result.id,
                success: true,
                error_message: None,
                synced_at: Utc::now(),
            }
        };

        Ok(result)
    }

    async fn log_email_activity(&self, activity: &CrmEmailActivity) -> Result<(), CrmError> {
        let client = self.get_client()?;

        // 対応するContactを検索
        let contact_result = client
            .query::<SalesforceContact>(&format!(
                "SELECT Id FROM Contact WHERE MarkMail_ID__c = '{}'",
                activity.subscriber_id
            ))
            .await
            .map_err(|e| CrmError::ApiError(e.to_string()))?;

        if let Some(contact) = contact_result.records.into_iter().next() {
            let task = SalesforceTask {
                attributes: None,
                id: None,
                subject: format!(
                    "Email Activity: {}",
                    self.activity_type_to_task_type(&activity.activity_type)
                ),
                status: "Completed".to_string(),
                priority: "Normal".to_string(),
                who_id: contact.id,
                what_id: activity.campaign_id.map(|id| id.to_string()),
                activity_date: Some(activity.occurred_at.format("%Y-%m-%d").to_string()),
                description: Some(format!(
                    "Email activity recorded from MarkMail: {:?}",
                    activity.activity_type
                )),
                task_type: Some("Email".to_string()),
            };

            let params =
                serde_json::to_value(&task).map_err(|e| CrmError::DataConversion(e.to_string()))?;

            client
                .create("Task", params)
                .await
                .map_err(|e| CrmError::ApiError(e.to_string()))?;
        }

        Ok(())
    }

    async fn sync_list_membership(&self, _list: &CrmList) -> Result<(), CrmError> {
        // TODO: CampaignMember の同期を実装
        Err(CrmError::Unknown(
            "リストメンバーシップ同期は未実装です".to_string(),
        ))
    }

    async fn create_lead(&self, lead: &CrmLead) -> Result<CrmSyncResult, CrmError> {
        let client = self.get_client()?;

        // 既存のリードを確認（重複防止）
        let query = format!(
            "SELECT Id, Email FROM Lead WHERE Email = '{}' LIMIT 1",
            lead.email
        );

        let existing_lead: QueryResponse<SalesforceLead> = client
            .query(&query)
            .await
            .map_err(|e| CrmError::ApiError(format!("リード検索エラー: {}", e)))?;

        let (crm_id, _created) = if let Some(existing) = existing_lead.records.first() {
            // 既存のリードがある場合は更新
            if let Some(id) = &existing.id {
                // カスタムフィールドを含む完全なパラメータを構築
                let params = self.build_lead_params(lead)?;

                client
                    .update("Lead", id, params)
                    .await
                    .map_err(|e| CrmError::ApiError(format!("リード更新エラー: {}", e)))?;

                (id.clone(), false)
            } else {
                return Err(CrmError::ApiError(
                    "既存リードのIDが取得できません".to_string(),
                ));
            }
        } else {
            // 新規リードを作成
            // カスタムフィールドを含む完全なパラメータを構築
            let params = self.build_lead_params(lead)?;

            let result = client
                .create("Lead", params)
                .await
                .map_err(|e| CrmError::ApiError(format!("リード作成エラー: {}", e)))?;

            (result.id, true)
        };

        Ok(CrmSyncResult {
            entity_type: "Lead".to_string(),
            markmail_id: lead.markmail_form_id,
            crm_id,
            success: true,
            error_message: None,
            synced_at: Utc::now(),
        })
    }

    async fn get_lead(&self, email: &str) -> Result<Option<CrmLead>, CrmError> {
        let client = self.get_client()?;
        let query = format!(
            "SELECT Id, Email, FirstName, LastName, Company, Phone, Title, Website, \
             LeadSource, Status, MarkMail_Form_ID__c, MarkMail_Submission_ID__c \
             FROM Lead WHERE Email = '{}' LIMIT 1",
            email
        );

        let response: QueryResponse<SalesforceLead> = client
            .query(&query)
            .await
            .map_err(|e| CrmError::ApiError(format!("リード取得エラー: {}", e)))?;

        if let Some(sf_lead) = response.records.first() {
            Ok(Some(CrmLead {
                id: sf_lead.id.clone(),
                markmail_form_id: sf_lead
                    .markmail_form_id
                    .as_ref()
                    .and_then(|id| Uuid::parse_str(id).ok())
                    .unwrap_or_default(),
                markmail_submission_id: sf_lead
                    .markmail_submission_id
                    .as_ref()
                    .and_then(|id| Uuid::parse_str(id).ok()),
                email: sf_lead.email.clone(),
                first_name: sf_lead.first_name.clone(),
                last_name: Some(sf_lead.last_name.clone()),
                company: Some(sf_lead.company.clone()),
                phone: sf_lead.phone.clone(),
                title: sf_lead.title.clone(),
                website: sf_lead.website.clone(),
                lead_source: sf_lead.lead_source.clone().unwrap_or_default(),
                status: sf_lead.status.clone(),
                description: None, // Not available in this Salesforce org
                custom_fields: std::collections::HashMap::new(),
                created_at: Utc::now(), // TODO: Salesforceから実際の作成日時を取得
            }))
        } else {
            Ok(None)
        }
    }

    async fn convert_lead_to_contact(&self, _lead_id: &str) -> Result<CrmSyncResult, CrmError> {
        // Salesforceのリード変換はより複雑なので、現時点では未実装
        Err(CrmError::Unknown("リード変換機能は未実装です".to_string()))
    }

    async fn get_custom_fields(&self) -> Result<Vec<CrmCustomField>, CrmError> {
        // TODO: Describe APIを使用してカスタムフィールドを取得
        Err(CrmError::Unknown(
            "カスタムフィールド取得は未実装です".to_string(),
        ))
    }

    async fn map_custom_fields(&self, _mapping: &CrmFieldMapping) -> Result<(), CrmError> {
        // TODO: フィールドマッピングの保存
        Err(CrmError::Unknown(
            "フィールドマッピングは未実装です".to_string(),
        ))
    }

    fn provider_name(&self) -> &str {
        "Salesforce"
    }

    fn supports_feature(&self, feature: CrmFeature) -> bool {
        match feature {
            CrmFeature::ContactSync => true,
            CrmFeature::CampaignSync => true,
            CrmFeature::CustomFields => true, // カスタムフィールドは実装済み
            CrmFeature::BulkOperations => self.bulk_client.is_some(), // Bulk APIクライアントがある場合true
            CrmFeature::WebhookSupport => false,                      // TODO: Phase 5で実装
            CrmFeature::RealTimeSync => false,                        // TODO: Phase 5で実装
        }
    }
}

impl SalesforceProvider {
    /// 個別同期処理（フォールバック用）
    async fn sync_contacts_individually(
        &self,
        contacts: Vec<CrmContact>,
    ) -> Result<CrmBulkSyncResult, CrmError> {
        let mut results = Vec::new();
        let total = contacts.len();
        let mut success = 0;
        let mut failed = 0;

        for contact in contacts {
            match self.sync_contact(&contact).await {
                Ok(result) => {
                    if result.success {
                        success += 1;
                    } else {
                        failed += 1;
                    }
                    results.push(result);
                }
                Err(e) => {
                    failed += 1;
                    results.push(CrmSyncResult {
                        entity_type: "Contact".to_string(),
                        markmail_id: contact.markmail_id,
                        crm_id: contact.id.unwrap_or_default(),
                        success: false,
                        error_message: Some(e.to_string()),
                        synced_at: Utc::now(),
                    });
                }
            }
        }

        Ok(CrmBulkSyncResult {
            total,
            success,
            failed,
            results,
        })
    }

    /// Bulk API 2.0を使用した同期処理
    async fn execute_bulk_sync(
        &self,
        bulk_client: &BulkClient,
        object_type: &str,
        csv_data: &str,
        contacts: &[CrmContact],
    ) -> Result<CrmBulkSyncResult, CrmError> {
        // ジョブ作成
        let job_config = JobConfig::new(object_type, BulkOperation::Insert);
        let job = bulk_client
            .create_job(job_config)
            .await
            .map_err(|e| CrmError::ApiError(format!("Failed to create bulk job: {}", e)))?;

        // データアップロード
        bulk_client
            .upload_data(&job.id, csv_data)
            .await
            .map_err(|e| CrmError::ApiError(format!("Failed to upload bulk data: {}", e)))?;

        // ジョブ開始
        let job = bulk_client
            .start_job(&job.id)
            .await
            .map_err(|e| CrmError::ApiError(format!("Failed to start bulk job: {}", e)))?;

        // 完了待機
        let completed_job = bulk_client
            .wait_for_completion(&job.id)
            .await
            .map_err(|e| CrmError::ApiError(format!("Bulk job failed: {}", e)))?;

        // 結果の処理
        let mut results = Vec::new();
        let success =
            (completed_job.number_records_processed - completed_job.number_records_failed) as usize;
        let failed = completed_job.number_records_failed as usize;

        // 成功レコードの処理
        if success > 0 {
            let _successful_csv = bulk_client
                .get_successful_records(&job.id)
                .await
                .unwrap_or_default();

            // CSV解析して結果を作成（簡易版）
            for (i, contact) in contacts.iter().enumerate() {
                if i < success {
                    results.push(CrmSyncResult {
                        entity_type: object_type.to_string(),
                        markmail_id: contact.markmail_id,
                        crm_id: contact.id.clone().unwrap_or_default(),
                        success: true,
                        error_message: None,
                        synced_at: Utc::now(),
                    });
                }
            }
        }

        // 失敗レコードの処理
        if failed > 0 {
            let _failed_csv = bulk_client
                .get_failed_records(&job.id)
                .await
                .unwrap_or_default();

            // CSV解析してエラーメッセージを取得（簡易版）
            for contact in contacts.iter().skip(success).take(failed) {
                results.push(CrmSyncResult {
                    entity_type: object_type.to_string(),
                    markmail_id: contact.markmail_id,
                    crm_id: contact.id.clone().unwrap_or_default(),
                    success: false,
                    error_message: Some("Bulk sync failed".to_string()),
                    synced_at: Utc::now(),
                });
            }
        }

        Ok(CrmBulkSyncResult {
            total: contacts.len(),
            success,
            failed,
            results,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_provider_name() {
        let config = SalesforceConfig {
            org_alias: "test".to_string(),
            api_version: "v60.0".to_string(),
            instance_url: "https://test.salesforce.com".to_string(),
            access_token: "test_token".to_string(),
            refresh_token: None,
        };

        let provider = SalesforceProvider::new(config).await.unwrap();
        assert_eq!(provider.provider_name(), "Salesforce");
    }

    #[tokio::test]
    async fn test_supports_feature() {
        let config = SalesforceConfig {
            org_alias: "test".to_string(),
            api_version: "v60.0".to_string(),
            instance_url: "https://test.salesforce.com".to_string(),
            access_token: "test_token".to_string(),
            refresh_token: None,
        };

        let provider = SalesforceProvider::new(config).await.unwrap();
        assert!(provider.supports_feature(CrmFeature::ContactSync));
        assert!(provider.supports_feature(CrmFeature::CampaignSync));
        assert!(provider.supports_feature(CrmFeature::CustomFields));
        // Bulk APIクライアントが作成される場合はtrue
        assert!(provider.supports_feature(CrmFeature::BulkOperations));
        assert!(!provider.supports_feature(CrmFeature::WebhookSupport));
        assert!(!provider.supports_feature(CrmFeature::RealTimeSync));
    }

    #[tokio::test]
    async fn test_activity_type_conversion() {
        let config = SalesforceConfig {
            org_alias: "test".to_string(),
            api_version: "v60.0".to_string(),
            instance_url: "https://test.salesforce.com".to_string(),
            access_token: "test_token".to_string(),
            refresh_token: None,
        };

        let provider = SalesforceProvider::new(config).await.unwrap();

        assert_eq!(
            provider.activity_type_to_task_type(&EmailActivityType::Sent),
            "Email Sent"
        );
        assert_eq!(
            provider.activity_type_to_task_type(&EmailActivityType::Opened),
            "Email Opened"
        );
    }
}

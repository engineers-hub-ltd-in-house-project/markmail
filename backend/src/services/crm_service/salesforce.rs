use async_trait::async_trait;
use chrono::Utc;
use rustforce::response::QueryResponse;
use rustforce::Client as SalesforceClient;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::crm::{
    CrmBulkSyncResult, CrmCampaign, CrmContact, CrmCustomField, CrmEmailActivity, CrmFeature,
    CrmFieldMapping, CrmList, CrmSyncResult, EmailActivityType,
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

/// Salesforceプロバイダー
pub struct SalesforceProvider {
    config: SalesforceConfig,
}

impl SalesforceProvider {
    /// 新しいSalesforceプロバイダーを作成
    pub async fn new(config: SalesforceConfig) -> Result<Self, CrmError> {
        Ok(Self { config })
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
        let mut results = Vec::new();
        let total = contacts.len();
        let mut success = 0;
        let mut failed = 0;

        // TODO: 実際のBulk APIを使用する最適化が必要
        // 現在は1件ずつ同期
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
            CrmFeature::CustomFields => false, // Phase 2では未実装
            CrmFeature::BulkOperations => false, // 実際のBulk API使用は未実装
            CrmFeature::WebhookSupport => false,
            CrmFeature::RealTimeSync => false,
        }
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
        assert!(!provider.supports_feature(CrmFeature::WebhookSupport));
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

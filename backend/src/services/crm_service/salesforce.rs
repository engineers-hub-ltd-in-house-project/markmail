use async_trait::async_trait;

use crate::models::crm::{
    CrmBulkSyncResult, CrmCampaign, CrmContact, CrmCustomField, CrmEmailActivity, CrmFeature,
    CrmFieldMapping, CrmList, CrmSyncResult,
};

use super::{CrmError, CrmProvider, SalesforceConfig};

/// Salesforceプロバイダー
pub struct SalesforceProvider {
    #[allow(dead_code)]
    config: SalesforceConfig,
    // TODO: Salesforce CLIクライアントを追加
}

impl SalesforceProvider {
    /// 新しいSalesforceプロバイダーを作成
    pub async fn new(config: SalesforceConfig) -> Result<Self, CrmError> {
        // TODO: Salesforce接続の初期化
        // 現在は基本的な構造のみ

        Ok(Self { config })
    }

    /// Salesforce APIバージョンを取得
    #[allow(dead_code)]
    fn api_version(&self) -> &str {
        &self.config.api_version
    }

    /// インスタンスURLを取得
    #[allow(dead_code)]
    fn instance_url(&self) -> &str {
        &self.config.instance_url
    }
}

#[async_trait]
impl CrmProvider for SalesforceProvider {
    async fn sync_contact(&self, _contact: &CrmContact) -> Result<CrmSyncResult, CrmError> {
        // TODO: Salesforce Contact オブジェクトとの同期を実装
        Err(CrmError::Unknown("未実装".to_string()))
    }

    async fn get_contact(&self, _email: &str) -> Result<Option<CrmContact>, CrmError> {
        // TODO: SOQLクエリでContactを検索
        // SELECT Id, Email, FirstName, LastName, Company, Phone FROM Contact WHERE Email = :email
        Err(CrmError::Unknown("未実装".to_string()))
    }

    async fn update_contact(&self, _contact: &CrmContact) -> Result<CrmSyncResult, CrmError> {
        // TODO: Salesforce Contactの更新
        Err(CrmError::Unknown("未実装".to_string()))
    }

    async fn delete_contact(&self, _id: &str) -> Result<(), CrmError> {
        // TODO: Salesforce Contactの削除
        Err(CrmError::Unknown("未実装".to_string()))
    }

    async fn bulk_sync_contacts(
        &self,
        _contacts: Vec<CrmContact>,
    ) -> Result<CrmBulkSyncResult, CrmError> {
        // TODO: Bulk APIを使用した一括同期
        Err(CrmError::Unknown("未実装".to_string()))
    }

    async fn sync_campaign(&self, _campaign: &CrmCampaign) -> Result<CrmSyncResult, CrmError> {
        // TODO: Salesforce Campaignとの同期
        Err(CrmError::Unknown("未実装".to_string()))
    }

    async fn log_email_activity(&self, _activity: &CrmEmailActivity) -> Result<(), CrmError> {
        // TODO: EmailMessage または Task としてアクティビティを記録
        Err(CrmError::Unknown("未実装".to_string()))
    }

    async fn sync_list_membership(&self, _list: &CrmList) -> Result<(), CrmError> {
        // TODO: CampaignMember の同期
        Err(CrmError::Unknown("未実装".to_string()))
    }

    async fn get_custom_fields(&self) -> Result<Vec<CrmCustomField>, CrmError> {
        // TODO: Describe APIを使用してカスタムフィールドを取得
        Err(CrmError::Unknown("未実装".to_string()))
    }

    async fn map_custom_fields(&self, _mapping: &CrmFieldMapping) -> Result<(), CrmError> {
        // TODO: フィールドマッピングの保存
        Err(CrmError::Unknown("未実装".to_string()))
    }

    fn provider_name(&self) -> &str {
        "Salesforce"
    }

    fn supports_feature(&self, feature: CrmFeature) -> bool {
        match feature {
            CrmFeature::ContactSync => true,
            CrmFeature::CampaignSync => true,
            CrmFeature::CustomFields => true,
            CrmFeature::BulkOperations => true,
            CrmFeature::WebhookSupport => false, // Phase 1では未実装
            CrmFeature::RealTimeSync => false,   // Phase 1では未実装
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
}

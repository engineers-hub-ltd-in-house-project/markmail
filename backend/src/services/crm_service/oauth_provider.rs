use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    models::crm::{
        CrmBulkSyncResult, CrmCampaign, CrmContact, CrmCustomField, CrmEmailActivity, CrmFeature,
        CrmFieldMapping, CrmLead, CrmList, CrmSyncResult,
    },
    services::crm_service::{
        oauth_integration::OAuthIntegrationService, salesforce::SalesforceProvider, CrmError,
        CrmProvider, SalesforceConfig,
    },
};

/// OAuth2認証を使用するSalesforceプロバイダーのラッパー
pub struct OAuthSalesforceProvider {
    user_id: Uuid,
    #[allow(dead_code)]
    pool: PgPool,
    oauth_service: OAuthIntegrationService,
}

impl OAuthSalesforceProvider {
    /// 新しいOAuth2ベースのSalesforceプロバイダーを作成
    pub async fn new(pool: PgPool, user_id: Uuid) -> Result<Self, CrmError> {
        let oauth_service = OAuthIntegrationService::new(pool.clone()).map_err(|e| {
            CrmError::Configuration(format!("OAuth service initialization failed: {e}"))
        })?;

        // 認証状態を確認
        if !oauth_service
            .is_authenticated(user_id)
            .await
            .map_err(|e| CrmError::Authentication(format!("Authentication check failed: {e}")))?
        {
            return Err(CrmError::Authentication(
                "User is not authenticated with Salesforce".to_string(),
            ));
        }

        Ok(Self {
            user_id,
            pool,
            oauth_service,
        })
    }

    /// 内部のSalesforceプロバイダーを作成（アクセストークンを自動更新）
    async fn create_provider(&self) -> Result<SalesforceProvider, CrmError> {
        tracing::info!("Creating Salesforce provider for user: {}", self.user_id);

        // 有効なアクセストークンを取得（自動リフレッシュ）
        let access_token = self
            .oauth_service
            .get_access_token(self.user_id)
            .await
            .map_err(|e| {
                tracing::error!(
                    "Failed to get access token for user {}: {}",
                    self.user_id,
                    e
                );
                CrmError::Authentication(format!("Failed to get access token: {e}"))
            })?;

        tracing::debug!("Successfully retrieved access token");

        // インスタンスURLを取得
        let instance_url = self
            .oauth_service
            .get_instance_url(self.user_id)
            .await
            .map_err(|e| {
                tracing::error!(
                    "Failed to get instance URL for user {}: {}",
                    self.user_id,
                    e
                );
                CrmError::Configuration(format!("Failed to get instance URL: {e}"))
            })?;

        tracing::debug!("Instance URL: {}", instance_url);

        // 認証詳細を取得
        let auth_details = self
            .oauth_service
            .get_auth_details(self.user_id)
            .await
            .map_err(|e| {
                tracing::error!(
                    "Failed to get auth details for user {}: {}",
                    self.user_id,
                    e
                );
                CrmError::Configuration(format!("Failed to get auth details: {e}"))
            })?
            .ok_or_else(|| {
                tracing::error!("No authentication details found for user {}", self.user_id);
                CrmError::Authentication("No authentication details found".to_string())
            })?;

        tracing::info!(
            "Auth details retrieved - org_id: {}, username: {}",
            auth_details.org_id,
            auth_details.username
        );

        // Salesforce設定を構築
        let config = SalesforceConfig {
            org_alias: auth_details.org_id.clone(),
            api_version: "v60.0".to_string(), // TODO: 設定から取得
            instance_url,
            access_token,
            refresh_token: None, // OAuth2マネージャーが管理
        };

        SalesforceProvider::new(config).await
    }
}

#[async_trait]
impl CrmProvider for OAuthSalesforceProvider {
    async fn sync_contact(&self, contact: &CrmContact) -> Result<CrmSyncResult, CrmError> {
        let provider = self.create_provider().await?;
        provider.sync_contact(contact).await
    }

    async fn get_contact(&self, email: &str) -> Result<Option<CrmContact>, CrmError> {
        let provider = self.create_provider().await?;
        provider.get_contact(email).await
    }

    async fn update_contact(&self, contact: &CrmContact) -> Result<CrmSyncResult, CrmError> {
        let provider = self.create_provider().await?;
        provider.update_contact(contact).await
    }

    async fn delete_contact(&self, id: &str) -> Result<(), CrmError> {
        let provider = self.create_provider().await?;
        provider.delete_contact(id).await
    }

    async fn bulk_sync_contacts(
        &self,
        contacts: Vec<CrmContact>,
    ) -> Result<CrmBulkSyncResult, CrmError> {
        let provider = self.create_provider().await?;
        provider.bulk_sync_contacts(contacts).await
    }

    async fn create_lead(&self, lead: &CrmLead) -> Result<CrmSyncResult, CrmError> {
        let provider = self.create_provider().await?;
        provider.create_lead(lead).await
    }

    async fn get_lead(&self, email: &str) -> Result<Option<CrmLead>, CrmError> {
        let provider = self.create_provider().await?;
        provider.get_lead(email).await
    }

    async fn convert_lead_to_contact(&self, lead_id: &str) -> Result<CrmSyncResult, CrmError> {
        let provider = self.create_provider().await?;
        provider.convert_lead_to_contact(lead_id).await
    }

    async fn sync_campaign(&self, campaign: &CrmCampaign) -> Result<CrmSyncResult, CrmError> {
        let provider = self.create_provider().await?;
        provider.sync_campaign(campaign).await
    }

    async fn log_email_activity(&self, activity: &CrmEmailActivity) -> Result<(), CrmError> {
        let provider = self.create_provider().await?;
        provider.log_email_activity(activity).await
    }

    async fn sync_list_membership(&self, list: &CrmList) -> Result<(), CrmError> {
        let provider = self.create_provider().await?;
        provider.sync_list_membership(list).await
    }

    async fn get_custom_fields(&self) -> Result<Vec<CrmCustomField>, CrmError> {
        let provider = self.create_provider().await?;
        provider.get_custom_fields().await
    }

    async fn map_custom_fields(&self, mapping: &CrmFieldMapping) -> Result<(), CrmError> {
        let provider = self.create_provider().await?;
        provider.map_custom_fields(mapping).await
    }

    fn provider_name(&self) -> &str {
        "Salesforce (OAuth2)"
    }

    fn supports_feature(&self, feature: CrmFeature) -> bool {
        // すべての機能をサポート
        match feature {
            CrmFeature::ContactSync => true,
            CrmFeature::CampaignSync => true,
            CrmFeature::CustomFields => true,
            CrmFeature::BulkOperations => true,
            CrmFeature::WebhookSupport => false, // OAuth2では未実装
            CrmFeature::RealTimeSync => false,   // OAuth2では未実装
        }
    }
}

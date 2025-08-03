use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

use crate::{
    database::crm_integrations::{
        self, CreateCrmIntegrationParams, CreateSyncLogParams, CrmIntegration,
    },
    models::crm::{
        CrmBulkSyncResult, CrmCampaign, CrmContact, CrmCustomField, CrmEmailActivity, CrmFeature,
        CrmFieldMapping, CrmIntegrationSettings, CrmLead, CrmList, CrmProviderType, CrmSyncResult,
    },
};

pub mod oauth_integration;
pub mod oauth_provider;
pub mod salesforce;
pub mod salesforce_auth;

/// CRMプロバイダーのエラー型
#[derive(Error, Debug)]
pub enum CrmError {
    #[error("CRM接続エラー: {0}")]
    Connection(String),

    #[error("CRM認証エラー: {0}")]
    Authentication(String),

    #[error("CRM API エラー: {0}")]
    ApiError(String),

    #[error("データ変換エラー: {0}")]
    DataConversion(String),

    #[error("設定エラー: {0}")]
    Configuration(String),

    #[error("レート制限エラー: {0}")]
    RateLimit(String),

    #[error("データベースエラー: {0}")]
    Database(#[from] sqlx::Error),

    #[error("シリアライズエラー: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("不明なエラー: {0}")]
    Unknown(String),
}

/// CRMプロバイダーのトレイト
#[async_trait]
pub trait CrmProvider: Send + Sync {
    // 連絡先管理
    async fn sync_contact(&self, contact: &CrmContact) -> Result<CrmSyncResult, CrmError>;
    async fn get_contact(&self, email: &str) -> Result<Option<CrmContact>, CrmError>;
    async fn update_contact(&self, contact: &CrmContact) -> Result<CrmSyncResult, CrmError>;
    async fn delete_contact(&self, id: &str) -> Result<(), CrmError>;
    async fn bulk_sync_contacts(
        &self,
        contacts: Vec<CrmContact>,
    ) -> Result<CrmBulkSyncResult, CrmError>;

    // リード管理
    async fn create_lead(&self, lead: &CrmLead) -> Result<CrmSyncResult, CrmError>;
    async fn get_lead(&self, email: &str) -> Result<Option<CrmLead>, CrmError>;
    async fn convert_lead_to_contact(&self, lead_id: &str) -> Result<CrmSyncResult, CrmError>;

    // キャンペーン・アクティビティ管理
    async fn sync_campaign(&self, campaign: &CrmCampaign) -> Result<CrmSyncResult, CrmError>;
    async fn log_email_activity(&self, activity: &CrmEmailActivity) -> Result<(), CrmError>;
    async fn sync_list_membership(&self, list: &CrmList) -> Result<(), CrmError>;

    // カスタムフィールド管理
    async fn get_custom_fields(&self) -> Result<Vec<CrmCustomField>, CrmError>;
    async fn map_custom_fields(&self, mapping: &CrmFieldMapping) -> Result<(), CrmError>;

    // メタデータ
    fn provider_name(&self) -> &str;
    fn supports_feature(&self, feature: CrmFeature) -> bool;
}

/// CRM統合の設定
#[derive(Debug, Clone)]
pub struct CrmConfig {
    pub provider: CrmProviderType,
    pub salesforce_config: Option<SalesforceConfig>,
    pub sync_enabled: bool,
    pub batch_size: usize,
}

/// Salesforce設定
#[derive(Debug, Clone)]
pub struct SalesforceConfig {
    pub org_alias: String,
    pub api_version: String,
    pub instance_url: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
}

/// 統合設定保存パラメータ
pub struct SaveIntegrationParams<'a> {
    pub user_id: Uuid,
    pub provider: CrmProviderType,
    pub org_id: &'a str,
    pub instance_url: &'a str,
    pub access_token: &'a str,
    pub refresh_token: Option<&'a str>,
    pub settings: &'a CrmIntegrationSettings,
}

/// CRMサービス
pub struct CrmService {
    provider: Arc<Box<dyn CrmProvider>>,
    #[allow(dead_code)]
    config: CrmConfig,
    #[allow(dead_code)]
    pool: PgPool,
}

impl CrmService {
    /// 新しいCRMサービスを作成
    pub async fn new(pool: PgPool, user_id: Uuid) -> Result<Self, CrmError> {
        // TODO: データベースから統合設定を読み込む
        // 現在は仮の実装
        let config = Self::load_config_from_db(&pool, user_id).await?;
        let provider = Self::create_provider(&config, &pool, user_id).await?;

        Ok(Self {
            provider: Arc::new(provider),
            config,
            pool,
        })
    }

    /// データベースから設定を読み込む
    async fn load_config_from_db(pool: &PgPool, user_id: Uuid) -> Result<CrmConfig, CrmError> {
        // Salesforce統合を取得（現在はSalesforceのみサポート）
        let integration =
            crm_integrations::get_user_crm_integration(pool, user_id, CrmProviderType::Salesforce)
                .await?
                .ok_or_else(|| {
                    CrmError::Configuration("CRM統合が設定されていません".to_string())
                })?;

        if !integration.is_active() {
            return Err(CrmError::Configuration(
                "CRM統合が無効化されています".to_string(),
            ));
        }

        // Salesforce設定を構築
        let api_version = integration
            .salesforce_settings
            .as_ref()
            .and_then(|s| s.get("api_version"))
            .and_then(|v| v.as_str())
            .unwrap_or("v60.0")
            .to_string();

        let instance_url = integration.instance_url.clone().ok_or_else(|| {
            CrmError::Configuration("インスタンスURLが設定されていません".to_string())
        })?;

        let access_token = integration.get_access_token().ok_or_else(|| {
            CrmError::Configuration("アクセストークンが設定されていません".to_string())
        })?;

        let refresh_token = integration.get_refresh_token();

        let salesforce_config = SalesforceConfig {
            org_alias: integration.org_id.unwrap_or_default(),
            api_version,
            instance_url,
            access_token,
            refresh_token,
        };

        Ok(CrmConfig {
            provider: CrmProviderType::Salesforce,
            salesforce_config: Some(salesforce_config),
            sync_enabled: true,
            batch_size: 100,
        })
    }

    /// プロバイダーを作成
    async fn create_provider(
        config: &CrmConfig,
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Box<dyn CrmProvider>, CrmError> {
        match &config.provider {
            CrmProviderType::Salesforce => {
                // まずOAuth2認証を試みる
                let oauth_service = oauth_integration::OAuthIntegrationService::new(pool.clone())
                    .map_err(|e| {
                    CrmError::Configuration(format!("OAuth service error: {e}"))
                })?;

                if oauth_service
                    .is_authenticated(user_id)
                    .await
                    .map_err(CrmError::Authentication)?
                {
                    // OAuth2認証済みの場合
                    let provider =
                        oauth_provider::OAuthSalesforceProvider::new(pool.clone(), user_id).await?;
                    Ok(Box::new(provider))
                } else if let Some(sf_config) = &config.salesforce_config {
                    // 従来のCLI/設定ベースの認証
                    let provider = salesforce::SalesforceProvider::new(sf_config.clone()).await?;
                    Ok(Box::new(provider))
                } else {
                    Err(CrmError::Configuration(
                        "Salesforce認証が設定されていません。OAuth2認証を行うか、統合設定を追加してください".to_string(),
                    ))
                }
            }
        }
    }

    /// プロバイダーへのアクセス
    pub fn provider(&self) -> &dyn CrmProvider {
        &**self.provider
    }

    /// 統合設定を保存
    pub async fn save_integration(
        pool: &PgPool,
        params: SaveIntegrationParams<'_>,
    ) -> Result<Uuid, CrmError> {
        let integration_params = CreateCrmIntegrationParams {
            user_id: params.user_id,
            provider: params.provider,
            org_id: params.org_id,
            instance_url: params.instance_url,
            access_token: params.access_token,
            refresh_token: params.refresh_token,
            settings: params.settings,
        };

        let integration_id =
            crm_integrations::create_crm_integration(pool, integration_params).await?;

        Ok(integration_id)
    }

    /// 同期ログを記録
    pub async fn log_sync_activity(
        pool: &PgPool,
        integration_id: Uuid,
        entity_type: &str,
        results: &[CrmSyncResult],
    ) -> Result<(), CrmError> {
        let entity_count = results.len() as i32;
        let success_count = results.iter().filter(|r| r.success).count() as i32;
        let error_count = entity_count - success_count;

        let error_details = if error_count > 0 {
            let errors: Vec<_> = results
                .iter()
                .filter(|r| !r.success)
                .filter_map(|r| r.error_message.as_ref())
                .map(|e| e.as_str())
                .collect();
            Some(serde_json::json!({"errors": errors}))
        } else {
            None
        };

        let params = CreateSyncLogParams {
            integration_id,
            sync_type: "manual", // TODO: 同期タイプをパラメータ化
            entity_type,
            entity_count,
            success_count,
            error_count,
            error_details,
        };

        crm_integrations::create_sync_log(pool, params).await?;

        // 個別の同期ステータスを記録
        for result in results {
            if result.success {
                crm_integrations::upsert_sync_status(
                    pool,
                    integration_id,
                    entity_type,
                    result.markmail_id,
                    &result.crm_id,
                    "synced",
                )
                .await?;
            }
        }

        Ok(())
    }

    /// 統合設定を取得
    pub async fn get_integration(
        pool: &PgPool,
        user_id: Uuid,
        provider: CrmProviderType,
    ) -> Result<Option<CrmIntegration>, CrmError> {
        Ok(crm_integrations::get_user_crm_integration(pool, user_id, provider).await?)
    }

    /// 統合を無効化
    pub async fn deactivate_integration(
        pool: &PgPool,
        integration_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), CrmError> {
        crm_integrations::deactivate_crm_integration(pool, integration_id, user_id).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let error = CrmError::Connection("接続に失敗しました".to_string());
        assert_eq!(error.to_string(), "CRM接続エラー: 接続に失敗しました");
    }
}

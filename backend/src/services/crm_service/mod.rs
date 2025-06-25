use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

use crate::models::crm::{
    CrmBulkSyncResult, CrmCampaign, CrmContact, CrmCustomField, CrmEmailActivity, CrmFeature,
    CrmFieldMapping, CrmList, CrmProviderType, CrmSyncResult,
};

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
        let provider = Self::create_provider(&config).await?;

        Ok(Self {
            provider: Arc::new(provider),
            config,
            pool,
        })
    }

    /// データベースから設定を読み込む
    async fn load_config_from_db(_pool: &PgPool, _user_id: Uuid) -> Result<CrmConfig, CrmError> {
        // TODO: 実際のデータベースクエリを実装
        // 現在は仮の実装
        Err(CrmError::Configuration(
            "CRM統合が設定されていません".to_string(),
        ))
    }

    /// プロバイダーを作成
    async fn create_provider(config: &CrmConfig) -> Result<Box<dyn CrmProvider>, CrmError> {
        match &config.provider {
            CrmProviderType::Salesforce => {
                if let Some(sf_config) = &config.salesforce_config {
                    let provider = salesforce::SalesforceProvider::new(sf_config.clone()).await?;
                    Ok(Box::new(provider))
                } else {
                    Err(CrmError::Configuration(
                        "Salesforce設定が見つかりません".to_string(),
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
        &self,
        _user_id: Uuid,
        _config: CrmConfig,
        _credentials: serde_json::Value,
    ) -> Result<Uuid, CrmError> {
        // TODO: データベースに統合設定を保存
        Err(CrmError::Unknown("未実装".to_string()))
    }

    /// 同期ログを記録
    pub async fn log_sync_activity(
        &self,
        _integration_id: Uuid,
        _sync_type: &str,
        _entity_type: &str,
        _result: &CrmBulkSyncResult,
    ) -> Result<(), CrmError> {
        // TODO: 同期ログをデータベースに記録
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

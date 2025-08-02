use crate::{
    crm::oauth::{SalesforceOAuthClient, TokenManager},
    models::crm_oauth::SalesforceOAuthSettings,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// OAuth2ベースのSalesforce統合サービス
pub struct OAuthIntegrationService {
    pool: PgPool,
}

impl OAuthIntegrationService {
    /// 新しいサービスインスタンスを作成
    pub fn new(pool: PgPool) -> Result<Self, String> {
        Ok(Self { pool })
    }

    /// TokenManagerを作成
    fn create_token_manager(&self) -> Result<TokenManager, String> {
        let settings = SalesforceOAuthSettings::from_env()?;
        let oauth_client = SalesforceOAuthClient::new(settings)?;
        Ok(TokenManager::new(self.pool.clone(), oauth_client))
    }

    /// ユーザーが認証済みかチェック
    pub async fn is_authenticated(&self, user_id: Uuid) -> Result<bool, String> {
        let token_manager = self.create_token_manager()?;
        match token_manager.get_auth_status(user_id).await {
            Ok(Some(_)) => Ok(true),
            Ok(None) => Ok(false),
            Err(e) => Err(format!("認証状態の確認に失敗: {e}")),
        }
    }

    /// 有効なアクセストークンを取得（自動リフレッシュ付き）
    pub async fn get_access_token(&self, user_id: Uuid) -> Result<String, String> {
        let token_manager = self.create_token_manager()?;
        token_manager.get_valid_access_token(user_id).await
    }

    /// インスタンスURLを取得
    pub async fn get_instance_url(&self, user_id: Uuid) -> Result<String, String> {
        let token_manager = self.create_token_manager()?;
        let token_info = token_manager
            .get_auth_status(user_id)
            .await
            .map_err(|e| format!("認証情報の取得に失敗: {e}"))?
            .ok_or_else(|| "認証情報が見つかりません".to_string())?;

        Ok(token_info.instance_url)
    }

    /// 認証状態の詳細を取得
    pub async fn get_auth_details(&self, user_id: Uuid) -> Result<Option<AuthDetails>, String> {
        let token_manager = self.create_token_manager()?;
        match token_manager.get_auth_status(user_id).await {
            Ok(Some(token)) => {
                // ユーザー情報を取得
                let access_token = self.get_access_token(user_id).await?;
                let settings = SalesforceOAuthSettings::from_env()?;
                let oauth_client = SalesforceOAuthClient::new(settings)?;
                let user_info = oauth_client.get_user_info(&access_token).await?;

                Ok(Some(AuthDetails {
                    org_id: user_info.organization_id,
                    username: user_info.preferred_username,
                    instance_url: token.instance_url,
                    connected: true,
                }))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(format!("認証状態の確認に失敗: {e}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthDetails {
    pub org_id: String,
    pub username: String,
    pub instance_url: String,
    pub connected: bool,
}

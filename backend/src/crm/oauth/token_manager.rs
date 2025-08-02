use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use super::salesforce_oauth::SalesforceOAuthClient;
use crate::models::crm_oauth::{SalesforceOAuthToken, SalesforceTokenResponse, TokenRefreshStatus};

pub struct TokenManager {
    pool: PgPool,
    oauth_client: SalesforceOAuthClient,
}

impl TokenManager {
    /// 新しいトークンマネージャーを作成
    pub fn new(pool: PgPool, oauth_client: SalesforceOAuthClient) -> Self {
        Self { pool, oauth_client }
    }

    /// トークンを保存
    pub async fn save_tokens(
        &self,
        user_id: Uuid,
        tokens: &SalesforceTokenResponse,
        instance_url: &str,
    ) -> Result<(), sqlx::Error> {
        let expires_at = Utc::now() + Duration::seconds(tokens.expires_in.unwrap_or(7200) as i64);

        sqlx::query!(
            r#"
            INSERT INTO salesforce_oauth_tokens 
            (user_id, access_token, refresh_token, instance_url, token_type, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (user_id) 
            DO UPDATE SET 
                access_token = EXCLUDED.access_token,
                refresh_token = EXCLUDED.refresh_token,
                instance_url = EXCLUDED.instance_url,
                token_type = EXCLUDED.token_type,
                expires_at = EXCLUDED.expires_at,
                updated_at = CURRENT_TIMESTAMP
            "#,
            user_id,
            tokens.access_token,
            tokens.refresh_token.as_ref().unwrap(),
            instance_url,
            tokens.token_type,
            expires_at,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 有効なアクセストークンを取得（必要に応じて自動更新）
    pub async fn get_valid_access_token(&self, user_id: Uuid) -> Result<String, String> {
        let token_info = sqlx::query!(
            r#"
            SELECT access_token, refresh_token, expires_at, instance_url
            FROM salesforce_oauth_tokens
            WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error: {e}"))?
        .ok_or_else(|| "No token found for user".to_string())?;

        // トークンの有効期限を確認
        let expires_at: DateTime<Utc> = token_info.expires_at;
        let now = Utc::now();

        if expires_at > now + Duration::minutes(5) {
            // まだ有効（5分の余裕を持って判定）
            Ok(token_info.access_token)
        } else {
            // 期限切れまたは期限切れ間近 - リフレッシュ
            self.refresh_and_save_token(user_id, &token_info.refresh_token)
                .await
        }
    }

    /// トークンをリフレッシュして保存
    async fn refresh_and_save_token(
        &self,
        user_id: Uuid,
        refresh_token: &str,
    ) -> Result<String, String> {
        match self.oauth_client.refresh_token(refresh_token).await {
            Ok(new_tokens) => {
                // 既存のinstance_urlを取得
                let instance_url = self.get_instance_url(user_id).await?;

                // 新しいトークンを保存
                self.save_tokens(user_id, &new_tokens, &instance_url)
                    .await
                    .map_err(|e| format!("Failed to save refreshed token: {e}"))?;

                // ログを記録
                self.log_refresh_attempt(user_id, true, None).await;

                Ok(new_tokens.access_token)
            }
            Err(e) => {
                self.log_refresh_attempt(user_id, false, Some(&e)).await;
                Err(format!("Token refresh failed: {e}"))
            }
        }
    }

    /// ユーザーのOAuth認証状態を取得
    pub async fn get_auth_status(
        &self,
        user_id: Uuid,
    ) -> Result<Option<SalesforceOAuthToken>, sqlx::Error> {
        let token = sqlx::query_as!(
            SalesforceOAuthToken,
            r#"
            SELECT id, user_id, access_token, refresh_token, instance_url, 
                   token_type, expires_at, 
                   created_at as "created_at!", 
                   updated_at as "updated_at!"
            FROM salesforce_oauth_tokens
            WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(token)
    }

    /// トークンを削除（ログアウト）
    pub async fn delete_tokens(&self, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "DELETE FROM salesforce_oauth_tokens WHERE user_id = $1",
            user_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// インスタンスURLを取得
    async fn get_instance_url(&self, user_id: Uuid) -> Result<String, String> {
        sqlx::query_scalar!(
            "SELECT instance_url FROM salesforce_oauth_tokens WHERE user_id = $1",
            user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to get instance URL: {e}"))
    }

    /// トークンリフレッシュの試行をログに記録
    async fn log_refresh_attempt(&self, user_id: Uuid, success: bool, error: Option<&str>) {
        let status = if success {
            TokenRefreshStatus::Success
        } else {
            TokenRefreshStatus::Failure
        };

        let _ = sqlx::query!(
            r#"
            INSERT INTO salesforce_token_refresh_logs 
            (user_id, status, error_message)
            VALUES ($1, $2, $3)
            "#,
            user_id,
            status.as_str(),
            error
        )
        .execute(&self.pool)
        .await;
    }

    /// Authorization Codeを交換してトークンを保存
    pub async fn exchange_code_and_save(&self, user_id: Uuid, code: String) -> Result<(), String> {
        // Authorization Codeをトークンに交換
        let mut tokens = self.oauth_client.exchange_code(code).await?;

        // ユーザー情報を取得してinstance_urlを取得
        let user_info = self
            .oauth_client
            .get_user_info(&tokens.access_token)
            .await?;
        let instance_url = extract_instance_url(&user_info.urls.rest)?;

        // トークンにinstance_urlを設定
        tokens.instance_url = Some(instance_url.clone());

        // トークンを保存
        self.save_tokens(user_id, &tokens, &instance_url)
            .await
            .map_err(|e| format!("Failed to save tokens: {e}"))?;

        Ok(())
    }
}

/// RESTエンドポイントからインスタンスURLを抽出
fn extract_instance_url(rest_url: &str) -> Result<String, String> {
    // 例: "https://na1.salesforce.com/services/data/v59.0/"
    // から "https://na1.salesforce.com" を抽出
    rest_url
        .split("/services/data/")
        .next()
        .ok_or_else(|| "Invalid REST URL format".to_string())
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_instance_url() {
        let rest_url = "https://na1.salesforce.com/services/data/v59.0/";
        let result = extract_instance_url(rest_url);
        assert_eq!(result.unwrap(), "https://na1.salesforce.com");

        let rest_url = "https://my-domain.my.salesforce.com/services/data/v59.0/";
        let result = extract_instance_url(rest_url);
        assert_eq!(result.unwrap(), "https://my-domain.my.salesforce.com");
    }
}

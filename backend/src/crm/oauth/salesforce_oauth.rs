use oauth2::{
    basic::{
        BasicClient, BasicErrorResponse, BasicRevocationErrorResponse,
        BasicTokenIntrospectionResponse, BasicTokenResponse,
    },
    AuthUrl, Client, ClientId, ClientSecret, CsrfToken, EndpointNotSet, EndpointSet, RedirectUrl,
    Scope, StandardRevocableToken, TokenUrl,
};
use serde::{Deserialize, Serialize};

use crate::models::crm_oauth::{SalesforceOAuthSettings, SalesforceTokenResponse};

// Type alias for a fully configured OAuth2 client
type ConfiguredClient = Client<
    BasicErrorResponse,
    BasicTokenResponse,
    BasicTokenIntrospectionResponse,
    StandardRevocableToken,
    BasicRevocationErrorResponse,
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointSet,
>;

pub struct SalesforceOAuthClient {
    client: ConfiguredClient,
    #[allow(dead_code)]
    settings: SalesforceOAuthSettings,
    http_client: reqwest::Client,
}

impl SalesforceOAuthClient {
    /// 新しいOAuth2クライアントを作成
    pub fn new(settings: SalesforceOAuthSettings) -> Result<Self, String> {
        let client = BasicClient::new(ClientId::new(settings.client_id.clone()))
            .set_client_secret(ClientSecret::new(settings.client_secret.clone()))
            .set_auth_uri(AuthUrl::new(settings.auth_url.clone()).map_err(|e| e.to_string())?)
            .set_token_uri(TokenUrl::new(settings.token_url.clone()).map_err(|e| e.to_string())?)
            .set_redirect_uri(
                RedirectUrl::new(settings.redirect_url.clone()).map_err(|e| e.to_string())?,
            );

        // SSRF対策のためリダイレクトを無効化
        let http_client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

        Ok(Self {
            client,
            settings,
            http_client,
        })
    }

    /// 認証URLを生成
    pub fn get_auth_url(&self) -> (String, CsrfToken) {
        let (auth_url, csrf_token) = self
            .client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("api".to_string()))
            .add_scope(Scope::new("refresh_token".to_string()))
            .add_scope(Scope::new("offline_access".to_string()))
            .url();

        (auth_url.to_string(), csrf_token)
    }

    /// Authorization Codeをアクセストークンに交換
    pub async fn exchange_code(&self, code: String) -> Result<SalesforceTokenResponse, String> {
        // Salesforceのトークンエンドポイントに直接リクエストを送信
        let params = [
            ("grant_type", "authorization_code"),
            ("code", &code),
            ("client_id", &self.settings.client_id),
            ("client_secret", &self.settings.client_secret),
            ("redirect_uri", &self.settings.redirect_url),
        ];

        let response = self
            .http_client
            .post(&self.settings.token_url)
            .form(&params)
            .send()
            .await
            .map_err(|e| format!("Token exchange request failed: {e}"))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("Token exchange failed: {error_text}"));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {e}"))?;
        tracing::debug!("Token exchange response: {}", response_text);

        let token_response: SalesforceTokenResponse = serde_json::from_str(&response_text)
            .map_err(|e| {
                format!("Failed to parse token response: {e}. Response was: {response_text}")
            })?;

        tracing::info!(
            "Token exchange successful. Instance URL: {:?}",
            token_response.instance_url
        );
        Ok(token_response)
    }

    /// リフレッシュトークンを使用してアクセストークンを更新
    pub async fn refresh_token(
        &self,
        refresh_token: &str,
    ) -> Result<SalesforceTokenResponse, String> {
        let params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", &self.settings.client_id),
            ("client_secret", &self.settings.client_secret),
        ];

        let response = self
            .http_client
            .post(&self.settings.token_url)
            .form(&params)
            .send()
            .await
            .map_err(|e| format!("Token refresh request failed: {e}"))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("Token refresh failed: {error_text}"));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {e}"))?;
        tracing::debug!("Token exchange response: {}", response_text);

        let token_response: SalesforceTokenResponse = serde_json::from_str(&response_text)
            .map_err(|e| {
                format!("Failed to parse token response: {e}. Response was: {response_text}")
            })?;

        tracing::info!(
            "Token exchange successful. Instance URL: {:?}",
            token_response.instance_url
        );
        Ok(token_response)
    }

    /// アクセストークンを使用してSalesforceのユーザー情報を取得
    /// インスタンスURLを渡すことで、組織固有のエンドポイントを使用する
    pub async fn get_user_info(&self, access_token: &str) -> Result<SalesforceUserInfo, String> {
        self.get_user_info_with_instance_url(access_token, None)
            .await
    }

    /// インスタンスURLを指定してユーザー情報を取得
    pub async fn get_user_info_with_instance_url(
        &self,
        access_token: &str,
        instance_url: Option<&str>,
    ) -> Result<SalesforceUserInfo, String> {
        // インスタンスURLが指定されている場合はそれを使用、そうでない場合はデフォルトのURLを使用
        let userinfo_url = if let Some(instance_url) = instance_url {
            format!(
                "{}/services/oauth2/userinfo",
                instance_url.trim_end_matches('/')
            )
        } else {
            // フォールバック: サンドボックスか本番環境かに応じてURLを決定
            if self.settings.is_sandbox {
                "https://test.salesforce.com/services/oauth2/userinfo".to_string()
            } else {
                "https://login.salesforce.com/services/oauth2/userinfo".to_string()
            }
        };

        tracing::debug!("Getting user info from URL: {}", userinfo_url);

        let client = reqwest::Client::new();
        let response = client
            .get(&userinfo_url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| format!("Failed to get user info: {e}"))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            tracing::error!(
                "User info request failed: {} - Response: {}",
                status,
                error_text
            );
            return Err(format!("User info request failed: {error_text}"));
        }

        let user_info: SalesforceUserInfo = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse user info: {e}"))?;

        Ok(user_info)
    }
}

/// Salesforceユーザー情報
#[derive(Debug, Serialize, Deserialize)]
pub struct SalesforceUserInfo {
    pub sub: String,
    pub user_id: String,
    pub organization_id: String,
    pub preferred_username: String,
    pub nickname: String,
    pub name: String,
    pub email: String,
    pub email_verified: bool,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub zoneinfo: String,
    pub photos: SalesforcePhotos,
    pub profile: String,
    pub picture: String,
    pub address: Option<SalesforceAddress>,
    pub urls: SalesforceUrls,
    pub active: bool,
    pub user_type: String,
    pub language: String,
    pub locale: String,
    #[serde(rename = "utcOffset")]
    pub utc_offset: i32,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SalesforcePhotos {
    pub picture: String,
    pub thumbnail: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SalesforceAddress {
    pub country: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SalesforceUrls {
    pub enterprise: String,
    pub metadata: String,
    pub partner: String,
    pub rest: String,
    pub sobjects: String,
    pub search: String,
    pub query: String,
    pub recent: String,
    pub tooling_soap: String,
    pub tooling_rest: String,
    pub profile: String,
    pub feeds: String,
    pub groups: String,
    pub users: String,
    pub feed_items: String,
    pub feed_elements: String,
    pub custom_domain: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let settings = SalesforceOAuthSettings {
            client_id: "test_client_id".to_string(),
            client_secret: "test_client_secret".to_string(),
            auth_url: "https://login.salesforce.com/services/oauth2/authorize".to_string(),
            token_url: "https://login.salesforce.com/services/oauth2/token".to_string(),
            redirect_url: "http://localhost:3000/api/crm/oauth/salesforce/callback".to_string(),
            is_sandbox: false,
        };

        let client = SalesforceOAuthClient::new(settings);
        assert!(client.is_ok());
    }

    #[test]
    fn test_auth_url_generation() {
        let settings = SalesforceOAuthSettings {
            client_id: "test_client_id".to_string(),
            client_secret: "test_client_secret".to_string(),
            auth_url: "https://login.salesforce.com/services/oauth2/authorize".to_string(),
            token_url: "https://login.salesforce.com/services/oauth2/token".to_string(),
            redirect_url: "http://localhost:3000/api/crm/oauth/salesforce/callback".to_string(),
            is_sandbox: false,
        };

        let client = SalesforceOAuthClient::new(settings).unwrap();
        let (auth_url, csrf_token) = client.get_auth_url();

        assert!(auth_url.contains("https://login.salesforce.com/services/oauth2/authorize"));
        assert!(auth_url.contains("client_id=test_client_id"));
        assert!(auth_url.contains("response_type=code"));
        assert!(auth_url.contains("scope=api+refresh_token+offline_access"));
        assert!(!csrf_token.secret().is_empty());
    }
}

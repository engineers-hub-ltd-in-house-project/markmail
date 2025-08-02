use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Salesforce OAuth トークン情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalesforceOAuthToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub access_token: String,
    pub refresh_token: String,
    pub instance_url: String,
    pub token_type: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// トークンリフレッシュログ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalesforceTokenRefreshLog {
    pub id: Uuid,
    pub user_id: Uuid,
    pub status: TokenRefreshStatus,
    pub error_message: Option<String>,
    pub refreshed_at: DateTime<Utc>,
}

/// トークンリフレッシュステータス
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TokenRefreshStatus {
    Success,
    Failure,
}

impl TokenRefreshStatus {
    pub fn as_str(&self) -> &str {
        match self {
            TokenRefreshStatus::Success => "success",
            TokenRefreshStatus::Failure => "failure",
        }
    }
}

/// OAuth認証開始リクエスト
#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthInitRequest {
    // 将来的な拡張用（例：スコープのカスタマイズ）
}

/// OAuth認証開始レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthInitResponse {
    pub auth_url: String,
    pub state: String,
}

/// OAuth認証コールバッククエリ
#[derive(Debug, Deserialize)]
pub struct OAuthCallbackQuery {
    pub code: String,
    pub state: String,
}

/// OAuth認証状態レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthStatusResponse {
    pub is_authenticated: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub instance_url: Option<String>,
}

/// Salesforceトークンレスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct SalesforceTokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub expires_in: Option<u64>,
    pub instance_url: Option<String>,
    pub id: Option<String>,
    pub issued_at: Option<String>,
}

/// Salesforce OAuth設定
#[derive(Debug, Clone)]
pub struct SalesforceOAuthSettings {
    pub client_id: String,
    pub client_secret: String,
    pub auth_url: String,
    pub token_url: String,
    pub redirect_url: String,
}

impl SalesforceOAuthSettings {
    /// 環境変数から設定を読み込み
    pub fn from_env() -> Result<Self, String> {
        Ok(Self {
            client_id: std::env::var("SALESFORCE_CLIENT_ID")
                .map_err(|_| "SALESFORCE_CLIENT_ID not set")?,
            client_secret: std::env::var("SALESFORCE_CLIENT_SECRET")
                .map_err(|_| "SALESFORCE_CLIENT_SECRET not set")?,
            auth_url: std::env::var("SALESFORCE_AUTH_URL").unwrap_or_else(|_| {
                "https://login.salesforce.com/services/oauth2/authorize".to_string()
            }),
            token_url: std::env::var("SALESFORCE_TOKEN_URL").unwrap_or_else(|_| {
                "https://login.salesforce.com/services/oauth2/token".to_string()
            }),
            redirect_url: std::env::var("SALESFORCE_REDIRECT_URI")
                .map_err(|_| "SALESFORCE_REDIRECT_URI not set")?,
        })
    }
}

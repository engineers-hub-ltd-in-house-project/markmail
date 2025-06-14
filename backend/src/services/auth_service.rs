// 認証サービス
// TODO: JWT生成、パスワードハッシュ化などの実装

use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

use crate::{
    database::{password_reset, refresh_tokens, users},
    models::user::{
        AuthResponse, ForgotPasswordRequest, LoginRequest, MessageResponse, RegisterRequest,
        ResetPasswordRequest, UserResponse,
    },
    utils::{
        jwt::{generate_refresh_token, generate_token, verify_token, Claims, TokenType},
        password::{hash_password, verify_password},
    },
};

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("データベースエラー: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("パスワードエラー: {0}")]
    PasswordError(#[from] crate::utils::password::PasswordError),

    #[error("JWTエラー: {0}")]
    JwtError(#[from] crate::utils::jwt::JwtError),

    #[error("メールアドレスは既に使用されています")]
    EmailAlreadyExists,

    #[error("メールアドレスまたはパスワードが正しくありません")]
    InvalidCredentials,

    #[error("ユーザーが見つかりません")]
    UserNotFound,

    #[error("無効なリフレッシュトークン")]
    InvalidRefreshToken,

    #[error("無効またはパスワードリセットトークンの有効期限が切れています")]
    InvalidOrExpiredResetToken,

    #[error(
        "最近パスワードリセットをリクエストしました。しばらくしてからもう一度お試しください。"
    )]
    TooManyResetRequests,

    #[error("メール送信エラー: {0}")]
    EmailError(String),
}

pub struct AuthService {
    pool: PgPool,
}

impl Default for AuthService {
    fn default() -> Self {
        let pool = PgPool::connect_lazy("postgres://invalid_url_for_default")
            .expect("This is only for default impl and should never be used");
        Self { pool }
    }
}

impl AuthService {
    pub fn new(pool: PgPool) -> Self {
        tracing::debug!("AuthService::new - pool: {:?}", pool);
        Self { pool }
    }

    /// ユーザー登録
    pub async fn register(&self, request: RegisterRequest) -> Result<AuthResponse, AuthError> {
        tracing::debug!(
            "register - email: {}, name: {}",
            request.email,
            request.name
        );

        // メールアドレスの重複チェック
        if users::email_exists(&self.pool, &request.email).await? {
            tracing::debug!("メールアドレスが既に使用されています: {}", request.email);
            return Err(AuthError::EmailAlreadyExists);
        }

        // パスワードをハッシュ化
        tracing::debug!("パスワードのハッシュ化を実行します");
        let password_hash = match hash_password(&request.password) {
            Ok(hash) => hash,
            Err(e) => {
                tracing::error!("パスワードハッシュ化エラー: {:?}", e);
                return Err(AuthError::PasswordError(e));
            }
        };
        tracing::debug!("パスワードのハッシュ化が完了しました");

        // ユーザーを作成
        tracing::debug!("ユーザーを作成します: {}", request.email);
        let user =
            match users::create_user(&self.pool, &request.email, &password_hash, &request.name)
                .await
            {
                Ok(user) => user,
                Err(e) => {
                    tracing::error!("ユーザー作成エラー: {:?}", e);
                    return Err(AuthError::DatabaseError(e));
                }
            };

        // JWTトークンを生成
        let claims = Claims::new(
            user.id,
            user.email.clone(),
            user.name.clone(),
            24,
            TokenType::Access,
        );
        let token = generate_token(&claims)?;

        // リフレッシュトークンを生成して保存
        let refresh_token = generate_refresh_token();
        refresh_tokens::save_refresh_token(&self.pool, user.id, &refresh_token, 30).await?;

        Ok(AuthResponse {
            token,
            refresh_token,
            user: user.into(),
        })
    }

    /// ログイン
    pub async fn login(&self, request: LoginRequest) -> Result<AuthResponse, AuthError> {
        // ユーザーを検索
        let user = users::find_user_by_email(&self.pool, &request.email)
            .await?
            .ok_or(AuthError::InvalidCredentials)?;

        // パスワードを検証
        if !verify_password(&request.password, &user.password_hash)? {
            return Err(AuthError::InvalidCredentials);
        }

        // アクティブなユーザーかチェック
        if !user.is_active {
            return Err(AuthError::InvalidCredentials);
        }

        // JWTトークンを生成
        let claims = Claims::new(
            user.id,
            user.email.clone(),
            user.name.clone(),
            24,
            TokenType::Access,
        );
        let token = generate_token(&claims)?;

        // リフレッシュトークンを生成して保存
        let refresh_token = generate_refresh_token();
        refresh_tokens::save_refresh_token(&self.pool, user.id, &refresh_token, 30).await?;

        Ok(AuthResponse {
            token,
            refresh_token,
            user: user.into(),
        })
    }

    /// リフレッシュトークンを使って新しいトークンを取得
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<AuthResponse, AuthError> {
        // リフレッシュトークンを検証
        let user_id = refresh_tokens::verify_refresh_token(&self.pool, refresh_token)
            .await?
            .ok_or(AuthError::InvalidRefreshToken)?;

        // ユーザー情報を取得
        let user = users::find_user_by_id(&self.pool, user_id)
            .await?
            .ok_or(AuthError::UserNotFound)?;

        // アクティブなユーザーかチェック
        if !user.is_active {
            return Err(AuthError::InvalidRefreshToken);
        }

        // 古いリフレッシュトークンを削除
        refresh_tokens::delete_refresh_token(&self.pool, refresh_token).await?;

        // 新しいJWTトークンを生成
        let claims = Claims::new(
            user.id,
            user.email.clone(),
            user.name.clone(),
            24,
            TokenType::Access,
        );
        let token = generate_token(&claims)?;

        // 新しいリフレッシュトークンを生成して保存
        let new_refresh_token = generate_refresh_token();
        refresh_tokens::save_refresh_token(&self.pool, user.id, &new_refresh_token, 30).await?;

        Ok(AuthResponse {
            token,
            refresh_token: new_refresh_token,
            user: user.into(),
        })
    }

    /// トークンからユーザー情報を取得
    #[allow(dead_code)]
    pub async fn get_user_from_token(&self, token: &str) -> Result<UserResponse, AuthError> {
        // トークンを検証
        let token_data = verify_token(token)?;
        let user_id =
            Uuid::parse_str(&token_data.claims.sub).map_err(|_| AuthError::UserNotFound)?;

        // ユーザー情報を取得
        let user = users::find_user_by_id(&self.pool, user_id)
            .await?
            .ok_or(AuthError::UserNotFound)?;

        Ok(user.into())
    }

    /// ログアウト（リフレッシュトークンを削除）
    #[allow(dead_code)]
    pub async fn logout(&self, refresh_token: &str) -> Result<(), AuthError> {
        refresh_tokens::delete_refresh_token(&self.pool, refresh_token).await?;
        Ok(())
    }

    /// パスワードリセットリクエスト
    pub async fn request_password_reset(
        &self,
        request: ForgotPasswordRequest,
    ) -> Result<MessageResponse, AuthError> {
        use chrono::{Duration, Utc};
        use rand::{distributions::Alphanumeric, Rng};

        // ユーザーを検索（存在しない場合も成功レスポンスを返す - セキュリティのため）
        let user = match users::find_user_by_email(&self.pool, &request.email).await? {
            Some(user) => user,
            None => {
                return Ok(MessageResponse {
                    message: "パスワードリセットのメールを送信しました（メールアドレスが登録されている場合）".to_string(),
                });
            }
        };

        // レート制限チェック（5分以内に再リクエストは不可）
        if password_reset::has_recent_reset_request(&self.pool, user.id, 5).await? {
            return Err(AuthError::TooManyResetRequests);
        }

        // ランダムなトークンを生成
        let token: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        // トークンの有効期限を設定（1時間）
        let expires_at = Utc::now() + Duration::hours(1);

        // トークンをデータベースに保存
        password_reset::create_password_reset_token(&self.pool, user.id, &token, expires_at)
            .await?;

        // パスワードリセットメールを送信
        if let Err(e) = self
            .send_password_reset_email(&user.email, &user.name, &token)
            .await
        {
            tracing::error!("パスワードリセットメール送信エラー: {:?}", e);
            return Err(AuthError::EmailError(e.to_string()));
        }

        Ok(MessageResponse {
            message:
                "パスワードリセットのメールを送信しました（メールアドレスが登録されている場合）"
                    .to_string(),
        })
    }

    /// パスワードリセット実行
    pub async fn reset_password(
        &self,
        request: ResetPasswordRequest,
    ) -> Result<MessageResponse, AuthError> {
        use chrono::Utc;

        // トークンを検証
        let token_data = password_reset::get_password_reset_token(&self.pool, &request.token)
            .await?
            .ok_or(AuthError::InvalidOrExpiredResetToken)?;

        // トークンが使用済みか確認
        if token_data.used {
            return Err(AuthError::InvalidOrExpiredResetToken);
        }

        // トークンの有効期限を確認
        if token_data.expires_at < Utc::now() {
            return Err(AuthError::InvalidOrExpiredResetToken);
        }

        // 新しいパスワードをハッシュ化
        let password_hash = hash_password(&request.new_password)?;

        // パスワードを更新
        users::update_password(&self.pool, token_data.user_id, &password_hash).await?;

        // トークンを使用済みにする
        password_reset::mark_token_as_used(&self.pool, token_data.id).await?;

        Ok(MessageResponse {
            message: "パスワードが正常にリセットされました".to_string(),
        })
    }

    /// パスワードリセットメール送信（プライベートメソッド）
    async fn send_password_reset_email(
        &self,
        email: &str,
        name: &str,
        token: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use crate::services::email_service::EmailService;
        use std::collections::HashMap;

        // フロントエンドのリセットURLを構築
        let reset_url = format!(
            "{}/auth/reset-password?token={}",
            std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5173".to_string()),
            token
        );

        // メールテンプレート変数
        let mut variables = HashMap::new();
        variables.insert("name".to_string(), name.to_string());
        variables.insert("reset_url".to_string(), reset_url);
        variables.insert("valid_hours".to_string(), "1".to_string());

        // メールサービスを使用して送信
        let email_service = EmailService::new(self.pool.clone()).await?;
        email_service
            .send_password_reset_email(email, name, variables)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // テストは実際のデータベース接続が必要なため、統合テストで実装する
}

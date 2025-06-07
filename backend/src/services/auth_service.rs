// 認証サービス
// TODO: JWT生成、パスワードハッシュ化などの実装

use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

use crate::{
    database::{refresh_tokens, users},
    models::user::{AuthResponse, LoginRequest, RegisterRequest, UserResponse},
    utils::{
        jwt::{generate_refresh_token, generate_token, verify_token, Claims},
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
}

pub struct AuthService {
    pool: PgPool,
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
        let claims = Claims::new(user.id, user.email.clone(), user.name.clone(), 24);
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
        let claims = Claims::new(user.id, user.email.clone(), user.name.clone(), 24);
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
        let claims = Claims::new(user.id, user.email.clone(), user.name.clone(), 24);
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
}

#[cfg(test)]
mod tests {
    use super::*;

    // テストは実際のデータベース接続が必要なため、統合テストで実装する
}

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    Extension, Json,
};
use redis::Commands;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{
    crm::oauth::{SalesforceOAuthClient, TokenManager},
    middleware::auth::AuthUser,
    models::crm_oauth::{
        OAuthCallbackQuery, OAuthInitResponse, OAuthStatusResponse, SalesforceOAuthSettings,
    },
    AppState,
};

/// OAuth認証を開始
pub async fn init_salesforce_auth(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<OAuthInitResponse>, (StatusCode, Json<Value>)> {
    // OAuth設定を取得
    let settings = SalesforceOAuthSettings::from_env().map_err(|e| {
        tracing::error!("OAuth configuration error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": format!("OAuth configuration error: {}", e)
            })),
        )
    })?;

    // OAuthクライアントを作成
    let oauth_client = SalesforceOAuthClient::new(settings).map_err(|e| {
        tracing::error!("Failed to create OAuth client: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to initialize OAuth client"
            })),
        )
    })?;

    // 認証URLを生成
    let (auth_url, csrf_token) = oauth_client.get_auth_url();

    // CSRFトークンをRedisに保存（5分間有効）
    let redis_key = format!("oauth_state:{}", csrf_token.secret());
    let mut redis_conn = state.redis.get_connection().map_err(|e| {
        tracing::error!("Failed to get Redis connection: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to save authentication state"
            })),
        )
    })?;

    redis_conn
        .set_ex::<_, _, ()>(&redis_key, auth_user.user_id.to_string(), 300)
        .map_err(|e| {
            tracing::error!("Failed to save state to Redis: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to save authentication state"
                })),
            )
        })?;

    Ok(Json(OAuthInitResponse {
        auth_url,
        state: csrf_token.secret().to_string(),
    }))
}

/// OAuth認証コールバック
pub async fn salesforce_auth_callback(
    State(state): State<AppState>,
    Query(params): Query<OAuthCallbackQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    // CSRFトークンを検証
    let redis_key = format!("oauth_state:{}", params.state);
    let mut redis_conn = state.redis.get_connection().map_err(|e| {
        tracing::error!("Failed to get Redis connection: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to retrieve authentication state"
            })),
        )
    })?;

    let user_id_str: Option<String> = redis_conn.get(&redis_key).map_err(|e| {
        tracing::error!("Failed to get state from Redis: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to retrieve authentication state"
            })),
        )
    })?;

    let user_id_str = user_id_str.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Invalid or expired state parameter"
            })),
        )
    })?;

    let user_id = Uuid::parse_str(&user_id_str).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Invalid user ID in state"
            })),
        )
    })?;

    // Redisからstateを削除（一度だけ使用可能）
    let _: () = redis_conn.del(&redis_key).unwrap_or(());

    // OAuth設定を取得
    let settings = SalesforceOAuthSettings::from_env().map_err(|e| {
        tracing::error!("OAuth configuration error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "OAuth configuration error"
            })),
        )
    })?;

    // OAuthクライアントとトークンマネージャーを作成
    let oauth_client = SalesforceOAuthClient::new(settings).map_err(|e| {
        tracing::error!("Failed to create OAuth client: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to initialize OAuth client"
            })),
        )
    })?;

    let token_manager = TokenManager::new(state.db.clone(), oauth_client);

    // Authorization Codeを交換してトークンを保存
    token_manager
        .exchange_code_and_save(user_id, params.code)
        .await
        .map_err(|e| {
            tracing::error!("Failed to exchange code: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to complete authentication"
                })),
            )
        })?;

    // 成功画面へリダイレクト
    Ok(Redirect::to("/crm/oauth/success"))
}

/// 現在の認証状態を確認
pub async fn check_salesforce_auth_status(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<OAuthStatusResponse>, (StatusCode, Json<Value>)> {
    // OAuth設定を取得
    let settings = SalesforceOAuthSettings::from_env().map_err(|e| {
        tracing::error!("OAuth configuration error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "OAuth configuration error"
            })),
        )
    })?;

    // OAuthクライアントとトークンマネージャーを作成
    let oauth_client = SalesforceOAuthClient::new(settings).map_err(|e| {
        tracing::error!("Failed to create OAuth client: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to initialize OAuth client"
            })),
        )
    })?;

    let token_manager = TokenManager::new(state.db.clone(), oauth_client);

    // 認証状態を取得
    let auth_status = token_manager
        .get_auth_status(auth_user.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to check authentication status"
                })),
            )
        })?;

    if let Some(token) = auth_status {
        Ok(Json(OAuthStatusResponse {
            is_authenticated: true,
            expires_at: Some(token.expires_at),
            instance_url: Some(token.instance_url),
        }))
    } else {
        Ok(Json(OAuthStatusResponse {
            is_authenticated: false,
            expires_at: None,
            instance_url: None,
        }))
    }
}

/// Salesforce認証を解除（ログアウト）
pub async fn revoke_salesforce_auth(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // OAuth設定を取得
    let settings = SalesforceOAuthSettings::from_env().map_err(|e| {
        tracing::error!("OAuth configuration error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "OAuth configuration error"
            })),
        )
    })?;

    // OAuthクライアントとトークンマネージャーを作成
    let oauth_client = SalesforceOAuthClient::new(settings).map_err(|e| {
        tracing::error!("Failed to create OAuth client: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to initialize OAuth client"
            })),
        )
    })?;

    let token_manager = TokenManager::new(state.db.clone(), oauth_client);

    // トークンを削除
    token_manager
        .delete_tokens(auth_user.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete tokens: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to revoke authentication"
                })),
            )
        })?;

    Ok(Json(json!({
        "success": true,
        "message": "Salesforce authentication revoked successfully"
    })))
}

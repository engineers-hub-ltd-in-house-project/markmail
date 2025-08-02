use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{
    middleware::auth::AuthUser,
    models::crm::{CrmIntegrationSettings, CrmProviderType},
    services::crm_service::{
        oauth_integration::{AuthDetails, OAuthIntegrationService},
        CrmService, SaveIntegrationParams,
    },
    AppState,
};

/// OAuth2ベースのCRM統合状態レスポンス
#[derive(Debug, Serialize)]
pub struct CrmOAuthStatusResponse {
    pub is_authenticated: bool,
    pub auth_details: Option<AuthDetails>,
}

/// OAuth2ベースのCRM統合の作成リクエスト
#[derive(Debug, Deserialize)]
pub struct CreateOAuthIntegrationRequest {
    pub provider: CrmProviderType,
    pub settings: CrmIntegrationSettings,
}

/// OAuth2認証状態を確認
pub async fn check_crm_oauth_status(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<CrmOAuthStatusResponse>, (StatusCode, Json<Value>)> {
    let service = OAuthIntegrationService::new(state.db.clone()).map_err(|e| {
        tracing::error!("Failed to create OAuth integration service: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "サービスの初期化に失敗しました"
            })),
        )
    })?;

    let auth_details = service
        .get_auth_details(auth_user.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get auth details: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "認証状態の確認に失敗しました"
                })),
            )
        })?;

    Ok(Json(CrmOAuthStatusResponse {
        is_authenticated: auth_details.is_some(),
        auth_details,
    }))
}

/// OAuth2認証済みのCRM統合を作成
pub async fn create_oauth_integration(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateOAuthIntegrationRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    // OAuth2認証状態を確認
    let service = OAuthIntegrationService::new(state.db.clone()).map_err(|e| {
        tracing::error!("Failed to create OAuth integration service: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "サービスの初期化に失敗しました"
            })),
        )
    })?;

    let auth_details = service
        .get_auth_details(auth_user.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get auth details: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "認証状態の確認に失敗しました"
                })),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "error": "Salesforce OAuth認証が必要です"
                })),
            )
        })?;

    // アクセストークンを取得
    let access_token = service
        .get_access_token(auth_user.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get access token: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "アクセストークンの取得に失敗しました"
                })),
            )
        })?;

    // 統合設定を保存
    let params = SaveIntegrationParams {
        user_id: auth_user.user_id,
        provider: req.provider,
        org_id: &auth_details.org_id,
        instance_url: &auth_details.instance_url,
        access_token: &access_token,
        refresh_token: None, // OAuth2トークンマネージャーが管理
        settings: &req.settings,
    };

    let integration_id = CrmService::save_integration(&state.db, params)
        .await
        .map_err(|e| {
            tracing::error!("Failed to save integration: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": format!("統合設定の保存に失敗しました: {}", e)
                })),
            )
        })?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "success": true,
            "integration_id": integration_id,
            "message": "CRM統合が正常に作成されました"
        })),
    ))
}

/// OAuth2トークンを使用してCRMデータを同期
pub async fn sync_crm_data(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    // OAuth2認証状態を確認
    let oauth_service = OAuthIntegrationService::new(state.db.clone()).map_err(|e| {
        tracing::error!("Failed to create OAuth integration service: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "サービスの初期化に失敗しました"
            })),
        )
    })?;

    if !oauth_service
        .is_authenticated(auth_user.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to check authentication: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "認証状態の確認に失敗しました"
                })),
            )
        })?
    {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "Salesforce OAuth認証が必要です"
            })),
        ));
    }

    // CRMサービスを作成（OAuth2トークンを使用）
    let _crm_service = CrmService::new(state.db.clone(), auth_user.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create CRM service: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": format!("CRMサービスの初期化に失敗しました: {}", e)
                })),
            )
        })?;

    // ここで実際の同期処理を実行
    // TODO: 実装

    Ok(Json(json!({
        "success": true,
        "message": "同期を開始しました"
    })))
}

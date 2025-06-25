use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    middleware::auth::AuthUser,
    models::crm::{CrmIntegrationSettings, CrmProviderType},
    services::crm_service::{salesforce_auth::SalesforceAuth, CrmService},
    AppState,
};

/// CRM認証リクエスト
#[derive(Debug, Deserialize)]
pub struct CrmAuthRequest {
    pub provider: CrmProviderType,
    pub org_alias: String,
    pub auth_method: String, // "web" or "device"
}

/// CRM認証レスポンス
#[derive(Debug, Serialize)]
pub struct CrmAuthResponse {
    pub success: bool,
    pub message: String,
    pub org_info: Option<SalesforceOrgInfo>,
}

/// Salesforce組織情報（レスポンス用）
#[derive(Debug, Serialize)]
pub struct SalesforceOrgInfo {
    pub org_id: String,
    pub username: String,
    pub instance_url: String,
    pub connected_status: String,
}

/// CRM統合設定リクエスト
#[derive(Debug, Deserialize)]
pub struct CreateCrmIntegrationRequest {
    pub provider: CrmProviderType,
    pub org_alias: String,
    pub settings: CrmIntegrationSettings,
}

/// CRM統合設定レスポンス
#[derive(Debug, Serialize)]
pub struct CrmIntegrationResponse {
    pub id: Uuid,
    pub provider: CrmProviderType,
    pub is_active: bool,
    pub settings: CrmIntegrationSettings,
    pub connected_at: chrono::DateTime<chrono::Utc>,
}

/// CRM同期リクエスト
#[derive(Debug, Deserialize)]
pub struct CrmSyncRequest {
    pub entity_type: String, // "contacts" or "campaigns"
    pub entity_ids: Vec<Uuid>,
}

/// CRM同期レスポンス
#[derive(Debug, Serialize)]
pub struct CrmSyncResponse {
    pub total: usize,
    pub success: usize,
    pub failed: usize,
    pub results: Vec<SyncResultItem>,
}

#[derive(Debug, Serialize)]
pub struct SyncResultItem {
    pub entity_id: Uuid,
    pub crm_id: Option<String>,
    pub success: bool,
    pub error: Option<String>,
}

/// Salesforce認証を開始
pub async fn authenticate_salesforce(
    Extension(_auth_user): Extension<AuthUser>,
    Json(req): Json<CrmAuthRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Salesforce CLIがインストールされているか確認
    match SalesforceAuth::check_cli_installed() {
        Ok(true) => {}
        Ok(false) => {
            return Err((
                StatusCode::SERVICE_UNAVAILABLE,
                "Salesforce CLIがインストールされていません".to_string(),
            ));
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("CLI確認エラー: {}", e),
            ));
        }
    }

    // 認証を実行
    let result = match req.auth_method.as_str() {
        "web" => SalesforceAuth::login_web(&req.org_alias).await,
        "device" => SalesforceAuth::login_device(&req.org_alias).await,
        _ => {
            return Err((StatusCode::BAD_REQUEST, "無効な認証方法です".to_string()));
        }
    };

    match result {
        Ok(auth_result) => {
            let org_info = SalesforceOrgInfo {
                org_id: auth_result.org_info.org_id,
                username: auth_result.org_info.username,
                instance_url: auth_result.org_info.instance_url,
                connected_status: auth_result.org_info.connected_status,
            };

            Ok(Json(CrmAuthResponse {
                success: true,
                message: "認証に成功しました".to_string(),
                org_info: Some(org_info),
            }))
        }
        Err(e) => Err((
            StatusCode::UNAUTHORIZED,
            format!("認証に失敗しました: {}", e),
        )),
    }
}

/// CRM統合設定を作成
pub async fn create_crm_integration(
    State(_state): State<AppState>,
    Extension(_auth_user): Extension<AuthUser>,
    Json(req): Json<CreateCrmIntegrationRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 実際のデータベース保存を実装

    // 仮のレスポンス
    let response = CrmIntegrationResponse {
        id: Uuid::new_v4(),
        provider: req.provider,
        is_active: true,
        settings: req.settings,
        connected_at: chrono::Utc::now(),
    };

    Ok(Json(response))
}

/// CRM統合設定を取得
pub async fn get_crm_integration(
    State(_state): State<AppState>,
    Extension(_auth_user): Extension<AuthUser>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // TODO: データベースから設定を取得

    Err((
        StatusCode::NOT_FOUND,
        "CRM統合が設定されていません".to_string(),
    ))
}

/// CRM統合設定を削除
pub async fn delete_crm_integration(
    State(_state): State<AppState>,
    Extension(_auth_user): Extension<AuthUser>,
    Path(_integration_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: データベースから削除

    Ok(StatusCode::NO_CONTENT)
}

/// 連絡先を同期
pub async fn sync_contacts(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CrmSyncRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // CRMサービスを初期化
    let _crm_service = match CrmService::new(state.db.clone(), auth_user.user_id).await {
        Ok(service) => service,
        Err(e) => {
            return Err((
                StatusCode::SERVICE_UNAVAILABLE,
                format!("CRMサービスの初期化に失敗: {}", e),
            ));
        }
    };

    // TODO: 実際の同期処理を実装

    let response = CrmSyncResponse {
        total: req.entity_ids.len(),
        success: 0,
        failed: req.entity_ids.len(),
        results: req
            .entity_ids
            .into_iter()
            .map(|id| SyncResultItem {
                entity_id: id,
                crm_id: None,
                success: false,
                error: Some("同期機能は未実装です".to_string()),
            })
            .collect(),
    };

    Ok(Json(response))
}

/// キャンペーンを同期
pub async fn sync_campaigns(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CrmSyncRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // CRMサービスを初期化
    let _crm_service = match CrmService::new(state.db.clone(), auth_user.user_id).await {
        Ok(service) => service,
        Err(e) => {
            return Err((
                StatusCode::SERVICE_UNAVAILABLE,
                format!("CRMサービスの初期化に失敗: {}", e),
            ));
        }
    };

    // TODO: 実際の同期処理を実装

    let response = CrmSyncResponse {
        total: req.entity_ids.len(),
        success: 0,
        failed: req.entity_ids.len(),
        results: req
            .entity_ids
            .into_iter()
            .map(|id| SyncResultItem {
                entity_id: id,
                crm_id: None,
                success: false,
                error: Some("同期機能は未実装です".to_string()),
            })
            .collect(),
    };

    Ok(Json(response))
}

/// 組織一覧を取得
pub async fn list_salesforce_orgs(
    Extension(_auth_user): Extension<AuthUser>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match SalesforceAuth::list_orgs().await {
        Ok(orgs) => Ok(Json(orgs)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("組織一覧の取得に失敗: {}", e),
        )),
    }
}

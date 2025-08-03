use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    database::subscribers::find_subscriber_by_id,
    middleware::auth::AuthUser,
    models::crm::{CrmContact, CrmIntegrationSettings, CrmProviderType},
    services::crm_service::{salesforce_auth::SalesforceAuth, CrmService, SaveIntegrationParams},
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
                format!("CLI確認エラー: {e}"),
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
        Err(e) => Err((StatusCode::UNAUTHORIZED, format!("認証に失敗しました: {e}"))),
    }
}

/// CRM統合設定を作成
pub async fn create_crm_integration(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateCrmIntegrationRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Salesforce組織情報を取得
    let org_info = SalesforceAuth::get_org_info(&req.org_alias)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                format!("組織情報の取得に失敗: {e}"),
            )
        })?;

    // データベースに統合設定を保存
    let params = SaveIntegrationParams {
        user_id: auth_user.user_id,
        provider: req.provider.clone(),
        org_id: &req.org_alias,
        instance_url: &org_info.org_info.instance_url,
        access_token: &org_info.org_info.access_token,
        refresh_token: org_info.refresh_token.as_deref(),
        settings: &req.settings,
    };

    let integration_id = CrmService::save_integration(&state.db, params)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("統合設定の保存に失敗: {e}"),
            )
        })?;

    let response = CrmIntegrationResponse {
        id: integration_id,
        provider: req.provider,
        is_active: true,
        settings: req.settings,
        connected_at: chrono::Utc::now(),
    };

    Ok(Json(response))
}

/// CRM統合設定を取得
pub async fn get_crm_integration(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<CrmIntegrationResponse>, (StatusCode, String)> {
    // 現在はSalesforceのみサポート
    let Some(integration) =
        CrmService::get_integration(&state.db, auth_user.user_id, CrmProviderType::Salesforce)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("統合設定の取得に失敗: {e}"),
                )
            })?
    else {
        return Err((
            StatusCode::NOT_FOUND,
            "CRM統合が設定されていません".to_string(),
        ));
    };

    let settings = integration.get_sync_settings();

    let response = CrmIntegrationResponse {
        id: integration.id,
        provider: CrmProviderType::Salesforce,
        is_active: integration.is_active(),
        settings,
        connected_at: integration.created_at,
    };

    Ok(Json(response))
}

/// CRM統合設定を削除
pub async fn delete_crm_integration(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(integration_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    CrmService::deactivate_integration(&state.db, integration_id, auth_user.user_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("統合の無効化に失敗: {e}"),
            )
        })?;

    Ok(StatusCode::NO_CONTENT)
}

/// 連絡先を同期
pub async fn sync_contacts(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CrmSyncRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // CRMサービスを初期化
    let crm_service = match CrmService::new(state.db.clone(), auth_user.user_id).await {
        Ok(service) => service,
        Err(e) => {
            return Err((
                StatusCode::SERVICE_UNAVAILABLE,
                format!("CRMサービスの初期化に失敗: {e}"),
            ));
        }
    };

    // 統合情報を取得
    let integration = match CrmService::get_integration(
        &state.db,
        auth_user.user_id,
        CrmProviderType::Salesforce,
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("統合設定の取得に失敗: {e}"),
        )
    })? {
        Some(integration) => integration,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                "CRM統合が設定されていません".to_string(),
            ));
        }
    };

    // 連絡先を同期（実際の実装）
    let mut results = Vec::new();
    let mut success = 0;
    let mut failed = 0;

    for entity_id in req.entity_ids {
        // 購読者データを取得
        let subscriber = match find_subscriber_by_id(&state.db, entity_id, auth_user.user_id).await
        {
            Ok(Some(sub)) => sub,
            Ok(None) => {
                failed += 1;
                results.push(SyncResultItem {
                    entity_id,
                    crm_id: None,
                    success: false,
                    error: Some("購読者が見つかりません".to_string()),
                });
                continue;
            }
            Err(e) => {
                failed += 1;
                results.push(SyncResultItem {
                    entity_id,
                    crm_id: None,
                    success: false,
                    error: Some(format!("購読者の取得に失敗: {e}")),
                });
                continue;
            }
        };

        // 購読者をCRMコンタクトに変換
        let crm_contact = CrmContact::from_subscriber(&subscriber);

        // CRMに同期
        let sync_result = crm_service.provider().sync_contact(&crm_contact).await;

        match sync_result {
            Ok(result) => {
                if result.success {
                    success += 1;
                } else {
                    failed += 1;
                }

                results.push(SyncResultItem {
                    entity_id,
                    crm_id: Some(result.crm_id),
                    success: result.success,
                    error: result.error_message,
                });
            }
            Err(e) => {
                failed += 1;
                results.push(SyncResultItem {
                    entity_id,
                    crm_id: None,
                    success: false,
                    error: Some(e.to_string()),
                });
            }
        }
    }

    // 同期結果をログに記録
    let sync_results: Vec<_> = results
        .iter()
        .map(|r| crate::models::crm::CrmSyncResult {
            entity_type: "contact".to_string(),
            markmail_id: r.entity_id,
            crm_id: r.crm_id.clone().unwrap_or_default(),
            success: r.success,
            error_message: r.error.clone(),
            synced_at: chrono::Utc::now(),
        })
        .collect();

    if let Err(e) =
        CrmService::log_sync_activity(&state.db, integration.id, "contact", &sync_results).await
    {
        eprintln!("同期ログの記録に失敗: {e}");
    }

    let response = CrmSyncResponse {
        total: results.len(),
        success,
        failed,
        results,
    };

    Ok(Json(response))
}

/// すべての購読者を一括同期
pub async fn bulk_sync_subscribers(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    use crate::database::subscribers::list_subscribers;
    use crate::models::subscriber::SubscriberStatus;

    // CRMサービスを初期化
    let crm_service = match CrmService::new(state.db.clone(), auth_user.user_id).await {
        Ok(service) => service,
        Err(e) => {
            return Err((
                StatusCode::SERVICE_UNAVAILABLE,
                format!("CRMサービスの初期化に失敗: {e}"),
            ));
        }
    };

    // 統合情報を取得
    let integration = match CrmService::get_integration(
        &state.db,
        auth_user.user_id,
        CrmProviderType::Salesforce,
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("統合設定の取得に失敗: {e}"),
        )
    })? {
        Some(integration) => integration,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                "CRM統合が設定されていません".to_string(),
            ));
        }
    };

    // すべてのアクティブな購読者を取得
    let subscribers = list_subscribers(
        &state.db,
        auth_user.user_id,
        Some(10000), // 大量データ処理のため上限を設定
        Some(0),
        Some(SubscriberStatus::Active),
        None, // search
        None, // tag
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("購読者の取得に失敗: {e}"),
        )
    })?;

    // 購読者をCRMコンタクトに変換
    let crm_contacts: Vec<CrmContact> = subscribers
        .iter()
        .map(CrmContact::from_subscriber)
        .collect();

    // Bulk API 2.0を使用して一括同期
    let bulk_result = crm_service
        .provider()
        .bulk_sync_contacts(crm_contacts)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("一括同期に失敗: {e}"),
            )
        })?;

    // 同期結果をログに記録
    if let Err(e) =
        CrmService::log_sync_activity(&state.db, integration.id, "contact", &bulk_result.results)
            .await
    {
        eprintln!("同期ログの記録に失敗: {e}");
    }

    // レスポンスを構築
    let response = CrmSyncResponse {
        total: bulk_result.total,
        success: bulk_result.success,
        failed: bulk_result.failed,
        results: bulk_result
            .results
            .into_iter()
            .map(|r| SyncResultItem {
                entity_id: r.markmail_id,
                crm_id: Some(r.crm_id),
                success: r.success,
                error: r.error_message,
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
    let crm_service = match CrmService::new(state.db.clone(), auth_user.user_id).await {
        Ok(service) => service,
        Err(e) => {
            return Err((
                StatusCode::SERVICE_UNAVAILABLE,
                format!("CRMサービスの初期化に失敗: {e}"),
            ));
        }
    };

    // 統合情報を取得
    let integration = match CrmService::get_integration(
        &state.db,
        auth_user.user_id,
        CrmProviderType::Salesforce,
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("統合設定の取得に失敗: {e}"),
        )
    })? {
        Some(integration) => integration,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                "CRM統合が設定されていません".to_string(),
            ));
        }
    };

    // キャンペーンを同期（実際の実装）
    let mut results = Vec::new();
    let mut success = 0;
    let mut failed = 0;

    for entity_id in req.entity_ids {
        // TODO: 実際のキャンペーンデータを取得して同期
        // 現在は仮の実装
        let sync_result = crm_service
            .provider()
            .sync_campaign(&crate::models::crm::CrmCampaign {
                id: None,
                markmail_id: entity_id,
                name: "Test Campaign".to_string(),
                status: "active".to_string(),
                start_date: None,
                end_date: None,
                member_count: 0,
                email_stats: crate::models::crm::CrmEmailStats {
                    sent: 0,
                    opened: 0,
                    clicked: 0,
                    bounced: 0,
                    unsubscribed: 0,
                },
            })
            .await;

        match sync_result {
            Ok(result) => {
                if result.success {
                    success += 1;
                } else {
                    failed += 1;
                }

                results.push(SyncResultItem {
                    entity_id,
                    crm_id: Some(result.crm_id),
                    success: result.success,
                    error: result.error_message,
                });
            }
            Err(e) => {
                failed += 1;
                results.push(SyncResultItem {
                    entity_id,
                    crm_id: None,
                    success: false,
                    error: Some(e.to_string()),
                });
            }
        }
    }

    // 同期ログを記録
    let sync_results: Vec<_> = results
        .iter()
        .map(|r| crate::models::crm::CrmSyncResult {
            entity_type: "campaign".to_string(),
            markmail_id: r.entity_id,
            crm_id: r.crm_id.clone().unwrap_or_default(),
            success: r.success,
            error_message: r.error.clone(),
            synced_at: chrono::Utc::now(),
        })
        .collect();

    if let Err(e) =
        CrmService::log_sync_activity(&state.db, integration.id, "campaign", &sync_results).await
    {
        eprintln!("同期ログの記録に失敗: {e}");
    }

    let response = CrmSyncResponse {
        total: results.len(),
        success,
        failed,
        results,
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
            format!("組織一覧の取得に失敗: {e}"),
        )),
    }
}

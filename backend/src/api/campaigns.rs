use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;
use validator::Validate;

use crate::{
    database::{
        campaigns::{self, find_campaign_by_id},
        templates,
    },
    middleware::auth::AuthUser,
    models::campaign::{
        CampaignListResponse, CampaignResponse, CampaignStatus, CreateCampaignRequest,
        ListCampaignOptions, ScheduleCampaignRequest, UpdateCampaignRequest,
    },
    services::{
        campaign_service::CampaignService, markdown_service::MarkdownService,
        subscriber_service::SubscriberService,
    },
    AppState,
};

/// キャンペーン一覧を取得
pub async fn list_campaigns(
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<ListCampaignOptions>,
    State(state): State<AppState>,
) -> Result<Json<CampaignListResponse>, (StatusCode, Json<Value>)> {
    let limit = query.limit.unwrap_or(50).min(100);
    let offset = query.offset.unwrap_or(0);

    match campaigns::list_user_campaigns(&state.db, auth_user.user_id, &query).await {
        Ok(campaign_list) => {
            let total = campaigns::count_user_campaigns(&state.db, auth_user.user_id, query.status)
                .await
                .unwrap_or(0);

            let response = CampaignListResponse {
                campaigns: campaign_list.into_iter().map(|c| c.into()).collect(),
                total,
                limit,
                offset,
            };

            Ok(Json(response))
        }
        Err(e) => {
            tracing::error!("キャンペーン一覧取得エラー: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "キャンペーン一覧の取得に失敗しました"
                })),
            ))
        }
    }
}

/// キャンペーンを新規作成
pub async fn create_campaign(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Json(payload): Json<CreateCampaignRequest>,
) -> Result<Json<CampaignResponse>, (StatusCode, Json<Value>)> {
    // バリデーション
    if let Err(errors) = payload.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "バリデーションエラー",
                "details": errors
            })),
        ));
    }

    // キャンペーンサービスを使用して作成
    let campaign_service = CampaignService::new();
    match campaign_service
        .create_campaign(&state.db, auth_user.user_id, &payload)
        .await
    {
        Ok(campaign) => {
            tracing::info!("キャンペーン作成成功: {}", campaign.id);
            Ok(Json(campaign.into()))
        }
        Err(e) => {
            tracing::error!("キャンペーン作成エラー: {}", e);
            Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": e
                })),
            ))
        }
    }
}

/// キャンペーン詳細を取得
pub async fn get_campaign(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<CampaignResponse>, (StatusCode, Json<Value>)> {
    match campaigns::find_campaign_by_id(&state.db, id, auth_user.user_id).await {
        Ok(Some(campaign)) => Ok(Json(campaign.into())),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "キャンペーンが見つかりません"
            })),
        )),
        Err(e) => {
            tracing::error!("キャンペーン取得エラー: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "キャンペーンの取得に失敗しました"
                })),
            ))
        }
    }
}

/// キャンペーンを更新
pub async fn update_campaign(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateCampaignRequest>,
) -> Result<Json<CampaignResponse>, (StatusCode, Json<Value>)> {
    // バリデーション
    if let Err(errors) = payload.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "バリデーションエラー",
                "details": errors
            })),
        ));
    }

    // キャンペーンサービスを使用して更新
    let campaign_service = CampaignService::new();
    match campaign_service
        .update_campaign(&state.db, id, auth_user.user_id, &payload)
        .await
    {
        Ok(campaign) => {
            tracing::info!("キャンペーン更新成功: {}", campaign.id);
            Ok(Json(campaign.into()))
        }
        Err(e) => {
            tracing::error!("キャンペーン更新エラー: {}", e);
            Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": e
                })),
            ))
        }
    }
}

/// キャンペーンを削除
pub async fn delete_campaign(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match campaigns::delete_campaign(&state.db, id, auth_user.user_id).await {
        Ok(true) => {
            tracing::info!("キャンペーン削除成功: {}", id);
            Ok(Json(json!({
                "message": "キャンペーンが削除されました",
                "campaign_id": id
            })))
        }
        Ok(false) => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "キャンペーンが見つからないか、送信済み/送信中のキャンペーンは削除できません"
            })),
        )),
        Err(e) => {
            tracing::error!("キャンペーン削除エラー: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "キャンペーンの削除に失敗しました"
                })),
            ))
        }
    }
}

/// キャンペーン送信前の検証
pub async fn validate_campaign_before_send(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // キャンペーン情報を取得
    let campaign = match campaigns::find_campaign_by_id(&state.db, id, auth_user.user_id).await {
        Ok(Some(campaign)) => campaign,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "キャンペーンが見つかりません"
                })),
            ))
        }
        Err(e) => {
            tracing::error!("キャンペーン取得エラー: {:?}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "キャンペーンの取得に失敗しました"
                })),
            ));
        }
    };

    // テンプレート情報を取得
    let template = match templates::find_template_by_id(
        &state.db,
        campaign.template_id,
        Some(auth_user.user_id),
    )
    .await
    {
        Ok(Some(template)) => template,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "テンプレートが見つかりません"
                })),
            ))
        }
        Err(e) => {
            tracing::error!("テンプレート取得エラー: {:?}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "テンプレートの取得に失敗しました"
                })),
            ));
        }
    };

    // マークダウンサービスを初期化
    let markdown_service = MarkdownService::new();

    // テンプレートから変数を抽出
    let used_variables = markdown_service.extract_variables(&template.markdown_content);
    let subject_variables = markdown_service.extract_variables(&template.subject_template);

    // 全ての使用変数をマージ（重複除去）
    let mut all_variables: Vec<String> = used_variables;
    for var in subject_variables {
        if !all_variables.contains(&var) {
            all_variables.push(var);
        }
    }

    // システムで自動的に提供される標準変数
    let standard_variables = vec![
        "name".to_string(),
        "first_name".to_string(),
        "email".to_string(),
        "unsubscribe_url".to_string(),
    ];

    // カスタム変数（標準変数以外）を特定
    let custom_variables: Vec<String> = all_variables
        .iter()
        .filter(|var| !standard_variables.contains(var))
        .cloned()
        .collect();

    // テンプレートに定義されている変数
    let defined_variables: Vec<String> =
        if let serde_json::Value::Object(vars) = &template.variables {
            vars.keys().cloned().collect()
        } else {
            vec![]
        };

    // 不足している変数（カスタム変数のうち定義されていないもの）
    let missing_variables: Vec<String> = custom_variables
        .iter()
        .filter(|var| !defined_variables.contains(var))
        .cloned()
        .collect();

    // 購読者数を取得
    let campaign_service = CampaignService::new();
    let subscribers = campaign_service
        .get_campaign_subscribers(&state.db, id, auth_user.user_id)
        .await
        .map_err(|e| {
            tracing::error!("購読者取得エラー: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "購読者の取得に失敗しました"
                })),
            )
        })?;

    let is_valid = missing_variables.is_empty() && !subscribers.is_empty();

    Ok(Json(json!({
        "is_valid": is_valid,
        "subscriber_count": subscribers.len(),
        "used_variables": all_variables,
        "standard_variables": standard_variables,
        "custom_variables": custom_variables,
        "defined_variables": defined_variables,
        "missing_variables": missing_variables,
        "warnings": if missing_variables.is_empty() {
            vec![]
        } else {
            vec![format!(
                "以下の変数がテンプレートで定義されていません: {}",
                missing_variables.join(", ")
            )]
        }
    })))
}

/// キャンペーン送信
pub async fn send_campaign(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // キャンペーンサービスを使用して送信開始
    let campaign_service = CampaignService::new();
    match campaign_service
        .start_sending_campaign(&state.db, id, auth_user.user_id)
        .await
    {
        Ok(campaign) => {
            tracing::info!("キャンペーン送信開始: {}", campaign.id);

            // 非同期でメール送信処理を開始
            let db = state.db.clone();
            let campaign_id = campaign.id;
            let user_id = auth_user.user_id;

            tokio::spawn(async move {
                let campaign_service = CampaignService::new();
                if let Err(e) = campaign_service
                    .process_campaign_sending(&db, campaign_id, user_id)
                    .await
                {
                    tracing::error!("キャンペーン送信処理エラー: {}", e);
                }
            });

            Ok(Json(json!({
                "message": "キャンペーンの送信を開始しました",
                "campaign_id": id,
                "status": "sending"
            })))
        }
        Err(e) => {
            tracing::error!("キャンペーン送信エラー: {}", e);
            Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": e
                })),
            ))
        }
    }
}

/// キャンペーンをスケジュール
pub async fn schedule_campaign(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ScheduleCampaignRequest>,
) -> Result<Json<CampaignResponse>, (StatusCode, Json<Value>)> {
    // キャンペーンサービスを使用してスケジュール
    let campaign_service = CampaignService::new();
    match campaign_service
        .schedule_campaign(&state.db, id, auth_user.user_id, &payload)
        .await
    {
        Ok(campaign) => {
            tracing::info!("キャンペーンスケジュール設定: {}", campaign.id);
            Ok(Json(campaign.into()))
        }
        Err(e) => {
            tracing::error!("キャンペーンスケジュールエラー: {}", e);
            Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": e
                })),
            ))
        }
    }
}

/// キャンペーンプレビュー
pub async fn preview_campaign(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<PreviewCampaignResponse>, (StatusCode, Json<Value>)> {
    // キャンペーンサービスを使用してプレビュー生成
    let campaign_service = CampaignService::new();
    match campaign_service
        .generate_campaign_preview(&state.db, id, auth_user.user_id)
        .await
    {
        Ok(html) => Ok(Json(PreviewCampaignResponse { html })),
        Err(e) => {
            tracing::error!("キャンペーンプレビューエラー: {}", e);
            Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": e
                })),
            ))
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PreviewCampaignResponse {
    pub html: String,
}

/// キャンペーンの購読者一覧を取得
pub async fn get_campaign_subscribers(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // キャンペーンの存在確認と権限チェック
    let _campaign = match find_campaign_by_id(&state.db, id, auth_user.user_id).await {
        Ok(campaign) => campaign,
        Err(e) => {
            tracing::error!("キャンペーン取得エラー: {}", e);
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "キャンペーンが見つかりません"
                })),
            ));
        }
    };

    // 購読者サービスを使用して購読者を取得
    let subscriber_service = SubscriberService::new();

    // TODO: キャンペーンに関連する購読者のみを取得する実装
    // 現在は全てのアクティブな購読者を取得
    match subscriber_service
        .list_subscribers(
            &state.db,
            auth_user.user_id,
            None, // ステータスフィルタを一時的に無効化
            None,
            100, // 最大100件
            0,
        )
        .await
    {
        Ok(subscribers) => Ok(Json(json!({
            "subscribers": subscribers,
            "campaign_id": id,
            "total": subscribers.len()
        }))),
        Err(e) => {
            tracing::error!("購読者取得エラー: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "購読者の取得に失敗しました"
                })),
            ))
        }
    }
}

/// キャンペーンの再送
pub async fn resend_campaign(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // キャンペーンの状態を確認
    let campaign = match campaigns::find_campaign_by_id(&state.db, id, auth_user.user_id).await {
        Ok(Some(campaign)) => campaign,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "キャンペーンが見つかりません"
                })),
            ))
        }
        Err(e) => {
            tracing::error!("キャンペーン取得エラー: {:?}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "キャンペーンの取得に失敗しました"
                })),
            ));
        }
    };

    // 再送可能なステータスかチェック（sent, error, cancelled, sending）
    match campaign.status {
        CampaignStatus::Sent
        | CampaignStatus::Error
        | CampaignStatus::Cancelled
        | CampaignStatus::Sending => {
            // ステータスをdraftに戻す
            if let Err(e) = campaigns::update_campaign_status(
                &state.db,
                id,
                auth_user.user_id,
                CampaignStatus::Draft,
            )
            .await
            {
                tracing::error!("キャンペーンステータス更新エラー: {:?}", e);
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "キャンペーンステータスの更新に失敗しました"
                    })),
                ));
            }

            // 送信統計をリセット
            if let Err(e) =
                campaigns::update_campaign_stats(&state.db, id, Some(0), Some(0), Some(0), Some(0))
                    .await
            {
                tracing::error!("キャンペーン統計リセットエラー: {:?}", e);
            }

            // 送信処理を開始
            let campaign_service = CampaignService::new();
            match campaign_service
                .start_sending_campaign(&state.db, id, auth_user.user_id)
                .await
            {
                Ok(campaign) => {
                    tracing::info!("キャンペーン再送開始: {}", campaign.id);

                    // 非同期でメール送信処理を開始
                    let db = state.db.clone();
                    let campaign_id = campaign.id;
                    let user_id = auth_user.user_id;

                    tokio::spawn(async move {
                        let campaign_service = CampaignService::new();
                        if let Err(e) = campaign_service
                            .process_campaign_sending(&db, campaign_id, user_id)
                            .await
                        {
                            tracing::error!("キャンペーン再送処理エラー: {}", e);
                        }
                    });

                    Ok(Json(json!({
                        "message": "キャンペーンの再送を開始しました",
                        "campaign_id": id,
                        "status": "sending"
                    })))
                }
                Err(e) => {
                    tracing::error!("キャンペーン再送開始エラー: {}", e);
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({
                            "error": format!("キャンペーンの再送に失敗しました: {}", e)
                        })),
                    ))
                }
            }
        }
        _ => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": format!("現在のステータス({})では再送できません。sent, error, cancelled, sendingステータスのキャンペーンのみ再送可能です。", campaign.status)
            })),
        )),
    }
}

/// キャンペーン関連のルーターを構築
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_campaigns).post(create_campaign))
        .route(
            "/:id",
            get(get_campaign)
                .put(update_campaign)
                .delete(delete_campaign),
        )
        .route("/:id/send", post(send_campaign))
        .route("/:id/resend", post(resend_campaign))
        .route("/:id/validate", get(validate_campaign_before_send))
        .route("/:id/schedule", post(schedule_campaign))
        .route("/:id/preview", get(preview_campaign))
        .route("/:id/subscribers", get(get_campaign_subscribers))
}

use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;
use validator::Validate;

use crate::middleware::auth::AuthUser;
use crate::models::sequence::TriggerType;
use crate::models::subscriber::{
    CreateSubscriberRequest, ImportSubscribersRequest, SubscriberStatus, UpdateSubscriberRequest,
};
use crate::services::{sequence_service::SequenceService, subscriber_service};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub status: Option<String>,
    pub search: Option<String>,
    pub tag: Option<String>,
}

/// 購読者一覧を取得
pub async fn list_subscribers(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Value>, StatusCode> {
    // ステータスフィルターがある場合、文字列からenumへ変換
    let status_filter = match &query.status {
        Some(status_str) => match status_str.to_lowercase().as_str() {
            "active" => Some(SubscriberStatus::Active),
            "unsubscribed" => Some(SubscriberStatus::Unsubscribed),
            "bounced" => Some(SubscriberStatus::Bounced),
            "complained" => Some(SubscriberStatus::Complained),
            _ => None,
        },
        None => None,
    };

    // サービスから購読者一覧を取得
    let result = subscriber_service::list_subscribers(
        &state.db,
        auth_user.user_id,
        query.limit,
        query.offset,
        status_filter,
        query.search.as_deref(),
        query.tag.as_deref(),
    )
    .await
    .map_err(|e| {
        eprintln!("購読者一覧取得エラー: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(json!({
        "subscribers": result.subscribers,
        "total": result.total,
        "available_tags": result.available_tags,
        "limit": query.limit.unwrap_or(50),
        "offset": query.offset.unwrap_or(0)
    })))
}

/// 購読者詳細を取得
pub async fn get_subscriber(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(subscriber_id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    let subscriber =
        subscriber_service::get_subscriber(&state.db, subscriber_id, auth_user.user_id)
            .await
            .map_err(|e| {
                eprintln!("購読者取得エラー: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

    match subscriber {
        Some(subscriber) => Ok(Json(json!({ "subscriber": subscriber }))),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// 購読者を追加
pub async fn add_subscriber(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(payload): Json<CreateSubscriberRequest>,
) -> Result<Json<Value>, StatusCode> {
    // リクエストのバリデーション
    if let Err(_errors) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // 購読者を作成
    let subscriber = subscriber_service::create_subscriber(&state.db, auth_user.user_id, payload)
        .await
        .map_err(|e| {
            // 既に存在するメールアドレスの場合
            if e.to_string().contains("既に登録されています") {
                return StatusCode::CONFLICT;
            }
            eprintln!("購読者追加エラー: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // シーケンスへの自動エンロールメントをトリガー
    let sequence_service = SequenceService::new();
    if let Err(e) = sequence_service
        .process_trigger_enrollment(
            &state.db,
            auth_user.user_id,
            TriggerType::SubscriberCreated,
            subscriber.id,
            None,
        )
        .await
    {
        eprintln!("シーケンスエンロールメントエラー: {}", e);
        // エラーが発生してもレスポンスは返す（購読者作成は成功しているため）
    }

    Ok(Json(json!({
        "message": "購読者が追加されました",
        "subscriber": subscriber
    })))
}

/// 購読者を更新
pub async fn update_subscriber(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(subscriber_id): Path<Uuid>,
    Json(payload): Json<UpdateSubscriberRequest>,
) -> Result<Json<Value>, StatusCode> {
    // リクエストのバリデーション
    if let Err(_errors) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // 購読者を更新
    let subscriber =
        subscriber_service::update_subscriber(&state.db, subscriber_id, auth_user.user_id, payload)
            .await
            .map_err(|e| {
                // 既に存在するメールアドレスの場合
                if e.to_string().contains("既に別の購読者に登録されています") {
                    return StatusCode::CONFLICT;
                }
                eprintln!("購読者更新エラー: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

    match subscriber {
        Some(subscriber) => Ok(Json(json!({
            "message": "購読者が更新されました",
            "subscriber": subscriber
        }))),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// 購読者を削除
pub async fn delete_subscriber_by_id(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(subscriber_id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    let deleted =
        subscriber_service::delete_subscriber(&state.db, subscriber_id, auth_user.user_id)
            .await
            .map_err(|e| {
                eprintln!("購読者削除エラー: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

    if deleted {
        Ok(Json(json!({
            "message": "購読者が削除されました"
        })))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// CSVから購読者をインポート
pub async fn import_subscribers_from_csv(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(payload): Json<ImportSubscribersRequest>,
) -> Result<Json<Value>, StatusCode> {
    // CSVのインポート処理を実行
    let result =
        subscriber_service::import_subscribers_from_csv(&state.db, auth_user.user_id, payload)
            .await
            .map_err(|e| {
                eprintln!("購読者インポートエラー: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

    Ok(Json(json!({
        "message": "CSVのインポートが完了しました",
        "imported": result.imported_count,
        "errors": result.errors
    })))
}

/// 購読者タグ一覧を取得
pub async fn get_subscriber_tags(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<Value>, StatusCode> {
    let tags = crate::database::subscribers::get_all_tags(&state.db, auth_user.user_id)
        .await
        .map_err(|e| {
            eprintln!("タグ一覧取得エラー: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(json!({
        "tags": tags
    })))
}

/// 購読者関連のルーターを構築
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_subscribers).post(add_subscriber))
        .route("/tags", get(get_subscriber_tags))
        .route("/import", post(import_subscribers_from_csv))
        .route(
            "/:id",
            get(get_subscriber)
                .put(update_subscriber)
                .delete(delete_subscriber_by_id),
        )
}

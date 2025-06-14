use crate::middleware::auth::AuthUser;
use crate::models::subscription::{CancelRequest, PlansResponse, UpgradeRequest};
use crate::services::subscription_service;
use crate::AppState;
use axum::{
    extract::{Extension, Query, State},
    response::Json,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// 全てのプランを取得
pub async fn get_plans(State(state): State<AppState>) -> Json<PlansResponse> {
    tracing::debug!("Getting subscription plans");

    match subscription_service::get_plans(&state.db).await {
        Ok(plans) => {
            tracing::debug!("Found {} plans", plans.len());
            Json(PlansResponse { plans })
        }
        Err(e) => {
            tracing::error!("Failed to get plans: {:?}", e);
            Json(PlansResponse { plans: vec![] })
        }
    }
}

/// 現在のサブスクリプション情報を取得
pub async fn get_subscription(
    Extension(user): Extension<AuthUser>,
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    match subscription_service::get_user_subscription_details(&state.db, user.user_id).await {
        Ok(details) => Json(serde_json::to_value(details).unwrap_or(serde_json::json!({}))),
        Err(e) => {
            tracing::error!("Failed to get subscription details: {:?}", e);
            Json(serde_json::json!({"error": "サブスクリプション情報の取得に失敗しました"}))
        }
    }
}

/// プランをアップグレード
pub async fn upgrade_plan(
    Extension(user): Extension<AuthUser>,
    State(state): State<AppState>,
    Json(request): Json<UpgradeRequest>,
) -> Json<serde_json::Value> {
    match subscription_service::upgrade_plan(&state.db, user.user_id, &request).await {
        Ok(subscription) => {
            Json(serde_json::to_value(subscription).unwrap_or(serde_json::json!({})))
        }
        Err(e) => Json(serde_json::json!({"error": e.to_string()})),
    }
}

/// サブスクリプションをキャンセル
pub async fn cancel_subscription(
    Extension(user): Extension<AuthUser>,
    State(state): State<AppState>,
    Json(request): Json<CancelRequest>,
) -> Json<serde_json::Value> {
    match subscription_service::cancel_subscription(&state.db, user.user_id, &request).await {
        Ok(subscription) => {
            Json(serde_json::to_value(subscription).unwrap_or(serde_json::json!({})))
        }
        Err(e) => Json(serde_json::json!({"error": e.to_string()})),
    }
}

/// 支払い履歴を取得
pub async fn get_payment_history(
    Extension(user): Extension<AuthUser>,
    Query(query): Query<PaginationQuery>,
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    match subscription_service::get_payment_history(&state.db, user.user_id, limit, offset).await {
        Ok(history) => Json(serde_json::json!({
            "payments": history,
            "limit": limit,
            "offset": offset
        })),
        Err(_) => Json(serde_json::json!({
            "payments": [],
            "limit": limit,
            "offset": offset
        })),
    }
}

/// 使用量を取得
pub async fn get_usage(
    Extension(user): Extension<AuthUser>,
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    match subscription_service::get_user_subscription_details(&state.db, user.user_id).await {
        Ok(details) => Json(serde_json::to_value(details.usage).unwrap_or(serde_json::json!({}))),
        Err(_) => Json(serde_json::json!({})),
    }
}

use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{
    middleware::auth::AuthUser,
    models::ai_usage::{AiUsageLog, AiUsageStats},
    services::ai_usage_service::AiUsageService,
    AppState,
};

#[derive(Debug, Deserialize)]
pub struct GetUsageHistoryQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct GetUsageHistoryResponse {
    pub usage_logs: Vec<AiUsageLog>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

/// AI使用統計を取得
pub async fn get_ai_usage_stats(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
) -> Result<Json<AiUsageStats>, (StatusCode, Json<Value>)> {
    let stats = AiUsageService::get_monthly_usage(&state.db, auth_user.user_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": format!("Failed to get AI usage stats: {}", e)
                })),
            )
        })?;

    Ok(Json(stats))
}

/// AI使用履歴を取得
pub async fn get_ai_usage_history(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Query(query): Query<GetUsageHistoryQuery>,
) -> Result<Json<GetUsageHistoryResponse>, (StatusCode, Json<Value>)> {
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = query.offset.unwrap_or(0);

    let usage_logs = AiUsageService::get_usage_history(&state.db, auth_user.user_id, limit, offset)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": format!("Failed to get AI usage history: {}", e)
                })),
            )
        })?;

    // 総数を取得
    let stats = AiUsageService::get_monthly_usage(&state.db, auth_user.user_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": format!("Failed to get AI usage stats: {}", e)
                })),
            )
        })?;

    Ok(Json(GetUsageHistoryResponse {
        usage_logs,
        total: stats.total_usage,
        limit,
        offset,
    }))
}

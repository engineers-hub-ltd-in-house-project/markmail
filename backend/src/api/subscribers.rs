use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};

use crate::AppState;

pub async fn list_subscribers(State(_state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "subscribers": [],
        "total": 0
    })))
}

pub async fn add_subscriber(
    State(_state): State<AppState>,
    Json(_payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: 購読者追加ロジックを実装
    Ok(Json(json!({
        "message": "購読者が追加されました"
    })))
}

pub async fn import_csv(
    State(_state): State<AppState>,
    Json(_payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: CSV インポートロジックを実装
    Ok(Json(json!({
        "message": "CSVのインポートが完了しました",
        "imported": 0
    })))
}

/// 購読者関連のルーターを構築
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_subscribers).post(add_subscriber))
        .route("/import", post(import_csv))
}

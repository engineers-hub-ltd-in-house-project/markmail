use axum::{extract::State, http::StatusCode, response::Json};
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
    Ok(Json(json!({
        "message": "購読者が追加されました"
    })))
}

pub async fn import_csv(
    State(_state): State<AppState>,
    Json(_payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "message": "CSVファイルからの購読者インポートが完了しました",
        "imported_count": 0
    })))
}

#[allow(dead_code)]
pub async fn import_subscribers(
    State(_state): State<AppState>,
    Json(_payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "message": "CSVファイルからの購読者インポートが完了しました",
        "imported_count": 0
    })))
}

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::AppState;

pub async fn list_campaigns(State(_state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "campaigns": [],
        "total": 0
    })))
}

pub async fn create_campaign(
    State(_state): State<AppState>,
    Json(_payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "message": "キャンペーンが作成されました"
    })))
}

pub async fn get_campaign(
    State(_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "campaign": {
            "id": id,
            "name": "サンプルキャンペーン"
        }
    })))
}

pub async fn send_campaign(
    State(_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "message": "キャンペーンの送信を開始しました",
        "campaign_id": id
    })))
}

pub async fn schedule_campaign(
    State(_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(_payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "message": "キャンペーンがスケジュールされました",
        "campaign_id": id
    })))
}

#[allow(dead_code)]
pub async fn update_campaign(
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
    Json(_payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "message": "キャンペーンが更新されました"
    })))
}

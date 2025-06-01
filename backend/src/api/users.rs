use axum::{extract::State, http::StatusCode, response::Json};
use serde_json::{json, Value};

use crate::AppState;

pub async fn get_profile(State(_state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    // TODO: プロフィール取得ロジックを実装
    Ok(Json(json!({
        "user": {
            "id": "dummy-user-id",
            "email": "user@example.com",
            "name": "テストユーザー"
        }
    })))
}

pub async fn update_profile(
    State(_state): State<AppState>,
    Json(_payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: プロフィール更新ロジックを実装
    Ok(Json(json!({
        "message": "プロフィールが更新されました"
    })))
}

use axum::{extract::State, http::StatusCode, response::Json};
use serde_json::{json, Value};

use crate::AppState;

pub async fn render_markdown(
    State(_state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: マークダウンレンダリングロジックを実装
    Ok(Json(json!({
        "html": "<h1>レンダリング結果</h1>",
        "variables": []
    })))
}

pub async fn validate_markdown(
    State(_state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: マークダウンバリデーションロジックを実装
    Ok(Json(json!({
        "valid": true,
        "errors": []
    })))
}

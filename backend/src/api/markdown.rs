use axum::{extract::State, http::StatusCode, response::Json};
use serde_json::{json, Value};

use crate::AppState;

pub async fn render_markdown(
    State(_state): State<AppState>,
    Json(_payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: マークダウンレンダリングロジックを実装
    Ok(Json(json!({
        "html": "<p>レンダリングされたHTML</p>"
    })))
}

pub async fn validate_markdown(
    State(_state): State<AppState>,
    Json(_payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: マークダウン検証ロジックを実装
    Ok(Json(json!({
        "valid": true,
        "errors": []
    })))
}

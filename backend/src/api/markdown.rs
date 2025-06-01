use axum::{extract::State, http::StatusCode, response::Json};
use serde_json::{json, Value};

use crate::AppState;

pub async fn render_markdown(
    State(_state): State<AppState>,
    Json(_payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: マークダウンレンダリングロジックを実装
    Ok(Json(json!({
        "html": "<h1>レンダリング結果</h1>",
        "variables": []
    })))
}

pub async fn validate_markdown(
    State(_state): State<AppState>,
    Json(_payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: マークダウンバリデーションロジックを実装
    Ok(Json(json!({
        "valid": true,
        "errors": []
    })))
}

#[allow(dead_code)]
pub async fn preview_template(
    State(_state): State<AppState>,
    Json(_payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: プレビューテンプレートロジックを実装
    Ok(Json(json!({
        "html": "<h1>プレビューテンプレート</h1>",
        "variables": []
    })))
}

use axum::{extract::State, http::StatusCode, response::Json};
use serde_json::{json, Value};

use crate::AppState;

pub async fn github_repos(State(_state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    // TODO: GitHub リポジトリ一覧取得ロジックを実装
    Ok(Json(json!({
        "repositories": []
    })))
}

pub async fn import_from_github(
    State(_state): State<AppState>,
    Json(_payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: GitHub連携ロジックを実装
    Ok(Json(json!({
        "message": "GitHubからのインポートが完了しました"
    })))
}

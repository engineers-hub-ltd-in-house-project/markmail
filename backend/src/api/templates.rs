use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use uuid::Uuid;
use validator::Validate;

use crate::{
    models::template::{CreateTemplateRequest, UpdateTemplateRequest},
    AppState,
};

pub async fn list_templates(State(_state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    // TODO: テンプレート一覧取得ロジックを実装
    Ok(Json(json!({
        "templates": [],
        "total": 0
    })))
}

pub async fn create_template(
    State(_state): State<AppState>,
    Json(payload): Json<CreateTemplateRequest>,
) -> Result<Json<Value>, StatusCode> {
    // バリデーション
    if payload.validate().is_err() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // TODO: テンプレート作成ロジックを実装
    Ok(Json(json!({
        "message": "テンプレートが作成されました",
        "template": {
            "name": payload.name,
            "description": payload.description
        }
    })))
}

pub async fn get_template(
    State(_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: テンプレート取得ロジックを実装
    Ok(Json(json!({
        "template": {
            "id": id,
            "name": "サンプルテンプレート"
        }
    })))
}

pub async fn update_template(
    State(_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateTemplateRequest>,
) -> Result<Json<Value>, StatusCode> {
    // バリデーション
    if payload.validate().is_err() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // TODO: テンプレート更新ロジックを実装
    Ok(Json(json!({
        "message": "テンプレートが更新されました",
        "template_id": id
    })))
}

pub async fn delete_template(
    State(_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: テンプレート削除ロジックを実装
    Ok(Json(json!({
        "message": "テンプレートが削除されました",
        "template_id": id
    })))
}

pub async fn preview_template(
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
    Json(_variables): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: テンプレートプレビューロジックを実装
    Ok(Json(json!({
        "html_preview": "<h1>プレビュー</h1>",
        "subject_preview": "メール件名プレビュー"
    })))
}

#[allow(dead_code)]
pub async fn render_template(
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
    Json(_variables): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: テンプレートレンダリングロジックを実装
    Ok(Json(json!({
        "html_rendered": "<h1>レンダリングされたHTML</h1>",
        "subject_rendered": "レンダリングされたメール件名"
    })))
}

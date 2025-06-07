use axum::{extract::State, http::StatusCode, response::Json, routing::post, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use validator::Validate;

use crate::{services::markdown_service::MarkdownService, AppState};

#[derive(Debug, Deserialize, Validate)]
pub struct RenderMarkdownRequest {
    #[validate(length(
        min = 1,
        max = 50000,
        message = "マークダウンコンテンツは1文字以上50000文字以下である必要があります"
    ))]
    pub markdown: String,
    pub variables: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct RenderMarkdownResponse {
    pub html: String,
    pub extracted_variables: Vec<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ValidateMarkdownRequest {
    #[validate(length(
        min = 1,
        max = 50000,
        message = "マークダウンコンテンツは1文字以上50000文字以下である必要があります"
    ))]
    pub markdown: String,
}

#[derive(Debug, Serialize)]
pub struct ValidateMarkdownResponse {
    pub valid: bool,
    pub errors: Vec<String>,
    pub extracted_variables: Vec<String>,
}

/// マークダウンをHTMLにレンダリング
pub async fn render_markdown(
    State(_state): State<AppState>,
    Json(payload): Json<RenderMarkdownRequest>,
) -> Result<Json<RenderMarkdownResponse>, (StatusCode, Json<Value>)> {
    // バリデーション
    if let Err(errors) = payload.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "バリデーションエラー",
                "details": errors
            })),
        ));
    }

    let markdown_service = MarkdownService::new();

    // 変数を抽出
    let extracted_variables = markdown_service.extract_variables(&payload.markdown);

    // HTMLに変換
    let html = if let Some(variables) = payload.variables {
        match markdown_service.render_with_variables(&payload.markdown, &variables) {
            Ok(html) => html,
            Err(e) => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": format!("マークダウンの変換に失敗しました: {}", e)
                    })),
                ));
            }
        }
    } else {
        markdown_service.render_to_html(&payload.markdown)
    };

    Ok(Json(RenderMarkdownResponse {
        html,
        extracted_variables,
    }))
}

/// マークダウンの構文を検証
pub async fn validate_markdown(
    State(_state): State<AppState>,
    Json(payload): Json<ValidateMarkdownRequest>,
) -> Result<Json<ValidateMarkdownResponse>, (StatusCode, Json<Value>)> {
    // バリデーション
    if let Err(errors) = payload.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "バリデーションエラー",
                "details": errors
            })),
        ));
    }

    let markdown_service = MarkdownService::new();

    // マークダウンの構文チェック
    let validation_errors = match markdown_service.validate_markdown(&payload.markdown) {
        Ok(errors) => errors,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": format!("マークダウンの検証中にエラーが発生しました: {}", e)
                })),
            ));
        }
    };

    // 変数を抽出
    let extracted_variables = markdown_service.extract_variables(&payload.markdown);

    let valid = validation_errors.is_empty();

    Ok(Json(ValidateMarkdownResponse {
        valid,
        errors: validation_errors,
        extracted_variables,
    }))
}

/// マークダウン関連のルーターを構築
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/render", post(render_markdown))
        .route("/validate", post(validate_markdown))
}

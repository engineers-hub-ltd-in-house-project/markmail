use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;
use validator::Validate;

use crate::{
    database::templates,
    middleware::auth::AuthUser,
    models::template::{
        CreateTemplateRequest, PreviewTemplateRequest, PreviewTemplateResponse,
        TemplateListResponse, TemplateResponse, UpdateTemplateRequest,
    },
    services::markdown_service::MarkdownService,
    AppState,
};

#[derive(Debug, Deserialize)]
pub struct ListTemplatesQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// テンプレート一覧取得
pub async fn list_templates(
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<ListTemplatesQuery>,
    State(state): State<AppState>,
) -> Result<Json<TemplateListResponse>, (StatusCode, Json<Value>)> {
    let limit = query.limit.unwrap_or(50).min(100);
    let offset = query.offset.unwrap_or(0);

    match templates::list_user_templates(&state.db, auth_user.user_id, Some(limit), Some(offset))
        .await
    {
        Ok(template_list) => {
            let total = templates::count_user_templates(&state.db, auth_user.user_id)
                .await
                .unwrap_or(0);

            let response = TemplateListResponse {
                templates: template_list.into_iter().map(|t| t.into()).collect(),
                total,
                limit,
                offset,
            };

            Ok(Json(response))
        }
        Err(e) => {
            tracing::error!("テンプレート一覧取得エラー: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "テンプレート一覧の取得に失敗しました"
                })),
            ))
        }
    }
}

/// テンプレート作成
pub async fn create_template(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Json(payload): Json<CreateTemplateRequest>,
) -> Result<Json<TemplateResponse>, (StatusCode, Json<Value>)> {
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

    match templates::create_template(&state.db, auth_user.user_id, &payload).await {
        Ok(template) => {
            tracing::info!("テンプレート作成成功: {}", template.id);
            Ok(Json(template.into()))
        }
        Err(e) => {
            tracing::error!("テンプレート作成エラー: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "テンプレートの作成に失敗しました"
                })),
            ))
        }
    }
}

/// テンプレート取得
pub async fn get_template(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<TemplateResponse>, (StatusCode, Json<Value>)> {
    match templates::find_template_by_id(&state.db, id, Some(auth_user.user_id)).await {
        Ok(Some(template)) => Ok(Json(template.into())),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "テンプレートが見つかりません"
            })),
        )),
        Err(e) => {
            tracing::error!("テンプレート取得エラー: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "テンプレートの取得に失敗しました"
                })),
            ))
        }
    }
}

/// テンプレート更新
pub async fn update_template(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateTemplateRequest>,
) -> Result<Json<TemplateResponse>, (StatusCode, Json<Value>)> {
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

    match templates::update_template(&state.db, id, auth_user.user_id, &payload).await {
        Ok(Some(template)) => {
            tracing::info!("テンプレート更新成功: {}", template.id);
            Ok(Json(template.into()))
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "テンプレートが見つかりません、または更新権限がありません"
            })),
        )),
        Err(e) => {
            tracing::error!("テンプレート更新エラー: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "テンプレートの更新に失敗しました"
                })),
            ))
        }
    }
}

/// テンプレート削除
pub async fn delete_template(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match templates::delete_template(&state.db, id, auth_user.user_id).await {
        Ok(true) => {
            tracing::info!("テンプレート削除成功: {}", id);
            Ok(Json(json!({
                "message": "テンプレートが削除されました",
                "template_id": id
            })))
        }
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "テンプレートが見つかりません、または削除権限がありません"
            })),
        )),
        Err(e) => {
            tracing::error!("テンプレート削除エラー: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "テンプレートの削除に失敗しました"
                })),
            ))
        }
    }
}

/// テンプレートプレビュー
pub async fn preview_template(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(variables): Json<PreviewTemplateRequest>,
) -> Result<Json<PreviewTemplateResponse>, (StatusCode, Json<Value>)> {
    // テンプレートを取得
    let template =
        match templates::find_template_by_id(&state.db, id, Some(auth_user.user_id)).await {
            Ok(Some(template)) => template,
            Ok(None) => {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(json!({
                        "error": "テンプレートが見つかりません"
                    })),
                ));
            }
            Err(e) => {
                tracing::error!("テンプレート取得エラー: {:?}", e);
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "テンプレートの取得に失敗しました"
                    })),
                ));
            }
        };

    // マークダウンサービスを使用してHTMLに変換
    let markdown_service = MarkdownService::new();

    let html = match markdown_service
        .render_with_variables(&template.markdown_content, &variables.variables)
    {
        Ok(html) => html,
        Err(e) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": format!("マークダウンの変換に失敗しました: {}", e)
                })),
            ));
        }
    };

    let subject =
        match markdown_service.render_subject(&template.subject_template, &variables.variables) {
            Ok(subject) => subject,
            Err(e) => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": format!("件名の変換に失敗しました: {}", e)
                    })),
                ));
            }
        };

    // HTMLコンテンツをキャッシュ用に保存
    if let Err(e) = templates::update_html_content(&state.db, template.id, &html).await {
        tracing::warn!("HTMLコンテンツのキャッシュ保存に失敗: {:?}", e);
    }

    Ok(Json(PreviewTemplateResponse { html, subject }))
}

/// テンプレート関連のルーターを構築
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_templates).post(create_template))
        .route(
            "/:id",
            get(get_template)
                .put(update_template)
                .delete(delete_template),
        )
        .route("/:id/preview", post(preview_template))
}

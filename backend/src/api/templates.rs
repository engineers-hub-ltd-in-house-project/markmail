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
        AnalyzeTemplateResponse, CreateTemplateRequest, PreviewTemplateRequest,
        PreviewTemplateResponse, TemplateListResponse, TemplateResponse, UpdateTemplateRequest,
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
    Json(mut payload): Json<CreateTemplateRequest>,
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

    // マークダウンサービスを使用して変数を抽出
    let markdown_service = MarkdownService::new();
    let content_vars = markdown_service.extract_variables(&payload.markdown_content);
    let subject_vars = markdown_service.extract_variables(&payload.subject_template);

    // 全ての変数をマージ
    let mut all_vars: Vec<String> = content_vars;
    for var in subject_vars {
        if !all_vars.contains(&var) {
            all_vars.push(var);
        }
    }

    // デフォルト値のマップを作成
    let default_values = get_default_variable_values();

    // variablesフィールドを構築（既存の値とマージ）
    let mut variables = if let Some(existing_vars) = &payload.variables {
        existing_vars.clone()
    } else {
        json!({})
    };

    if let Value::Object(ref mut map) = variables {
        // 使用されている全ての変数に対してデフォルト値を設定（既存の値がない場合）
        for var in all_vars {
            if !map.contains_key(&var) {
                if let Some(default_value) = default_values.get(&var) {
                    map.insert(var, default_value.clone());
                } else {
                    // カスタム変数のデフォルト値
                    map.insert(var.clone(), json!(format!("[{}]", var)));
                }
            }
        }
    }

    payload.variables = Some(variables);

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

/// 標準変数のデフォルト値を定義
fn get_default_variable_values() -> std::collections::HashMap<String, Value> {
    let mut defaults = std::collections::HashMap::new();

    // 標準変数のデフォルト値
    defaults.insert("name".to_string(), json!("お客様"));
    defaults.insert("first_name".to_string(), json!("お客様"));
    defaults.insert("email".to_string(), json!("example@example.com"));
    defaults.insert("company_name".to_string(), json!("貴社"));
    defaults.insert("product_name".to_string(), json!("製品"));
    defaults.insert("service_name".to_string(), json!("サービス"));
    defaults.insert("sender_name".to_string(), json!("送信者"));
    defaults.insert(
        "unsubscribe_url".to_string(),
        json!("https://example.com/unsubscribe"),
    );

    defaults
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
    Json(mut payload): Json<UpdateTemplateRequest>,
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

    // 既存のテンプレートを取得
    let existing_template =
        match templates::find_template_by_id(&state.db, id, Some(auth_user.user_id)).await {
            Ok(Some(template)) => template,
            Ok(None) => {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(json!({
                        "error": "テンプレートが見つかりません"
                    })),
                ))
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

    // 更新されるコンテンツから変数を抽出
    let markdown_service = MarkdownService::new();
    let markdown_content = payload
        .markdown_content
        .as_ref()
        .unwrap_or(&existing_template.markdown_content);
    let subject_template = payload
        .subject_template
        .as_ref()
        .unwrap_or(&existing_template.subject_template);

    let content_vars = markdown_service.extract_variables(markdown_content);
    let subject_vars = markdown_service.extract_variables(subject_template);

    // 全ての変数をマージ
    let mut all_vars: Vec<String> = content_vars;
    for var in subject_vars {
        if !all_vars.contains(&var) {
            all_vars.push(var);
        }
    }

    // デフォルト値のマップを作成
    let default_values = get_default_variable_values();

    // 既存のvariablesと新しいvariablesをマージ
    let mut variables = if let Some(new_vars) = &payload.variables {
        new_vars.clone()
    } else {
        existing_template.variables.clone()
    };

    if let Value::Object(ref mut map) = variables {
        // 使用されている全ての変数に対してデフォルト値を設定（既存の値がない場合）
        for var in all_vars {
            if !map.contains_key(&var) {
                if let Some(default_value) = default_values.get(&var) {
                    map.insert(var, default_value.clone());
                } else {
                    // カスタム変数のデフォルト値
                    map.insert(var.clone(), json!(format!("[{}]", var)));
                }
            }
        }
    }

    payload.variables = Some(variables);

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

/// テンプレート変数分析
pub async fn analyze_template_variables(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AnalyzeTemplateResponse>, (StatusCode, Json<Value>)> {
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
                ))
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

    // マークダウンサービスを初期化
    let markdown_service = MarkdownService::new();

    // テンプレートから変数を抽出
    let used_variables = markdown_service.extract_variables(&template.markdown_content);
    let subject_variables = markdown_service.extract_variables(&template.subject_template);

    // 全ての使用変数をマージ（重複除去）
    let mut all_variables: Vec<String> = used_variables;
    for var in subject_variables {
        if !all_variables.contains(&var) {
            all_variables.push(var);
        }
    }

    // システムで自動的に提供される標準変数
    let standard_variables = vec![
        "name".to_string(),
        "first_name".to_string(),
        "email".to_string(),
        "unsubscribe_url".to_string(),
    ];

    // カスタム変数（標準変数以外）を特定
    let custom_variables: Vec<String> = all_variables
        .iter()
        .filter(|var| !standard_variables.contains(var))
        .cloned()
        .collect();

    // テンプレートに定義されている変数
    let defined_variables: Vec<String> = if let Value::Object(vars) = &template.variables {
        vars.keys().cloned().collect()
    } else {
        vec![]
    };

    // 不足している変数（カスタム変数のうち定義されていないもの）
    let missing_variables: Vec<String> = custom_variables
        .iter()
        .filter(|var| !defined_variables.contains(var))
        .cloned()
        .collect();

    Ok(Json(AnalyzeTemplateResponse {
        used_variables: all_variables,
        standard_variables,
        custom_variables,
        defined_variables,
        missing_variables,
    }))
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
        .route("/:id/analyze", get(analyze_template_variables))
}

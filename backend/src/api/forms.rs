use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{
    database::{forms, subscribers},
    middleware::auth::AuthUser,
    models::form::{
        CreateFormRequest, CreateFormSubmissionRequest, Form, FormSubmission, UpdateFormRequest,
    },
    models::sequence::TriggerType,
    models::subscriber::{CreateSubscriberRequest, SubscriberStatus},
    services::sequence_service::SequenceService,
    AppState,
};

#[derive(Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    20
}

/// フォームデータからメールアドレスを抽出
fn extract_email_from_form_data(form_data: &Value, form_fields: &Value) -> Option<String> {
    if let (Value::Object(data), Value::Array(fields)) = (form_data, form_fields) {
        for field in fields {
            if let Value::Object(field_obj) = field {
                if let (Some(Value::String(field_type)), Some(Value::String(field_name))) =
                    (field_obj.get("field_type"), field_obj.get("name"))
                {
                    if field_type == "email" {
                        if let Some(Value::String(email)) = data.get(field_name) {
                            return Some(email.clone());
                        }
                    }
                }
            }
        }
    }
    None
}

/// フォームデータから名前を抽出
fn extract_name_from_form_data(form_data: &Value, form_fields: &Value) -> Option<String> {
    if let (Value::Object(data), Value::Array(fields)) = (form_data, form_fields) {
        for field in fields {
            if let Value::Object(field_obj) = field {
                if let (Some(Value::String(field_type)), Some(Value::String(field_name))) =
                    (field_obj.get("field_type"), field_obj.get("name"))
                {
                    if field_type == "text" && (field_name == "name" || field_name.contains("名前"))
                    {
                        if let Some(Value::String(name)) = data.get(field_name) {
                            return Some(name.clone());
                        }
                    }
                }
            }
        }
    }
    None
}

pub async fn create_form(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Json(payload): Json<CreateFormRequest>,
) -> Result<(StatusCode, Json<Form>), (StatusCode, Json<Value>)> {
    match forms::create_form(&state.db, auth_user.user_id, payload).await {
        Ok(form) => Ok((StatusCode::CREATED, Json(form))),
        Err(e) => {
            tracing::error!("フォーム作成エラー: {:?}", e);
            eprintln!("Form creation error: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": format!("フォームの作成に失敗しました: {:?}", e)
                })),
            ))
        }
    }
}

pub async fn get_forms(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
) -> Result<Json<Vec<Form>>, (StatusCode, Json<Value>)> {
    match forms::get_forms_by_user(&state.db, auth_user.user_id).await {
        Ok(forms) => Ok(Json(forms)),
        Err(e) => {
            tracing::error!("フォーム一覧取得エラー: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "フォーム一覧の取得に失敗しました"
                })),
            ))
        }
    }
}

pub async fn get_form(
    Extension(auth_user): Extension<AuthUser>,
    Path(form_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<Form>, (StatusCode, Json<Value>)> {
    match forms::get_form_by_id(&state.db, form_id).await {
        Ok(Some(form)) => {
            if form.user_id == auth_user.user_id {
                Ok(Json(form))
            } else {
                Err((
                    StatusCode::FORBIDDEN,
                    Json(json!({
                        "error": "このフォームへのアクセス権限がありません"
                    })),
                ))
            }
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "フォームが見つかりません"
            })),
        )),
        Err(e) => {
            tracing::error!("フォーム取得エラー: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "フォームの取得に失敗しました"
                })),
            ))
        }
    }
}

pub async fn update_form(
    Extension(auth_user): Extension<AuthUser>,
    Path(form_id): Path<Uuid>,
    State(state): State<AppState>,
    Json(payload): Json<UpdateFormRequest>,
) -> Result<Json<Form>, (StatusCode, Json<Value>)> {
    // Check ownership
    match forms::get_form_by_id(&state.db, form_id).await {
        Ok(Some(form)) => {
            if form.user_id == auth_user.user_id {
                match forms::update_form(&state.db, form_id, payload).await {
                    Ok(updated_form) => Ok(Json(updated_form)),
                    Err(e) => {
                        tracing::error!("フォーム更新エラー: {:?}", e);
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(json!({
                                "error": "フォームの更新に失敗しました"
                            })),
                        ))
                    }
                }
            } else {
                Err((
                    StatusCode::FORBIDDEN,
                    Json(json!({
                        "error": "このフォームへのアクセス権限がありません"
                    })),
                ))
            }
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "フォームが見つかりません"
            })),
        )),
        Err(e) => {
            tracing::error!("フォーム取得エラー: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "フォームの取得に失敗しました"
                })),
            ))
        }
    }
}

pub async fn delete_form(
    Extension(auth_user): Extension<AuthUser>,
    Path(form_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<StatusCode, (StatusCode, Json<Value>)> {
    // Check ownership
    match forms::get_form_by_id(&state.db, form_id).await {
        Ok(Some(form)) => {
            if form.user_id == auth_user.user_id {
                match forms::delete_form(&state.db, form_id).await {
                    Ok(_) => Ok(StatusCode::NO_CONTENT),
                    Err(e) => {
                        tracing::error!("フォーム削除エラー: {:?}", e);
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(json!({
                                "error": "フォームの削除に失敗しました"
                            })),
                        ))
                    }
                }
            } else {
                Err((
                    StatusCode::FORBIDDEN,
                    Json(json!({
                        "error": "このフォームへのアクセス権限がありません"
                    })),
                ))
            }
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "フォームが見つかりません"
            })),
        )),
        Err(e) => {
            tracing::error!("フォーム取得エラー: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "フォームの取得に失敗しました"
                })),
            ))
        }
    }
}

pub async fn get_form_submissions(
    Extension(auth_user): Extension<AuthUser>,
    Path(form_id): Path<Uuid>,
    Query(params): Query<PaginationParams>,
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Check form ownership
    match forms::get_form_by_id(&state.db, form_id).await {
        Ok(Some(form)) => {
            if form.user_id == auth_user.user_id {
                match forms::get_form_submissions(&state.db, form_id, params.limit, params.offset)
                    .await
                {
                    Ok(submissions) => {
                        match forms::count_form_submissions(&state.db, form_id).await {
                            Ok(total) => Ok(Json(json!({
                                "submissions": submissions,
                                "total": total,
                                "limit": params.limit,
                                "offset": params.offset
                            }))),
                            Err(e) => {
                                tracing::error!("フォーム送信数カウントエラー: {:?}", e);
                                Err((
                                    StatusCode::INTERNAL_SERVER_ERROR,
                                    Json(json!({
                                        "error": "フォーム送信数の取得に失敗しました"
                                    })),
                                ))
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("フォーム送信データ取得エラー: {:?}", e);
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(json!({
                                "error": "フォーム送信データの取得に失敗しました"
                            })),
                        ))
                    }
                }
            } else {
                Err((
                    StatusCode::FORBIDDEN,
                    Json(json!({
                        "error": "このフォームへのアクセス権限がありません"
                    })),
                ))
            }
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "フォームが見つかりません"
            })),
        )),
        Err(e) => {
            tracing::error!("フォーム取得エラー: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "フォームの取得に失敗しました"
                })),
            ))
        }
    }
}

// Public endpoint for form submission (no auth required)
pub async fn submit_form(
    Path(form_id): Path<Uuid>,
    State(state): State<AppState>,
    Json(request): Json<CreateFormSubmissionRequest>,
) -> Result<(StatusCode, Json<FormSubmission>), (StatusCode, Json<Value>)> {
    // Check if form exists and is active
    match forms::get_form_by_id(&state.db, form_id).await {
        Ok(Some(form)) => {
            if form.status == "published" {
                // TODO: Implement form validation based on form fields
                // TODO: Extract IP address and user agent from request headers

                // フォームデータからメールアドレスを抽出
                tracing::info!("Form data: {:?}", request.data);
                tracing::info!("Form fields: {:?}", form.form_fields);
                let email = extract_email_from_form_data(&request.data, &form.form_fields);
                tracing::info!("Extracted email: {:?}", email);

                // メールアドレスがある場合は購読者を作成またはリンク
                let subscriber_id = if let Some(email) = email {
                    match subscribers::find_subscriber_by_email(&state.db, &email, form.user_id)
                        .await
                    {
                        Ok(Some(subscriber)) => Some(subscriber.id),
                        Ok(None) => {
                            // 新規購読者を作成
                            let create_req = CreateSubscriberRequest {
                                email,
                                name: extract_name_from_form_data(&request.data, &form.form_fields),
                                status: Some(SubscriberStatus::Active),
                                tags: Some(vec![format!("form:{}", form.slug)]),
                                custom_fields: Some(request.data.clone()),
                            };
                            match subscribers::create_subscriber(
                                &state.db,
                                form.user_id,
                                &create_req,
                            )
                            .await
                            {
                                Ok(subscriber) => {
                                    tracing::info!(
                                        "Created new subscriber {} from form submission",
                                        subscriber.id
                                    );
                                    Some(subscriber.id)
                                }
                                Err(e) => {
                                    tracing::error!("Failed to create subscriber from form: {}", e);
                                    None
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to find subscriber by email: {}", e);
                            None
                        }
                    }
                } else {
                    tracing::warn!("No email field found in form submission");
                    None
                };

                match forms::create_form_submission(
                    &state.db,
                    form_id,
                    request.data,
                    subscriber_id,
                    None, // TODO: IP address
                    None, // TODO: User agent
                    None, // TODO: Referrer
                )
                .await
                {
                    Ok(submission) => {
                        // フォーム送信時のシーケンストリガー
                        if let Some(subscriber_id) = submission.subscriber_id {
                            let sequence_service = SequenceService::new();
                            if let Err(e) = sequence_service
                                .process_trigger_enrollment(
                                    &state.db,
                                    form.user_id,
                                    TriggerType::FormSubmission,
                                    subscriber_id,
                                    Some(json!({
                                        "form_id": form_id.to_string(),
                                        "submission_id": submission.id.to_string()
                                    })),
                                )
                                .await
                            {
                                tracing::error!("シーケンスエンロールメントエラー: {}", e);
                                // エラーが発生してもレスポンスは返す（フォーム送信は成功しているため）
                            }
                        }

                        Ok((StatusCode::CREATED, Json(submission)))
                    }
                    Err(e) => {
                        tracing::error!("フォーム送信エラー: {:?}", e);
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(json!({
                                "error": "フォームの送信に失敗しました"
                            })),
                        ))
                    }
                }
            } else {
                Err((
                    StatusCode::FORBIDDEN,
                    Json(json!({
                        "error": "このフォームは公開されていません"
                    })),
                ))
            }
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "フォームが見つかりません"
            })),
        )),
        Err(e) => {
            tracing::error!("フォーム取得エラー: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "フォームの取得に失敗しました"
                })),
            ))
        }
    }
}

// Public endpoint to get form for rendering (no auth required)
pub async fn get_public_form(
    Path(form_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<Form>, (StatusCode, Json<Value>)> {
    match forms::get_form_by_id(&state.db, form_id).await {
        Ok(Some(form)) => {
            if form.status == "published" {
                Ok(Json(form))
            } else {
                Err((
                    StatusCode::FORBIDDEN,
                    Json(json!({
                        "error": "このフォームは公開されていません"
                    })),
                ))
            }
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "フォームが見つかりません"
            })),
        )),
        Err(e) => {
            tracing::error!("フォーム取得エラー: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "フォームの取得に失敗しました"
                })),
            ))
        }
    }
}

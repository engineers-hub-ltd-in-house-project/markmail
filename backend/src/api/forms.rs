use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{
    database::forms,
    middleware::auth::AuthUser,
    models::form::{
        CreateFormRequest, CreateFormSubmissionRequest, Form, FormSubmission, UpdateFormRequest,
    },
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

                match forms::create_form_submission(
                    &state.db,
                    form_id,
                    request.data,
                    None, // TODO: Link to subscriber if applicable
                    None, // TODO: IP address
                    None, // TODO: User agent
                    None, // TODO: Referrer
                )
                .await
                {
                    Ok(submission) => Ok((StatusCode::CREATED, Json(submission))),
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

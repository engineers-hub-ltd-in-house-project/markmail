use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{
    database::sequences as db,
    middleware::auth::AuthUser,
    models::sequence::{
        CreateSequenceRequest, CreateSequenceStepRequest, UpdateSequenceRequest,
        UpdateSequenceStepRequest,
    },
    AppState,
};

pub async fn create_sequence(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Json(request): Json<CreateSequenceRequest>,
) -> Result<(StatusCode, Json<crate::models::sequence::Sequence>), (StatusCode, Json<Value>)> {
    match db::create_sequence(&state.db, user.user_id, request).await {
        Ok(sequence) => Ok((StatusCode::CREATED, Json(sequence))),
        Err(e) => {
            tracing::error!("シーケンス作成エラー: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "シーケンスの作成に失敗しました"
                })),
            ))
        }
    }
}

pub async fn get_sequences(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
) -> Result<Json<Vec<crate::models::sequence::Sequence>>, (StatusCode, Json<Value>)> {
    match db::get_sequences_by_user(&state.db, user.user_id).await {
        Ok(sequences) => Ok(Json(sequences)),
        Err(e) => {
            tracing::error!("シーケンス一覧取得エラー: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "シーケンス一覧の取得に失敗しました"
                })),
            ))
        }
    }
}

pub async fn get_sequence(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(sequence_id): Path<Uuid>,
) -> Result<Json<crate::models::sequence::Sequence>, (StatusCode, Json<Value>)> {
    match db::get_sequence_by_id(&state.db, sequence_id).await {
        Ok(Some(sequence)) => {
            if sequence.user_id == user.user_id {
                Ok(Json(sequence))
            } else {
                Err((
                    StatusCode::FORBIDDEN,
                    Json(json!({
                        "error": "このシーケンスへのアクセス権限がありません"
                    })),
                ))
            }
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "シーケンスが見つかりません"
            })),
        )),
        Err(e) => {
            tracing::error!("シーケンス取得エラー: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "シーケンスの取得に失敗しました"
                })),
            ))
        }
    }
}

pub async fn update_sequence(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(sequence_id): Path<Uuid>,
    Json(request): Json<UpdateSequenceRequest>,
) -> Result<Json<crate::models::sequence::Sequence>, (StatusCode, Json<Value>)> {
    // Check ownership
    match db::get_sequence_by_id(&state.db, sequence_id).await {
        Ok(Some(sequence)) => {
            if sequence.user_id == user.user_id {
                match db::update_sequence(&state.db, sequence_id, request).await {
                    Ok(updated_sequence) => Ok(Json(updated_sequence)),
                    Err(e) => {
                        tracing::error!("シーケンス更新エラー: {:?}", e);
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(json!({
                                "error": "シーケンスの更新に失敗しました"
                            })),
                        ))
                    }
                }
            } else {
                Err((
                    StatusCode::FORBIDDEN,
                    Json(json!({
                        "error": "このシーケンスへのアクセス権限がありません"
                    })),
                ))
            }
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "シーケンスが見つかりません"
            })),
        )),
        Err(e) => {
            tracing::error!("シーケンス取得エラー: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "シーケンスの取得に失敗しました"
                })),
            ))
        }
    }
}

pub async fn delete_sequence(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(sequence_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<Value>)> {
    // Check ownership
    match db::get_sequence_by_id(&state.db, sequence_id).await {
        Ok(Some(sequence)) => {
            if sequence.user_id == user.user_id {
                match db::delete_sequence(&state.db, sequence_id).await {
                    Ok(_) => Ok(StatusCode::NO_CONTENT),
                    Err(e) => {
                        tracing::error!("シーケンス削除エラー: {:?}", e);
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(json!({
                                "error": "シーケンスの削除に失敗しました"
                            })),
                        ))
                    }
                }
            } else {
                Err((
                    StatusCode::FORBIDDEN,
                    Json(json!({
                        "error": "このシーケンスへのアクセス権限がありません"
                    })),
                ))
            }
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "シーケンスが見つかりません"
            })),
        )),
        Err(e) => {
            tracing::error!("シーケンス取得エラー: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "シーケンスの取得に失敗しました"
                })),
            ))
        }
    }
}

pub async fn get_sequence_with_steps(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(sequence_id): Path<Uuid>,
) -> Result<Json<crate::models::sequence::SequenceWithSteps>, (StatusCode, Json<Value>)> {
    match db::get_sequence_with_steps(&state.db, sequence_id).await {
        Ok(Some(sequence_data)) => {
            if sequence_data.sequence.user_id == user.user_id {
                Ok(Json(sequence_data))
            } else {
                Err((
                    StatusCode::FORBIDDEN,
                    Json(json!({
                        "error": "このシーケンスへのアクセス権限がありません"
                    })),
                ))
            }
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "シーケンスが見つかりません"
            })),
        )),
        Err(e) => {
            tracing::error!("シーケンス取得エラー: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "シーケンスの取得に失敗しました"
                })),
            ))
        }
    }
}

pub async fn create_sequence_step(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(sequence_id): Path<Uuid>,
    Json(request): Json<CreateSequenceStepRequest>,
) -> Result<(StatusCode, Json<crate::models::sequence::SequenceStep>), (StatusCode, Json<Value>)> {
    // Check sequence ownership
    match db::get_sequence_by_id(&state.db, sequence_id).await {
        Ok(Some(sequence)) => {
            if sequence.user_id == user.user_id {
                match db::create_sequence_step(&state.db, sequence_id, request).await {
                    Ok(step) => Ok((StatusCode::CREATED, Json(step))),
                    Err(e) => {
                        tracing::error!("シーケンスステップ作成エラー: {:?}", e);
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(json!({
                                "error": "シーケンスステップの作成に失敗しました"
                            })),
                        ))
                    }
                }
            } else {
                Err((
                    StatusCode::FORBIDDEN,
                    Json(json!({
                        "error": "このシーケンスへのアクセス権限がありません"
                    })),
                ))
            }
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "シーケンスが見つかりません"
            })),
        )),
        Err(e) => {
            tracing::error!("シーケンス取得エラー: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "シーケンスの取得に失敗しました"
                })),
            ))
        }
    }
}

pub async fn update_sequence_step(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path((sequence_id, step_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<UpdateSequenceStepRequest>,
) -> Result<Json<crate::models::sequence::SequenceStep>, (StatusCode, Json<Value>)> {
    // Check sequence ownership
    match db::get_sequence_by_id(&state.db, sequence_id).await {
        Ok(Some(sequence)) => {
            if sequence.user_id == user.user_id {
                match db::update_sequence_step(&state.db, step_id, request).await {
                    Ok(step) => Ok(Json(step)),
                    Err(e) => {
                        tracing::error!("シーケンスステップ更新エラー: {:?}", e);
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(json!({
                                "error": "シーケンスステップの更新に失敗しました"
                            })),
                        ))
                    }
                }
            } else {
                Err((
                    StatusCode::FORBIDDEN,
                    Json(json!({
                        "error": "このシーケンスへのアクセス権限がありません"
                    })),
                ))
            }
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "シーケンスが見つかりません"
            })),
        )),
        Err(e) => {
            tracing::error!("シーケンス取得エラー: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "シーケンスの取得に失敗しました"
                })),
            ))
        }
    }
}

pub async fn delete_sequence_step(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path((sequence_id, step_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, Json<Value>)> {
    // Check sequence ownership
    match db::get_sequence_by_id(&state.db, sequence_id).await {
        Ok(Some(sequence)) => {
            if sequence.user_id == user.user_id {
                match db::delete_sequence_step(&state.db, step_id).await {
                    Ok(_) => Ok(StatusCode::NO_CONTENT),
                    Err(e) => {
                        tracing::error!("シーケンスステップ削除エラー: {:?}", e);
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(json!({
                                "error": "シーケンスステップの削除に失敗しました"
                            })),
                        ))
                    }
                }
            } else {
                Err((
                    StatusCode::FORBIDDEN,
                    Json(json!({
                        "error": "このシーケンスへのアクセス権限がありません"
                    })),
                ))
            }
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "シーケンスが見つかりません"
            })),
        )),
        Err(e) => {
            tracing::error!("シーケンス取得エラー: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "シーケンスの取得に失敗しました"
                })),
            ))
        }
    }
}

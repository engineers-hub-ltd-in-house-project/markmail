use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    database::sequences::*,
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
) -> Result<impl IntoResponse, StatusCode> {
    let sequence = create_sequence(&state.db, user.user_id, request)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::CREATED, Json(sequence)))
}

pub async fn get_sequences(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
) -> Result<impl IntoResponse, StatusCode> {
    let sequences = get_sequences_by_user(&state.db, user.user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(sequences))
}

pub async fn get_sequence(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(sequence_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let sequence = get_sequence_by_id(&state.db, sequence_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match sequence {
        Some(sequence) if sequence.user_id == user.user_id => Ok(Json(sequence)),
        Some(_) => Err(StatusCode::FORBIDDEN),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn update_sequence(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(sequence_id): Path<Uuid>,
    Json(request): Json<UpdateSequenceRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    // Check ownership
    let existing_sequence = get_sequence_by_id(&state.db, sequence_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match existing_sequence {
        Some(sequence) if sequence.user_id == user.user_id => {
            let updated_sequence = update_sequence(&state.db, sequence_id, request)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            Ok(Json(updated_sequence))
        }
        Some(_) => Err(StatusCode::FORBIDDEN),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn delete_sequence(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(sequence_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    // Check ownership
    let existing_sequence = get_sequence_by_id(&state.db, sequence_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match existing_sequence {
        Some(sequence) if sequence.user_id == user.user_id => {
            delete_sequence(&state.db, sequence_id)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            Ok(StatusCode::NO_CONTENT)
        }
        Some(_) => Err(StatusCode::FORBIDDEN),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn get_sequence_with_steps(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(sequence_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let sequence_with_steps = get_sequence_with_steps(&state.db, sequence_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match sequence_with_steps {
        Some(sequence_data) if sequence_data.sequence.user_id == user.user_id => {
            Ok(Json(sequence_data))
        }
        Some(_) => Err(StatusCode::FORBIDDEN),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn create_sequence_step(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(sequence_id): Path<Uuid>,
    Json(request): Json<CreateSequenceStepRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    // Check sequence ownership
    let sequence = get_sequence_by_id(&state.db, sequence_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match sequence {
        Some(sequence) if sequence.user_id == user.user_id => {
            let step = create_sequence_step(&state.db, sequence_id, request)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            Ok((StatusCode::CREATED, Json(step)))
        }
        Some(_) => Err(StatusCode::FORBIDDEN),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn update_sequence_step(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path((sequence_id, step_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<UpdateSequenceStepRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    // Check sequence ownership
    let sequence = get_sequence_by_id(&state.db, sequence_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match sequence {
        Some(sequence) if sequence.user_id == user.user_id => {
            let step = update_sequence_step(&state.db, step_id, request)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            Ok(Json(step))
        }
        Some(_) => Err(StatusCode::FORBIDDEN),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn delete_sequence_step(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path((sequence_id, step_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, StatusCode> {
    // Check sequence ownership
    let sequence = get_sequence_by_id(&state.db, sequence_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match sequence {
        Some(sequence) if sequence.user_id == user.user_id => {
            delete_sequence_step(&state.db, step_id)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            Ok(StatusCode::NO_CONTENT)
        }
        Some(_) => Err(StatusCode::FORBIDDEN),
        None => Err(StatusCode::NOT_FOUND),
    }
}
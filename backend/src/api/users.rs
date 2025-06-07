use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde_json::{json, Value};
use validator::Validate;

use crate::{
    database::users,
    middleware::auth::AuthUser,
    models::user::{UpdateProfileRequest, UserResponse},
    AppState,
};

/// プロフィール取得エンドポイント
pub async fn get_profile(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
) -> Result<Json<UserResponse>, (StatusCode, Json<Value>)> {
    match users::find_user_by_id(&state.db, auth_user.user_id).await {
        Ok(Some(user)) => Ok(Json(user.into())),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "ユーザーが見つかりません"
            })),
        )),
        Err(e) => {
            tracing::error!("ユーザー取得エラー: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "プロフィールの取得に失敗しました"
                })),
            ))
        }
    }
}

/// プロフィール更新エンドポイント
pub async fn update_profile(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<Json<UserResponse>, (StatusCode, Json<Value>)> {
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

    match users::update_user_profile(
        &state.db,
        auth_user.user_id,
        payload.name.as_deref(),
        payload.avatar_url.as_deref(),
    )
    .await
    {
        Ok(user) => Ok(Json(user.into())),
        Err(e) => {
            tracing::error!("プロフィール更新エラー: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "プロフィールの更新に失敗しました"
                })),
            ))
        }
    }
}

/// ユーザー関連のルーターを構築
pub fn router() -> Router<AppState> {
    Router::new().route("/profile", get(get_profile).put(update_profile))
}

use axum::{extract::State, http::StatusCode, response::Json};
use serde_json::{json, Value};
use validator::Validate;

use crate::{
    models::user::{AuthResponse, LoginRequest, RefreshTokenRequest, RegisterRequest},
    services::auth_service::{AuthError, AuthService},
    AppState,
};

/// ユーザー登録エンドポイント
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<Value>)> {
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

    let auth_service = AuthService::new(state.db.clone());

    match auth_service.register(payload).await {
        Ok(response) => Ok(Json(response)),
        Err(AuthError::EmailAlreadyExists) => Err((
            StatusCode::CONFLICT,
            Json(json!({
                "error": "メールアドレスは既に使用されています"
            })),
        )),
        Err(e) => {
            tracing::error!("ユーザー登録エラー: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "ユーザー登録に失敗しました"
                })),
            ))
        }
    }
}

/// ログインエンドポイント
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<Value>)> {
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

    let auth_service = AuthService::new(state.db.clone());

    match auth_service.login(payload).await {
        Ok(response) => Ok(Json(response)),
        Err(AuthError::InvalidCredentials) => Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "メールアドレスまたはパスワードが正しくありません"
            })),
        )),
        Err(e) => {
            tracing::error!("ログインエラー: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "ログインに失敗しました"
                })),
            ))
        }
    }
}

/// リフレッシュトークンエンドポイント
pub async fn refresh_token(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<Value>)> {
    let auth_service = AuthService::new(state.db.clone());

    match auth_service.refresh_token(&payload.refresh_token).await {
        Ok(response) => Ok(Json(response)),
        Err(AuthError::InvalidRefreshToken) => Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "無効なリフレッシュトークンです"
            })),
        )),
        Err(e) => {
            tracing::error!("トークン更新エラー: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "トークンの更新に失敗しました"
                })),
            ))
        }
    }
}

use axum::{extract::State, http::StatusCode, response::Json};
use serde_json::{json, Value};
use validator::Validate;

use crate::{
    models::user::{LoginRequest, RefreshTokenRequest, RegisterRequest},
    AppState,
};

pub async fn register(
    State(_state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<Value>, StatusCode> {
    // バリデーション
    if let Err(_) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // TODO: ユーザー登録ロジックを実装
    // - メールアドレスの重複チェック
    // - パスワードのハッシュ化
    // - データベースへの保存
    // - JWTトークンの生成

    Ok(Json(json!({
        "message": "ユーザー登録が完了しました",
        "user": {
            "email": payload.email,
            "name": payload.name
        }
    })))
}

pub async fn login(
    State(_state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<Value>, StatusCode> {
    // バリデーション
    if let Err(_) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // TODO: ログインロジックを実装
    // - ユーザーの存在確認
    // - パスワードの検証
    // - JWTトークンの生成

    Ok(Json(json!({
        "message": "ログインが完了しました",
        "token": "dummy_jwt_token",
        "user": {
            "email": payload.email
        }
    })))
}

pub async fn refresh_token(
    State(_state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: リフレッシュトークンロジックを実装
    // - リフレッシュトークンの検証
    // - 新しいJWTトークンの生成

    Ok(Json(json!({
        "message": "トークンが更新されました",
        "token": "new_dummy_jwt_token"
    })))
}

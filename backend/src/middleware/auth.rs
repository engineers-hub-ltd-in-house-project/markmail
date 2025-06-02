// 認証ミドルウェア
// TODO: JWT検証、認証が必要なエンドポイントの保護

use axum::{
    extract::Request,
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use serde_json::json;
use uuid::Uuid;

use crate::utils::jwt::verify_token;

/// 認証が必要なエンドポイント用のミドルウェア
pub async fn auth_middleware(
    mut request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    // Authorizationヘッダーを取得
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "error": "認証ヘッダーがありません"
                })),
            )
        })?;

    // Bearer トークンを抽出
    let token = auth_header.strip_prefix("Bearer ").ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "無効な認証ヘッダー形式"
            })),
        )
    })?;

    // トークンを検証
    let token_data = verify_token(token).map_err(|e| {
        tracing::debug!("トークン検証エラー: {:?}", e);
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "無効なトークン"
            })),
        )
    })?;

    // ユーザーIDをリクエストの拡張データに追加
    request.extensions_mut().insert(AuthUser {
        user_id: Uuid::parse_str(&token_data.claims.sub).unwrap(),
        email: token_data.claims.email,
        name: token_data.claims.name,
    });

    Ok(next.run(request).await)
}

/// 認証されたユーザー情報
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub email: String,
    pub name: String,
}

/// オプショナル認証ミドルウェア（認証は任意）
#[allow(dead_code)]
pub async fn optional_auth_middleware(mut request: Request, next: Next) -> Response {
    // Authorizationヘッダーを取得
    if let Some(auth_header) = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
    {
        // Bearer トークンを抽出
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            // トークンを検証（エラーは無視）
            if let Ok(token_data) = verify_token(token) {
                if let Ok(user_id) = Uuid::parse_str(&token_data.claims.sub) {
                    request.extensions_mut().insert(AuthUser {
                        user_id,
                        email: token_data.claims.email,
                        name: token_data.claims.name,
                    });
                }
            }
        }
    }

    next.run(request).await
}

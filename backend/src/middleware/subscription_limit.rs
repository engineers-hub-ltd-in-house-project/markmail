use crate::middleware::auth::AuthUser;
use crate::services::subscription_service;
use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use sqlx::PgPool;

/// リソース制限をチェックするミドルウェア
pub async fn check_resource_limit(
    resource_type: &'static str,
) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>>
       + Clone {
    move |request: Request, next: Next| {
        let resource = resource_type;
        Box::pin(async move {
            // リクエストパスからリソースタイプを判定
            let _path = request.uri().path();

            // 作成・更新系のHTTPメソッドのみチェック
            let method = request.method();
            if method != axum::http::Method::POST && method != axum::http::Method::PUT {
                return next.run(request).await;
            }

            // 認証ユーザーを取得
            let user = match request.extensions().get::<AuthUser>() {
                Some(user) => user.clone(),
                None => {
                    return (
                        StatusCode::UNAUTHORIZED,
                        Json(json!({
                            "error": "認証が必要です"
                        })),
                    )
                        .into_response();
                }
            };

            // データベースプールを取得
            let pool = match request.extensions().get::<PgPool>() {
                Some(pool) => pool.clone(),
                None => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({
                            "error": "データベース接続エラー"
                        })),
                    )
                        .into_response();
                }
            };

            // リソース制限をチェック
            match subscription_service::check_resource_limit(&pool, user.user_id, resource).await {
                Ok(true) => {
                    // 制限内なので続行
                    next.run(request).await
                }
                Ok(false) => {
                    // 制限を超えている
                    (
                        StatusCode::PAYMENT_REQUIRED,
                        Json(json!({
                            "error": format!("{}の作成上限に達しました。プランをアップグレードしてください。", get_resource_display_name(resource))
                        })),
                    )
                        .into_response()
                }
                Err(_) => {
                    // エラーが発生した場合は通過させる（サービスを止めない）
                    next.run(request).await
                }
            }
        })
    }
}

/// 機能アクセスをチェックするミドルウェア
pub async fn check_feature_access(
    feature: &'static str,
) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>>
       + Clone {
    move |request: Request, next: Next| {
        let feature_name = feature;
        Box::pin(async move {
            // 認証ユーザーを取得
            let user = match request.extensions().get::<AuthUser>() {
                Some(user) => user.clone(),
                None => {
                    return (
                        StatusCode::UNAUTHORIZED,
                        Json(json!({
                            "error": "認証が必要です"
                        })),
                    )
                        .into_response();
                }
            };

            // データベースプールを取得
            let pool = match request.extensions().get::<PgPool>() {
                Some(pool) => pool.clone(),
                None => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({
                            "error": "データベース接続エラー"
                        })),
                    )
                        .into_response();
                }
            };

            // 機能アクセスをチェック
            match subscription_service::check_feature_access(&pool, user.user_id, feature_name)
                .await
            {
                Ok(true) => {
                    // アクセス可能なので続行
                    next.run(request).await
                }
                Ok(false) => {
                    // アクセス不可
                    (
                        StatusCode::PAYMENT_REQUIRED,
                        Json(json!({
                            "error": format!("{}機能はご利用のプランでは使用できません。プランをアップグレードしてください。", get_feature_display_name(feature_name))
                        })),
                    )
                        .into_response()
                }
                Err(_) => {
                    // エラーが発生した場合は通過させる（サービスを止めない）
                    next.run(request).await
                }
            }
        })
    }
}

/// メール送信数をカウントするミドルウェア
pub async fn count_email_usage(request: Request, next: Next) -> Response {
    let response = next.run(request).await;

    // レスポンスが成功した場合のみカウント
    if response.status().is_success() {
        // TODO: 実際のメール送信数をカウントする処理を実装
        // subscription_service::record_usage(&pool, user_id, "emails_sent", count).await
    }

    response
}

fn get_resource_display_name(resource_type: &str) -> &str {
    match resource_type {
        "campaigns" => "キャンペーン",
        "templates" => "テンプレート",
        "sequences" => "シーケンス",
        "forms" => "フォーム",
        "contacts" => "コンタクト",
        _ => resource_type,
    }
}

fn get_feature_display_name(feature: &str) -> &str {
    match feature {
        "custom_markdown_components" => "カスタムMarkdownコンポーネント",
        "ai_features" => "AI",
        "advanced_analytics" => "高度な分析",
        "ab_testing" => "A/Bテスト",
        "api_access" => "API",
        "priority_support" => "優先サポート",
        "custom_domain" => "カスタムドメイン",
        "white_label" => "ホワイトラベル",
        _ => feature,
    }
}

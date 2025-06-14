use axum::Router;
use serde::Deserialize;

use crate::AppState;

// 開発環境用のインポート
#[cfg(debug_assertions)]
use crate::{
    middleware::auth::AuthUser,
    services::email_service::{EmailMessage, EmailService},
};
#[cfg(debug_assertions)]
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::Json,
    routing::post,
};
#[cfg(debug_assertions)]
use serde_json::{json, Value};

/// テストメール送信リクエスト
#[derive(Debug, Deserialize)]
pub struct SendTestEmailRequest {
    pub to: String,
    pub subject: String,
    pub content: String,
}

/// テストメール送信（開発環境用）
#[cfg(debug_assertions)]
pub async fn send_test_email(
    Extension(_auth_user): Extension<AuthUser>,
    State(_state): State<AppState>,
    Json(payload): Json<SendTestEmailRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // メールサービスを初期化
    let email_config = EmailService::from_env().map_err(|e| {
        tracing::error!("メール設定エラー: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": format!("メール設定の読み込みに失敗しました: {}", e)
            })),
        )
    })?;

    let email_service = EmailService::new(email_config).await.map_err(|e| {
        tracing::error!("メールサービス初期化エラー: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": format!("メールサービスの初期化に失敗しました: {}", e)
            })),
        )
    })?;

    // HTMLとテキストコンテンツを生成
    let html_body = format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <meta charset="UTF-8">
            <title>{}</title>
            <style>
                body {{
                    font-family: Arial, sans-serif;
                    line-height: 1.6;
                    color: #333;
                    max-width: 600px;
                    margin: 0 auto;
                    padding: 20px;
                }}
                .header {{
                    background-color: #007bff;
                    color: white;
                    padding: 20px;
                    text-align: center;
                    border-radius: 5px 5px 0 0;
                }}
                .content {{
                    background-color: #f8f9fa;
                    padding: 20px;
                    border-radius: 0 0 5px 5px;
                }}
                .footer {{
                    margin-top: 20px;
                    text-align: center;
                    font-size: 12px;
                    color: #666;
                }}
            </style>
        </head>
        <body>
            <div class="header">
                <h1>テストメール</h1>
            </div>
            <div class="content">
                <h2>{}</h2>
                <p>{}</p>
                <p>このメールは<strong>MarkMail</strong>のテスト送信機能から送信されました。</p>
            </div>
            <div class="footer">
                <p>© 2025 MarkMail - エンジニア向けメールマーケティングツール</p>
            </div>
        </body>
        </html>
        "#,
        payload.subject, payload.subject, payload.content
    );

    let text_body = format!(
        "{}\n\n{}\n\nこのメールはMarkMailのテスト送信機能から送信されました。\n\n© 2025 MarkMail",
        payload.subject, payload.content
    );

    // メールメッセージを作成
    let message = EmailMessage {
        to: vec![payload.to.clone()],
        subject: payload.subject,
        html_body,
        text_body: Some(text_body),
        reply_to: None,
        headers: None,
    };

    // メール送信
    match email_service.send_email(&message).await {
        Ok(result) => {
            tracing::info!("テストメール送信成功: {:?}", result);
            Ok(Json(json!({
                "message": "テストメールが送信されました",
                "message_id": result.message_id,
                "status": result.status,
                "to": payload.to
            })))
        }
        Err(e) => {
            tracing::error!("テストメール送信エラー: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": format!("メール送信に失敗しました: {}", e)
                })),
            ))
        }
    }
}

/// メール関連のルーターを構築
pub fn router() -> Router<AppState> {
    let mut router = Router::new();

    // 開発環境でのみテストメール送信エンドポイントを有効化
    #[cfg(debug_assertions)]
    {
        router = router.route("/test", post(send_test_email));
    }

    router
}

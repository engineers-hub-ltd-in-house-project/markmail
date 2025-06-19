use crate::services::stripe_service::StripeService;
use crate::AppState;
use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use tracing::{error, info};

/// Stripe Webhookエンドポイント
pub async fn handle_stripe_webhook(
    State(_state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    // Stripe-Signatureヘッダーを取得
    let signature = match headers.get("stripe-signature") {
        Some(sig) => match sig.to_str() {
            Ok(s) => s,
            Err(_) => {
                error!("Invalid Stripe-Signature header");
                return StatusCode::BAD_REQUEST;
            }
        },
        None => {
            error!("Missing Stripe-Signature header");
            return StatusCode::BAD_REQUEST;
        }
    };

    // ボディを文字列に変換
    let payload = match std::str::from_utf8(&body) {
        Ok(s) => s,
        Err(_) => {
            error!("Invalid webhook payload");
            return StatusCode::BAD_REQUEST;
        }
    };

    // Stripeサービスを初期化
    let stripe_service = StripeService::new(
        std::env::var("STRIPE_SECRET_KEY").unwrap_or_default(),
        std::env::var("STRIPE_WEBHOOK_SECRET").unwrap_or_default(),
    );

    // 署名を検証してイベントを取得
    let event = match stripe_service.verify_webhook_signature(payload, signature) {
        Ok(e) => e,
        Err(e) => {
            error!("Webhook signature verification failed: {:?}", e);
            return StatusCode::UNAUTHORIZED;
        }
    };

    info!("Received webhook event: {:?}", event.type_);

    // イベントを処理
    match stripe_service.handle_webhook_event(event).await {
        Ok(_) => {
            info!("Webhook event processed successfully");
            StatusCode::OK
        }
        Err(e) => {
            error!("Failed to process webhook event: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

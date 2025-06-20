use crate::services::stripe_service::StripeService;
use crate::AppState;
use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use serde_json::Value;
use tracing::{error, info};

/// Stripe Webhookエンドポイント
pub async fn handle_stripe_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    // Stripe-Signatureヘッダーを取得
    let signature = match headers.get("stripe-signature") {
        Some(sig) => match sig.to_str() {
            Ok(s) => {
                info!("Received Stripe-Signature: {}", &s[..20.min(s.len())]);
                s
            }
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
    let webhook_secret = std::env::var("STRIPE_WEBHOOK_SECRET").unwrap_or_default();
    info!(
        "Using webhook secret: {}",
        if webhook_secret.is_empty() {
            "EMPTY"
        } else {
            &webhook_secret[..10]
        }
    );

    let stripe_service = StripeService::new(
        std::env::var("STRIPE_SECRET_KEY").unwrap_or_default(),
        webhook_secret,
    );

    // 署名を検証
    if let Err(e) = stripe_service.verify_webhook_raw(payload, signature) {
        error!("Webhook signature verification failed: {:?}", e);
        return StatusCode::UNAUTHORIZED;
    }

    // JSONをパース
    let json: Value = match serde_json::from_str(payload) {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to parse webhook JSON: {:?}", e);
            return StatusCode::BAD_REQUEST;
        }
    };

    // イベントタイプを取得
    let event_type = match json.get("type").and_then(|v| v.as_str()) {
        Some(t) => t,
        None => {
            error!("Missing event type in webhook");
            return StatusCode::BAD_REQUEST;
        }
    };

    info!("Received webhook event: {}", event_type);

    // イベントを処理
    match stripe_service
        .handle_webhook_json(event_type, &json, &state.db)
        .await
    {
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

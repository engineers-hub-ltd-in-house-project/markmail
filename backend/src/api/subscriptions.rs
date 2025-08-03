use crate::middleware::auth::AuthUser;
use crate::models::subscription::{CancelRequest, PlansResponse, UpgradeRequest};
use crate::services::{stripe_service::StripeService, subscription_service};
use crate::AppState;
use axum::{
    extract::{Extension, Query, State},
    response::Json,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// 全てのプランを取得
pub async fn get_plans(State(state): State<AppState>) -> Json<PlansResponse> {
    tracing::debug!("Getting subscription plans");

    match subscription_service::get_plans(&state.db).await {
        Ok(plans) => {
            tracing::debug!("Found {} plans", plans.len());
            Json(PlansResponse { plans })
        }
        Err(e) => {
            tracing::error!("Failed to get plans: {:?}", e);
            Json(PlansResponse { plans: vec![] })
        }
    }
}

/// 現在のサブスクリプション情報を取得
pub async fn get_subscription(
    Extension(user): Extension<AuthUser>,
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    match subscription_service::get_user_subscription_details(&state.db, user.user_id).await {
        Ok(details) => Json(serde_json::to_value(details).unwrap_or(serde_json::json!({}))),
        Err(e) => {
            tracing::error!("Failed to get subscription details: {:?}", e);
            Json(serde_json::json!({"error": "サブスクリプション情報の取得に失敗しました"}))
        }
    }
}

/// プランをアップグレード
pub async fn upgrade_plan(
    Extension(user): Extension<AuthUser>,
    State(state): State<AppState>,
    Json(request): Json<UpgradeRequest>,
) -> Json<serde_json::Value> {
    match subscription_service::upgrade_plan(&state.db, user.user_id, &request).await {
        Ok(subscription) => {
            Json(serde_json::to_value(subscription).unwrap_or(serde_json::json!({})))
        }
        Err(e) => Json(serde_json::json!({"error": e.to_string()})),
    }
}

/// サブスクリプションをキャンセル
pub async fn cancel_subscription(
    Extension(user): Extension<AuthUser>,
    State(state): State<AppState>,
    Json(request): Json<CancelRequest>,
) -> Json<serde_json::Value> {
    match subscription_service::cancel_subscription(&state.db, user.user_id, &request).await {
        Ok(subscription) => {
            Json(serde_json::to_value(subscription).unwrap_or(serde_json::json!({})))
        }
        Err(e) => Json(serde_json::json!({"error": e.to_string()})),
    }
}

/// 支払い履歴を取得
pub async fn get_payment_history(
    Extension(user): Extension<AuthUser>,
    Query(query): Query<PaginationQuery>,
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    match subscription_service::get_payment_history(&state.db, user.user_id, limit, offset).await {
        Ok(history) => Json(serde_json::json!({
            "payments": history,
            "limit": limit,
            "offset": offset
        })),
        Err(_) => Json(serde_json::json!({
            "payments": [],
            "limit": limit,
            "offset": offset
        })),
    }
}

/// 使用量を取得
pub async fn get_usage(
    Extension(user): Extension<AuthUser>,
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    match subscription_service::get_user_subscription_details(&state.db, user.user_id).await {
        Ok(details) => Json(serde_json::to_value(details.usage).unwrap_or(serde_json::json!({}))),
        Err(_) => Json(serde_json::json!({})),
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateCheckoutRequest {
    pub plan_id: String,
    pub success_url: String,
    pub cancel_url: String,
}

#[derive(Debug, Serialize)]
pub struct CreateCheckoutResponse {
    pub checkout_url: String,
}

/// Stripe Checkout Sessionを作成
pub async fn create_checkout_session(
    Extension(user): Extension<AuthUser>,
    State(state): State<AppState>,
    Json(request): Json<CreateCheckoutRequest>,
) -> Json<serde_json::Value> {
    // ユーザーの情報を取得（Stripe customer IDを含む）
    let user_info = match crate::database::users::get_user_by_id(&state.db, user.user_id).await {
        Ok(Some(u)) => u,
        _ => {
            return Json(serde_json::json!({"error": "ユーザーが見つかりません"}));
        }
    };

    // プラン情報を取得
    let plan = match crate::database::subscriptions::get_plan_by_id(
        &state.db,
        request.plan_id.parse().unwrap_or_default(),
    )
    .await
    {
        Ok(p) => p,
        Err(_) => {
            return Json(serde_json::json!({"error": "プランが見つかりません"}));
        }
    };

    // Stripeサービスの初期化（TODO: AppStateに移動）
    let stripe_service = StripeService::new(
        std::env::var("STRIPE_SECRET_KEY").unwrap_or_default(),
        std::env::var("STRIPE_WEBHOOK_SECRET").unwrap_or_default(),
    );

    // Stripe顧客の作成または取得
    let customer_id = if let Some(stripe_id) = user_info.stripe_customer_id {
        stripe_id
    } else {
        // 新規顧客を作成
        match stripe_service
            .create_customer(user.user_id, &user_info.email, user_info.name.as_deref())
            .await
        {
            Ok(customer) => {
                // データベースに保存
                let _ = crate::database::users::update_stripe_customer_id(
                    &state.db,
                    user.user_id,
                    customer.id.as_ref(),
                )
                .await;
                customer.id.to_string()
            }
            Err(e) => {
                tracing::error!("Stripe顧客の作成に失敗: {:?}", e);
                return Json(serde_json::json!({"error": "顧客の作成に失敗しました"}));
            }
        }
    };

    // Checkout Sessionを作成
    match stripe_service
        .create_checkout_session(
            &customer_id,
            &plan.stripe_price_id.unwrap_or_default(),
            &request.success_url,
            &request.cancel_url,
        )
        .await
    {
        Ok(checkout_url) => Json(serde_json::json!({
            "checkout_url": checkout_url
        })),
        Err(e) => {
            tracing::error!("Checkout Sessionの作成に失敗: {:?}", e);
            Json(serde_json::json!({"error": "Checkout Sessionの作成に失敗しました"}))
        }
    }
}

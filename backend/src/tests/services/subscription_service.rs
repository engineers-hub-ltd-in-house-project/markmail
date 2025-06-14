use crate::{
    database::subscriptions,
    models::subscription::{CancelRequest, UpgradeRequest},
    services::subscription_service,
    tests::api::{subscriptions::cleanup_test_data, templates::create_test_user},
    AppState,
};
use chrono::Datelike;
use sqlx::PgPool;
use uuid::Uuid;

// テスト用のヘルパー関数 - サブスクリプションは自動作成される
async fn create_test_user_with_free_subscription(pool: &PgPool) -> Uuid {
    let user_id = create_test_user(pool).await;

    // フリープランのサブスクリプションは自動作成されるため、そのまま返す
    user_id
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_plans() {
        let state = AppState::new_for_test().await;
        let pool = state.db;
        cleanup_test_data(&pool).await;

        let plans = subscription_service::get_plans(&pool)
            .await
            .expect("Failed to get plans");

        assert_eq!(plans.len(), 3);

        let plan_names: Vec<String> = plans.iter().map(|p| p.name.clone()).collect();
        assert!(plan_names.contains(&"free".to_string()));
        assert!(plan_names.contains(&"pro".to_string()));
        assert!(plan_names.contains(&"business".to_string()));
    }

    #[tokio::test]
    async fn test_get_user_subscription_details_existing() {
        let state = AppState::new_for_test().await;
        let pool = state.db;
        cleanup_test_data(&pool).await;
        let user_id = create_test_user_with_free_subscription(&pool).await;

        let details = subscription_service::get_user_subscription_details(&pool, user_id)
            .await
            .expect("Failed to get subscription details");

        assert_eq!(details.plan.name, "free");
        assert_eq!(details.subscription.user_id, user_id);
        assert_eq!(details.subscription.status, "active");

        // 使用量の確認
        assert_eq!(details.usage.contacts.limit, 100); // freeプランの制限
        assert_eq!(details.usage.emails_sent.limit, 1000);
        assert_eq!(details.usage.campaigns.limit, 10);
    }

    #[tokio::test]
    async fn test_get_user_subscription_details_auto_create() {
        let state = AppState::new_for_test().await;
        let pool = state.db;
        cleanup_test_data(&pool).await;
        let user_id = create_test_user(&pool).await; // サブスクリプションなし

        // サブスクリプションが自動作成されるはず
        let details = subscription_service::get_user_subscription_details(&pool, user_id)
            .await
            .expect("Failed to get subscription details");

        assert_eq!(details.plan.name, "free");
        assert_eq!(details.subscription.user_id, user_id);
        assert_eq!(details.subscription.status, "active");
    }

    #[tokio::test]
    async fn test_upgrade_plan_success() {
        let state = AppState::new_for_test().await;
        let pool = state.db;
        cleanup_test_data(&pool).await;
        let user_id = create_test_user_with_free_subscription(&pool).await;

        // proプランを取得
        let pro_plan = subscriptions::get_plan_by_name(&pool, "pro")
            .await
            .expect("Failed to get pro plan");

        let upgrade_request = UpgradeRequest {
            plan_id: pro_plan.id,
        };

        let updated_subscription =
            subscription_service::upgrade_plan(&pool, user_id, &upgrade_request)
                .await
                .expect("Failed to upgrade plan");

        assert_eq!(updated_subscription.plan_id, pro_plan.id);

        // データベースで確認
        let subscription = subscriptions::get_user_subscription(&pool, user_id)
            .await
            .expect("Failed to get subscription")
            .expect("Subscription not found");

        assert_eq!(subscription.plan_id, pro_plan.id);
    }

    #[tokio::test]
    async fn test_upgrade_plan_with_usage_limit_check() {
        let state = AppState::new_for_test().await;
        let pool = state.db;
        cleanup_test_data(&pool).await;
        let user_id = create_test_user(&pool).await;

        // ユーザーは自動的にfreeプランでサブスクリプションが作成される
        // businessプランにアップグレード
        let business_plan = subscriptions::get_plan_by_name(&pool, "business")
            .await
            .expect("Failed to get business plan");

        let upgrade_request = UpgradeRequest {
            plan_id: business_plan.id,
        };

        subscription_service::upgrade_plan(&pool, user_id, &upgrade_request)
            .await
            .expect("Failed to upgrade to business plan");

        // 大量のコンタクトを作成（freeプランの制限を超える）
        for i in 0..150 {
            sqlx::query!(
                r#"
                INSERT INTO subscribers (user_id, email, name, status)
                VALUES ($1, $2, $3, 'active')
                "#,
                user_id,
                format!("contact{}@example.com", i),
                format!("Contact {}", i)
            )
            .execute(&pool)
            .await
            .expect("Failed to create test contact");
        }

        // freeプランへのダウングレードを試行
        let free_plan = subscriptions::get_plan_by_name(&pool, "free")
            .await
            .expect("Failed to get free plan");

        let downgrade_request = UpgradeRequest {
            plan_id: free_plan.id,
        };

        let result = subscription_service::upgrade_plan(&pool, user_id, &downgrade_request).await;

        // 使用量が制限を超えているため失敗するはず
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("コンタクト数") || error_message.contains("制限"));
    }

    #[tokio::test]
    async fn test_cancel_subscription_immediate() {
        let state = AppState::new_for_test().await;
        let pool = state.db;
        cleanup_test_data(&pool).await;
        let user_id = create_test_user_with_free_subscription(&pool).await;

        let cancel_request = CancelRequest {
            cancel_at_period_end: false,
        };

        let canceled_subscription =
            subscription_service::cancel_subscription(&pool, user_id, &cancel_request)
                .await
                .expect("Failed to cancel subscription");

        assert_eq!(canceled_subscription.status, "canceled");
        assert!(canceled_subscription.canceled_at.is_some());
        assert!(canceled_subscription.cancel_at.is_some());
    }

    #[tokio::test]
    async fn test_cancel_subscription_at_period_end() {
        let state = AppState::new_for_test().await;
        let pool = state.db;
        cleanup_test_data(&pool).await;
        let user_id = create_test_user_with_free_subscription(&pool).await;

        let cancel_request = CancelRequest {
            cancel_at_period_end: true,
        };

        let canceled_subscription =
            subscription_service::cancel_subscription(&pool, user_id, &cancel_request)
                .await
                .expect("Failed to cancel subscription");

        // 期間終了時のキャンセルの場合、ステータスは変わらないが、cancel_atが設定される
        assert!(canceled_subscription.canceled_at.is_some());
        assert!(canceled_subscription.cancel_at.is_some());
    }

    #[tokio::test]
    async fn test_check_resource_limit() {
        let state = AppState::new_for_test().await;
        let pool = state.db;
        cleanup_test_data(&pool).await;
        let user_id = create_test_user_with_free_subscription(&pool).await;

        // 制限内のテスト
        let can_create = subscription_service::check_resource_limit(&pool, user_id, "contacts")
            .await
            .expect("Failed to check resource limit");

        assert!(can_create);

        // 制限を超えるコンタクトを作成
        for i in 0..100 {
            sqlx::query!(
                r#"
                INSERT INTO subscribers (user_id, email, name, status)
                VALUES ($1, $2, $3, 'active')
                "#,
                user_id,
                format!("contact{}@example.com", i),
                format!("Contact {}", i)
            )
            .execute(&pool)
            .await
            .expect("Failed to create test contact");
        }

        // 制限に達した後のテスト
        let can_create_more =
            subscription_service::check_resource_limit(&pool, user_id, "contacts")
                .await
                .expect("Failed to check resource limit");

        assert!(!can_create_more);
    }

    #[tokio::test]
    async fn test_record_usage() {
        let state = AppState::new_for_test().await;
        let pool = state.db;
        cleanup_test_data(&pool).await;
        let user_id = create_test_user_with_free_subscription(&pool).await;

        // 使用量を記録
        subscription_service::record_usage(&pool, user_id, "emails_sent", 10)
            .await
            .expect("Failed to record usage");

        // 追加で記録
        subscription_service::record_usage(&pool, user_id, "emails_sent", 5)
            .await
            .expect("Failed to record usage");

        // 使用量を確認
        let now = chrono::Utc::now().date_naive();
        let period_start = now.with_day(1).unwrap_or(now);
        let period_end = (period_start + chrono::Duration::days(31))
            .with_day(1)
            .unwrap_or(period_start + chrono::Duration::days(31))
            - chrono::Duration::days(1);

        let usage_record = subscriptions::get_usage_for_period(
            &pool,
            user_id,
            "emails_sent",
            period_start,
            period_end,
        )
        .await
        .expect("Failed to get usage")
        .expect("Usage record not found");

        assert_eq!(usage_record.usage_count, 15); // 10 + 5
    }

    #[tokio::test]
    async fn test_check_feature_access() {
        let state = AppState::new_for_test().await;
        let pool = state.db;
        cleanup_test_data(&pool).await;
        let user_id = create_test_user_with_free_subscription(&pool).await;

        // freeプランではAI機能は使用不可
        let has_ai_features =
            subscription_service::check_feature_access(&pool, user_id, "ai_features")
                .await
                .expect("Failed to check feature access");

        assert!(!has_ai_features);

        // proプランにアップグレード
        let pro_plan = subscriptions::get_plan_by_name(&pool, "pro")
            .await
            .expect("Failed to get pro plan");

        let upgrade_request = UpgradeRequest {
            plan_id: pro_plan.id,
        };

        subscription_service::upgrade_plan(&pool, user_id, &upgrade_request)
            .await
            .expect("Failed to upgrade plan");

        // proプランではAI機能が使用可能
        let has_ai_features_after_upgrade =
            subscription_service::check_feature_access(&pool, user_id, "ai_features")
                .await
                .expect("Failed to check feature access");

        assert!(has_ai_features_after_upgrade);
    }
}

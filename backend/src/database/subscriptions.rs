use crate::models::subscription::{
    PaymentHistory, PaymentStatus, SubscriptionPlan, UsageAlert, UsageRecord, UserSubscription,
};
use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

/// 全てのアクティブなプランを取得
pub async fn get_active_plans(pool: &PgPool) -> Result<Vec<SubscriptionPlan>> {
    let plans = sqlx::query_as!(
        SubscriptionPlan,
        r#"
        SELECT 
            id, name, display_name, description, price, billing_interval,
            contact_limit, monthly_email_limit, campaign_limit, template_limit,
            sequence_limit, sequence_step_limit, form_limit, form_submission_limit,
            user_limit, webhook_limit, custom_markdown_components, ai_features, 
            advanced_analytics, ab_testing, api_access, priority_support, 
            custom_domain, white_label, sort_order, is_active, 
            features as "features: Option<serde_json::Value>", 
            created_at, updated_at
        FROM subscription_plans
        WHERE is_active = true
        ORDER BY sort_order, price
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(plans)
}

/// 特定のプランを取得
pub async fn get_plan_by_id(pool: &PgPool, plan_id: Uuid) -> Result<SubscriptionPlan> {
    let plan = sqlx::query_as!(
        SubscriptionPlan,
        r#"
        SELECT 
            id, name, display_name, description, price, billing_interval,
            contact_limit, monthly_email_limit, campaign_limit, template_limit,
            sequence_limit, sequence_step_limit, form_limit, form_submission_limit,
            user_limit, webhook_limit, custom_markdown_components, ai_features, 
            advanced_analytics, ab_testing, api_access, priority_support, 
            custom_domain, white_label, sort_order, is_active, 
            features as "features: Option<serde_json::Value>", 
            created_at, updated_at
        FROM subscription_plans
        WHERE id = $1
        "#,
        plan_id
    )
    .fetch_one(pool)
    .await?;

    Ok(plan)
}

/// プラン名でプランを取得
#[allow(dead_code)]
pub async fn get_plan_by_name(pool: &PgPool, name: &str) -> Result<SubscriptionPlan> {
    let plan = sqlx::query_as!(
        SubscriptionPlan,
        r#"
        SELECT 
            id, name, display_name, description, price, billing_interval,
            contact_limit, monthly_email_limit, campaign_limit, template_limit,
            sequence_limit, sequence_step_limit, form_limit, form_submission_limit,
            user_limit, webhook_limit, custom_markdown_components, ai_features, 
            advanced_analytics, ab_testing, api_access, priority_support, 
            custom_domain, white_label, sort_order, is_active, 
            features as "features: Option<serde_json::Value>", 
            created_at, updated_at
        FROM subscription_plans
        WHERE name = $1
        "#,
        name
    )
    .fetch_one(pool)
    .await?;

    Ok(plan)
}

/// ユーザーのサブスクリプションを取得
pub async fn get_user_subscription(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<UserSubscription>> {
    let subscription = sqlx::query_as!(
        UserSubscription,
        r#"
        SELECT 
            id, user_id, plan_id, status, current_period_start, 
            current_period_end, cancel_at, canceled_at, trial_end,
            metadata as "metadata: Option<serde_json::Value>", 
            stripe_subscription_id, stripe_customer_id,
            created_at, updated_at
        FROM user_subscriptions
        WHERE user_id = $1
        "#,
        user_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(subscription)
}

/// サブスクリプションを作成
#[allow(dead_code)]
pub async fn create_subscription(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    plan_id: Uuid,
    period_end: DateTime<Utc>,
) -> Result<UserSubscription> {
    let subscription = sqlx::query_as!(
        UserSubscription,
        r#"
        INSERT INTO user_subscriptions (
            user_id, plan_id, status, current_period_start, 
            current_period_end, metadata
        )
        VALUES ($1, $2, 'active', CURRENT_TIMESTAMP, $3, '{}')
        RETURNING 
            id, user_id, plan_id, status, current_period_start, 
            current_period_end, cancel_at, canceled_at, trial_end,
            metadata as "metadata: Option<serde_json::Value>", 
            stripe_subscription_id, stripe_customer_id,
            created_at, updated_at
        "#,
        user_id,
        plan_id,
        period_end
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(subscription)
}

/// 使用量記録を取得
pub async fn get_usage_for_period(
    pool: &PgPool,
    user_id: Uuid,
    metric_type: &str,
    period_start: chrono::NaiveDate,
    period_end: chrono::NaiveDate,
) -> Result<Option<UsageRecord>> {
    let record = sqlx::query_as!(
        UsageRecord,
        r#"
        SELECT 
            id, user_id, metric_type, usage_count, 
            period_start, period_end, created_at, updated_at
        FROM usage_records
        WHERE user_id = $1 
          AND metric_type = $2
          AND period_start = $3
          AND period_end = $4
        "#,
        user_id,
        metric_type,
        period_start,
        period_end
    )
    .fetch_optional(pool)
    .await?;

    Ok(record)
}

/// サブスクリプションを更新
pub async fn update_subscription(
    tx: &mut Transaction<'_, Postgres>,
    subscription_id: Uuid,
    plan_id: Option<Uuid>,
    status: Option<&str>,
    stripe_subscription_id: Option<String>,
    stripe_customer_id: Option<String>,
) -> Result<UserSubscription> {
    let subscription = sqlx::query_as!(
        UserSubscription,
        r#"
        UPDATE user_subscriptions
        SET 
            plan_id = COALESCE($2, plan_id),
            status = COALESCE($3, status),
            stripe_subscription_id = COALESCE($4, stripe_subscription_id),
            stripe_customer_id = COALESCE($5, stripe_customer_id),
            updated_at = CURRENT_TIMESTAMP
        WHERE id = $1
        RETURNING 
            id, user_id, plan_id, status, current_period_start, 
            current_period_end, cancel_at, canceled_at, trial_end,
            metadata as "metadata: Option<serde_json::Value>", 
            stripe_subscription_id, stripe_customer_id,
            created_at, updated_at
        "#,
        subscription_id,
        plan_id,
        status,
        stripe_subscription_id,
        stripe_customer_id
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(subscription)
}

/// サブスクリプションをキャンセル
pub async fn cancel_subscription(
    tx: &mut Transaction<'_, Postgres>,
    subscription_id: Uuid,
    cancel_at_period_end: bool,
) -> Result<UserSubscription> {
    let subscription = if cancel_at_period_end {
        // 期間終了時にキャンセル
        sqlx::query_as!(
            UserSubscription,
            r#"
            UPDATE user_subscriptions
            SET 
                cancel_at = current_period_end,
                canceled_at = CURRENT_TIMESTAMP,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $1
            RETURNING 
                id, user_id, plan_id, status, current_period_start, 
                current_period_end, cancel_at, canceled_at, trial_end,
                metadata as "metadata: Option<serde_json::Value>", 
                stripe_subscription_id, stripe_customer_id,
                created_at, updated_at
            "#,
            subscription_id
        )
        .fetch_one(&mut **tx)
        .await?
    } else {
        // 即座にキャンセル
        sqlx::query_as!(
            UserSubscription,
            r#"
            UPDATE user_subscriptions
            SET 
                status = 'canceled',
                cancel_at = CURRENT_TIMESTAMP,
                canceled_at = CURRENT_TIMESTAMP,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $1
            RETURNING 
                id, user_id, plan_id, status, current_period_start, 
                current_period_end, cancel_at, canceled_at, trial_end,
                metadata as "metadata: Option<serde_json::Value>", 
                stripe_subscription_id, stripe_customer_id,
                created_at, updated_at
            "#,
            subscription_id
        )
        .fetch_one(&mut **tx)
        .await?
    };

    Ok(subscription)
}

/// 使用量を記録
pub async fn record_usage(
    pool: &PgPool,
    user_id: Uuid,
    metric_type: &str,
    count: i32,
    period_start: chrono::NaiveDate,
    period_end: chrono::NaiveDate,
) -> Result<UsageRecord> {
    // 既存のレコードを確認
    let existing =
        get_usage_for_period(pool, user_id, metric_type, period_start, period_end).await?;

    let record = if let Some(existing_record) = existing {
        // 既存レコードを更新
        sqlx::query_as!(
            UsageRecord,
            r#"
            UPDATE usage_records 
            SET 
                usage_count = usage_count + $5,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $1 AND user_id = $2 AND metric_type = $3 AND period_start = $4 AND period_end = $6
            RETURNING 
                id, user_id, metric_type, usage_count, 
                period_start, period_end, created_at, updated_at
            "#,
            existing_record.id,
            user_id,
            metric_type,
            period_start,
            count,
            period_end
        )
        .fetch_one(pool)
        .await?
    } else {
        // 新規レコードを作成
        sqlx::query_as!(
            UsageRecord,
            r#"
            INSERT INTO usage_records (
                user_id, metric_type, usage_count, period_start, period_end
            )
            VALUES ($1, $2, $3, $4, $5)
            RETURNING 
                id, user_id, metric_type, usage_count, 
                period_start, period_end, created_at, updated_at
            "#,
            user_id,
            metric_type,
            count,
            period_start,
            period_end
        )
        .fetch_one(pool)
        .await?
    };

    Ok(record)
}

/// 現在の期間の使用量を取得
pub async fn get_current_usage(
    pool: &PgPool,
    user_id: Uuid,
    period_start: chrono::NaiveDate,
    period_end: chrono::NaiveDate,
) -> Result<Vec<UsageRecord>> {
    let records = sqlx::query_as!(
        UsageRecord,
        r#"
        SELECT 
            id, user_id, metric_type, usage_count, 
            period_start, period_end, created_at, updated_at
        FROM usage_records
        WHERE user_id = $1
          AND period_start = $2
          AND period_end = $3
        "#,
        user_id,
        period_start,
        period_end
    )
    .fetch_all(pool)
    .await?;

    Ok(records)
}

/// ユーザーのリソース数を取得
pub async fn count_user_resources(
    pool: &PgPool,
    user_id: Uuid,
    resource_type: &str,
) -> Result<i64> {
    let count = match resource_type {
        "contacts" => {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM subscribers WHERE user_id = $1",
                user_id
            )
            .fetch_one(pool)
            .await?
        }
        "campaigns" => {
            sqlx::query_scalar!("SELECT COUNT(*) FROM campaigns WHERE user_id = $1", user_id)
                .fetch_one(pool)
                .await?
        }
        "templates" => {
            sqlx::query_scalar!("SELECT COUNT(*) FROM templates WHERE user_id = $1", user_id)
                .fetch_one(pool)
                .await?
        }
        "sequences" => {
            sqlx::query_scalar!("SELECT COUNT(*) FROM sequences WHERE user_id = $1", user_id)
                .fetch_one(pool)
                .await?
        }
        "forms" => {
            sqlx::query_scalar!("SELECT COUNT(*) FROM forms WHERE user_id = $1", user_id)
                .fetch_one(pool)
                .await?
        }
        _ => Some(0),
    };

    Ok(count.unwrap_or(0))
}

/// 支払い履歴を作成
pub async fn create_payment_history(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    subscription_id: Option<Uuid>,
    amount: i32,
    status: PaymentStatus,
    stripe_payment_intent_id: Option<String>,
) -> Result<PaymentHistory> {
    let payment = sqlx::query_as!(
        PaymentHistory,
        r#"
        INSERT INTO payment_history (
            user_id, subscription_id, amount, currency, status,
            stripe_payment_intent_id, metadata
        )
        VALUES ($1, $2, $3, 'JPY', $4, $5, '{}')
        RETURNING 
            id, user_id, subscription_id, amount, currency,
            status, description, stripe_payment_intent_id,
            stripe_invoice_id, metadata as "metadata: serde_json::Value",
            paid_at, created_at
        "#,
        user_id,
        subscription_id,
        amount,
        status as PaymentStatus,
        stripe_payment_intent_id
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(payment)
}

/// 支払い履歴を取得
pub async fn get_payment_history(
    pool: &PgPool,
    user_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<PaymentHistory>> {
    let payments = sqlx::query_as!(
        PaymentHistory,
        r#"
        SELECT 
            id, user_id, subscription_id, amount, currency,
            status, description, stripe_payment_intent_id,
            stripe_invoice_id, metadata as "metadata: serde_json::Value",
            paid_at, created_at
        FROM payment_history
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        user_id,
        limit,
        offset
    )
    .fetch_all(pool)
    .await?;

    Ok(payments)
}

/// 使用量アラートを取得
pub async fn get_usage_alerts(pool: &PgPool, user_id: Uuid) -> Result<Vec<UsageAlert>> {
    let alerts = sqlx::query_as!(
        UsageAlert,
        r#"
        SELECT 
            id, user_id, metric_type, threshold_percentage,
            is_enabled, last_alerted_at, created_at, updated_at
        FROM usage_alerts
        WHERE user_id = $1 AND is_enabled = true
        "#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    Ok(alerts)
}

/// 使用量アラートを更新
pub async fn update_usage_alert(
    pool: &PgPool,
    alert_id: Uuid,
    last_alerted_at: DateTime<Utc>,
) -> Result<()> {
    sqlx::query!(
        r#"
        UPDATE usage_alerts
        SET last_alerted_at = $2, updated_at = CURRENT_TIMESTAMP
        WHERE id = $1
        "#,
        alert_id,
        last_alerted_at
    )
    .execute(pool)
    .await?;

    Ok(())
}

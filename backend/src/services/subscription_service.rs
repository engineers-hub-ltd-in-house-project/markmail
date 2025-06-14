use crate::database::subscriptions;
use crate::models::subscription::{
    CancelRequest, PaymentHistory, PaymentStatus, SubscriptionDetailsResponse, SubscriptionPlan,
    UpgradeRequest, UsageMetric, UsageSummary, UserSubscription,
};
use anyhow::{anyhow, Result};
use chrono::{Datelike, Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

/// 全てのアクティブなプランを取得
pub async fn get_plans(pool: &PgPool) -> Result<Vec<SubscriptionPlan>> {
    subscriptions::get_active_plans(pool).await
}

/// ユーザーのサブスクリプション情報を取得
pub async fn get_user_subscription_details(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<SubscriptionDetailsResponse> {
    // サブスクリプション情報を取得
    let subscription = match subscriptions::get_user_subscription(pool, user_id).await? {
        Some(sub) => sub,
        None => {
            // サブスクリプションが存在しない場合、freeプランを自動割り当て
            tracing::info!(
                "No subscription found for user {}. Creating free plan.",
                user_id
            );

            // freeプランを取得
            let free_plan = subscriptions::get_plan_by_name(pool, "free").await?;

            // トランザクション開始
            let mut tx = pool.begin().await?;

            // 30日後の期限を設定
            let period_end = Utc::now() + Duration::days(30);

            // サブスクリプションを作成
            let new_subscription =
                subscriptions::create_subscription(&mut tx, user_id, free_plan.id, period_end)
                    .await?;

            tx.commit().await?;

            new_subscription
        }
    };

    // プラン情報を取得
    let plan = subscriptions::get_plan_by_id(pool, subscription.plan_id).await?;

    // 現在の使用量を計算
    let usage = calculate_usage_summary(pool, user_id, &plan).await?;

    Ok(SubscriptionDetailsResponse {
        subscription,
        plan,
        usage,
    })
}

/// 使用量サマリーを計算
async fn calculate_usage_summary(
    pool: &PgPool,
    user_id: Uuid,
    plan: &SubscriptionPlan,
) -> Result<UsageSummary> {
    // 現在の期間を計算
    let now = Utc::now().date_naive();
    let period_start = now.with_day(1).unwrap_or(now);
    let period_end = (period_start + Duration::days(31))
        .with_day(1)
        .unwrap_or(period_start + Duration::days(31))
        - Duration::days(1);

    tracing::debug!(
        "Calculating usage for user {} from {} to {}",
        user_id,
        period_start,
        period_end
    );

    // 各リソースの使用量を取得
    let contacts_count = subscriptions::count_user_resources(pool, user_id, "contacts").await?;
    let campaigns_count = subscriptions::count_user_resources(pool, user_id, "campaigns").await?;
    let templates_count = subscriptions::count_user_resources(pool, user_id, "templates").await?;
    let sequences_count = subscriptions::count_user_resources(pool, user_id, "sequences").await?;
    let forms_count = subscriptions::count_user_resources(pool, user_id, "forms").await?;

    // 月間メール送信数を取得
    let emails_sent =
        subscriptions::get_usage_for_period(pool, user_id, "emails_sent", period_start, period_end)
            .await?
            .map(|r| r.usage_count)
            .unwrap_or(0);

    // フォーム送信数を取得
    let form_submissions = subscriptions::get_usage_for_period(
        pool,
        user_id,
        "form_submissions",
        period_start,
        period_end,
    )
    .await?
    .map(|r| r.usage_count)
    .unwrap_or(0);

    Ok(UsageSummary {
        contacts: UsageMetric::new(contacts_count as i32, plan.contact_limit),
        emails_sent: UsageMetric::new(emails_sent, plan.monthly_email_limit),
        campaigns: UsageMetric::new(campaigns_count as i32, plan.campaign_limit),
        templates: UsageMetric::new(templates_count as i32, plan.template_limit),
        sequences: UsageMetric::new(sequences_count as i32, plan.sequence_limit),
        forms: UsageMetric::new(forms_count as i32, plan.form_limit),
        form_submissions: UsageMetric::new(form_submissions, plan.form_submission_limit),
    })
}

/// プランをアップグレード
pub async fn upgrade_plan(
    pool: &PgPool,
    user_id: Uuid,
    request: &UpgradeRequest,
) -> Result<UserSubscription> {
    let mut tx = pool.begin().await?;

    // 現在のサブスクリプションを取得
    let current_subscription = subscriptions::get_user_subscription(pool, user_id)
        .await?
        .ok_or_else(|| anyhow!("サブスクリプションが見つかりません"))?;

    // 新しいプランを取得
    let new_plan = subscriptions::get_plan_by_id(pool, request.plan_id).await?;

    // 現在のプランを取得
    let current_plan = subscriptions::get_plan_by_id(pool, current_subscription.plan_id).await?;

    // ダウングレードのチェック
    if new_plan.price < current_plan.price {
        // ダウングレードの場合は使用量チェック
        let usage = calculate_usage_summary(pool, user_id, &new_plan).await?;

        // 制限を超えている項目があるかチェック
        if usage.contacts.is_at_limit() {
            return Err(anyhow!("コンタクト数が新しいプランの制限を超えています"));
        }
        if usage.campaigns.is_at_limit() {
            return Err(anyhow!("キャンペーン数が新しいプランの制限を超えています"));
        }
        if usage.templates.is_at_limit() {
            return Err(anyhow!("テンプレート数が新しいプランの制限を超えています"));
        }
        if usage.sequences.is_at_limit() {
            return Err(anyhow!("シーケンス数が新しいプランの制限を超えています"));
        }
        if usage.forms.is_at_limit() {
            return Err(anyhow!("フォーム数が新しいプランの制限を超えています"));
        }
    }

    // サブスクリプションを更新
    let updated_subscription = subscriptions::update_subscription(
        &mut tx,
        current_subscription.id,
        Some(request.plan_id),
        None,
        None,
        None,
    )
    .await?;

    // 支払い履歴を記録（実際の実装では Stripe 連携が必要）
    let amount_diff = new_plan.price - current_plan.price;
    if amount_diff > 0 {
        subscriptions::create_payment_history(
            &mut tx,
            user_id,
            Some(updated_subscription.id),
            amount_diff,
            PaymentStatus::Pending,
            None,
        )
        .await?;
    }

    tx.commit().await?;

    Ok(updated_subscription)
}

/// サブスクリプションをキャンセル
pub async fn cancel_subscription(
    pool: &PgPool,
    user_id: Uuid,
    request: &CancelRequest,
) -> Result<UserSubscription> {
    let mut tx = pool.begin().await?;

    // 現在のサブスクリプションを取得
    let subscription = subscriptions::get_user_subscription(pool, user_id)
        .await?
        .ok_or_else(|| anyhow!("サブスクリプションが見つかりません"))?;

    // キャンセル処理
    let canceled_subscription =
        subscriptions::cancel_subscription(&mut tx, subscription.id, request.cancel_at_period_end)
            .await?;

    tx.commit().await?;

    Ok(canceled_subscription)
}

/// 支払い履歴を取得
pub async fn get_payment_history(
    pool: &PgPool,
    user_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<PaymentHistory>> {
    subscriptions::get_payment_history(pool, user_id, limit, offset).await
}

/// 制限チェック
#[allow(dead_code)]
pub async fn check_resource_limit(
    pool: &PgPool,
    user_id: Uuid,
    resource_type: &str,
) -> Result<bool> {
    // サブスクリプション情報を取得
    let subscription = subscriptions::get_user_subscription(pool, user_id)
        .await?
        .ok_or_else(|| anyhow!("サブスクリプションが見つかりません"))?;

    // プラン情報を取得
    let plan = subscriptions::get_plan_by_id(pool, subscription.plan_id).await?;

    // 現在のリソース数を取得
    let current_count = subscriptions::count_user_resources(pool, user_id, resource_type).await?;

    // 制限値を取得
    let limit = plan.get_limit(resource_type);

    // 制限なし（-1）または制限内
    Ok(limit < 0 || current_count < limit as i64)
}

/// 使用量を記録
#[allow(dead_code)]
pub async fn record_usage(
    pool: &PgPool,
    user_id: Uuid,
    metric_type: &str,
    count: i32,
) -> Result<()> {
    let now = Utc::now().date_naive();
    let period_start = now.with_day(1).unwrap_or(now);
    let period_end = (period_start + Duration::days(31))
        .with_day(1)
        .unwrap_or(period_start + Duration::days(31))
        - Duration::days(1);

    subscriptions::record_usage(pool, user_id, metric_type, count, period_start, period_end)
        .await?;

    Ok(())
}

/// 機能が利用可能かチェック
#[allow(dead_code)]
pub async fn check_feature_access(pool: &PgPool, user_id: Uuid, feature: &str) -> Result<bool> {
    // サブスクリプション情報を取得
    let subscription = subscriptions::get_user_subscription(pool, user_id)
        .await?
        .ok_or_else(|| anyhow!("サブスクリプションが見つかりません"))?;

    // プラン情報を取得
    let plan = subscriptions::get_plan_by_id(pool, subscription.plan_id).await?;

    Ok(plan.has_feature(feature))
}

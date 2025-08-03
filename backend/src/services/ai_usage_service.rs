use crate::models::ai_usage::{AiUsageLog, AiUsageStats, CreateAiUsageLog};
use crate::models::subscription::SubscriptionPlan;
use chrono::{Datelike, Utc};
use sqlx::{Error, PgPool};
use uuid::Uuid;

pub struct AiUsageService;

impl AiUsageService {
    /// AI使用ログを記録
    pub async fn record_usage(
        pool: &PgPool,
        usage_log: CreateAiUsageLog,
    ) -> Result<AiUsageLog, Error> {
        let log = sqlx::query_as!(
            AiUsageLog,
            r#"
            INSERT INTO ai_usage_logs (user_id, feature_type, prompt, response, tokens_used, model_used)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, user_id, feature_type, prompt, response, tokens_used, model_used, created_at as "created_at!"
            "#,
            usage_log.user_id,
            usage_log.feature_type,
            usage_log.prompt,
            usage_log.response,
            usage_log.tokens_used,
            usage_log.model_used
        )
        .fetch_one(pool)
        .await?;

        Ok(log)
    }

    /// ユーザーの月間AI使用量を取得
    pub async fn get_monthly_usage(pool: &PgPool, user_id: Uuid) -> Result<AiUsageStats, Error> {
        let now = Utc::now();
        let start_of_month = chrono::NaiveDate::from_ymd_opt(now.year(), now.month(), 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();

        let stats = sqlx::query!(
            r#"
            SELECT 
                COUNT(*) as total_usage,
                COUNT(CASE WHEN feature_type = 'scenario' THEN 1 END) as scenario_usage,
                COUNT(CASE WHEN feature_type = 'content' THEN 1 END) as content_usage,
                COUNT(CASE WHEN feature_type = 'subject' THEN 1 END) as subject_usage
            FROM ai_usage_logs
            WHERE user_id = $1 AND created_at >= $2
            "#,
            user_id,
            start_of_month
        )
        .fetch_one(pool)
        .await?;

        Ok(AiUsageStats {
            total_usage: stats.total_usage.unwrap_or(0),
            scenario_usage: stats.scenario_usage.unwrap_or(0),
            content_usage: stats.content_usage.unwrap_or(0),
            subject_usage: stats.subject_usage.unwrap_or(0),
        })
    }

    /// AI機能の使用制限をチェック
    pub async fn check_ai_usage_limit(
        pool: &PgPool,
        user_id: Uuid,
        feature_type: &str,
        plan: &SubscriptionPlan,
    ) -> Result<bool, Error> {
        let stats = Self::get_monthly_usage(pool, user_id).await?;

        // 無制限プランの場合は常にtrue
        if plan.ai_monthly_limit.is_none() {
            return Ok(true);
        }

        let monthly_limit = plan.ai_monthly_limit.unwrap() as i64;
        let feature_limit = match feature_type {
            "scenario" => plan.ai_scenario_limit.unwrap_or(monthly_limit as i32) as i64,
            "content" => plan.ai_content_limit.unwrap_or(monthly_limit as i32) as i64,
            "subject" => plan.ai_subject_limit.unwrap_or(monthly_limit as i32) as i64,
            _ => monthly_limit,
        };

        let current_usage = match feature_type {
            "scenario" => stats.scenario_usage,
            "content" => stats.content_usage,
            "subject" => stats.subject_usage,
            _ => stats.total_usage,
        };

        Ok(current_usage < feature_limit && stats.total_usage < monthly_limit)
    }

    /// 使用履歴を取得
    pub async fn get_usage_history(
        pool: &PgPool,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AiUsageLog>, Error> {
        let logs = sqlx::query_as!(
            AiUsageLog,
            r#"
            SELECT id, user_id, feature_type, prompt, response, tokens_used, model_used, created_at as "created_at!"
            FROM ai_usage_logs
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

        Ok(logs)
    }
}

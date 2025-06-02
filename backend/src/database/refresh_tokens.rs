use chrono::{Duration, Utc};
use sqlx::{PgPool, Result};
use uuid::Uuid;

/// リフレッシュトークンを保存
pub async fn save_refresh_token(
    pool: &PgPool,
    user_id: Uuid,
    token: &str,
    expires_in_days: i64,
) -> Result<()> {
    let expires_at = Utc::now() + Duration::days(expires_in_days);

    sqlx::query!(
        r#"
        INSERT INTO refresh_tokens (user_id, token, expires_at)
        VALUES ($1, $2, $3)
        "#,
        user_id,
        token,
        expires_at
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// リフレッシュトークンを検証し、ユーザーIDを取得
pub async fn verify_refresh_token(pool: &PgPool, token: &str) -> Result<Option<Uuid>> {
    let result = sqlx::query!(
        r#"
        SELECT user_id
        FROM refresh_tokens
        WHERE token = $1 AND expires_at > NOW()
        "#,
        token
    )
    .fetch_optional(pool)
    .await?;

    Ok(result.map(|r| r.user_id))
}

/// リフレッシュトークンを削除（ログアウト時など）
pub async fn delete_refresh_token(pool: &PgPool, token: &str) -> Result<()> {
    sqlx::query!(
        r#"
        DELETE FROM refresh_tokens
        WHERE token = $1
        "#,
        token
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// ユーザーの全てのリフレッシュトークンを削除
#[allow(dead_code)]
pub async fn delete_user_refresh_tokens(pool: &PgPool, user_id: Uuid) -> Result<()> {
    sqlx::query!(
        r#"
        DELETE FROM refresh_tokens
        WHERE user_id = $1
        "#,
        user_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// 期限切れのリフレッシュトークンを削除（定期的なクリーンアップ用）
#[allow(dead_code)]
pub async fn cleanup_expired_tokens(pool: &PgPool) -> Result<u64> {
    let result = sqlx::query!(
        r#"
        DELETE FROM refresh_tokens
        WHERE expires_at < NOW()
        "#
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

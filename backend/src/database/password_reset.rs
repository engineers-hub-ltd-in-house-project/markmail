use chrono::{DateTime, Duration, Utc};
use sqlx::{postgres::PgRow, PgPool, Result, Row};
use uuid::Uuid;

pub struct PasswordResetToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub used: bool,
    pub created_at: DateTime<Utc>,
}

/// Create a new password reset token for a user
pub async fn create_password_reset_token(
    pool: &PgPool,
    user_id: Uuid,
    token: &str,
    expires_at: DateTime<Utc>,
) -> Result<PasswordResetToken> {
    // First, invalidate any existing unused tokens for this user
    sqlx::query(
        "UPDATE password_reset_tokens 
         SET used = TRUE 
         WHERE user_id = $1 AND used = FALSE",
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    // Create new token
    let result = sqlx::query(
        "INSERT INTO password_reset_tokens (user_id, token, expires_at)
         VALUES ($1, $2, $3)
         RETURNING id, user_id, token, expires_at, used, created_at",
    )
    .bind(user_id)
    .bind(token)
    .bind(expires_at)
    .map(|row: PgRow| PasswordResetToken {
        id: row.get("id"),
        user_id: row.get("user_id"),
        token: row.get("token"),
        expires_at: row.get("expires_at"),
        used: row.get("used"),
        created_at: row.get("created_at"),
    })
    .fetch_one(pool)
    .await?;

    Ok(result)
}

/// Get a password reset token by token string
pub async fn get_password_reset_token(
    pool: &PgPool,
    token: &str,
) -> Result<Option<PasswordResetToken>> {
    let result = sqlx::query(
        "SELECT id, user_id, token, expires_at, used, created_at
         FROM password_reset_tokens
         WHERE token = $1",
    )
    .bind(token)
    .map(|row: PgRow| PasswordResetToken {
        id: row.get("id"),
        user_id: row.get("user_id"),
        token: row.get("token"),
        expires_at: row.get("expires_at"),
        used: row.get("used"),
        created_at: row.get("created_at"),
    })
    .fetch_optional(pool)
    .await?;

    Ok(result)
}

/// Mark a password reset token as used
pub async fn mark_token_as_used(pool: &PgPool, token_id: Uuid) -> Result<()> {
    sqlx::query(
        "UPDATE password_reset_tokens
         SET used = TRUE
         WHERE id = $1",
    )
    .bind(token_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Delete expired tokens (cleanup job)
pub async fn delete_expired_tokens(pool: &PgPool) -> Result<u64> {
    let result = sqlx::query(
        "DELETE FROM password_reset_tokens
         WHERE expires_at < CURRENT_TIMESTAMP OR used = TRUE",
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

/// Check if a user has a recent password reset request (rate limiting)
pub async fn has_recent_reset_request(
    pool: &PgPool,
    user_id: Uuid,
    minutes_ago: i64,
) -> Result<bool> {
    let cutoff_time = Utc::now() - Duration::minutes(minutes_ago);

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) 
         FROM password_reset_tokens
         WHERE user_id = $1 AND created_at > $2",
    )
    .bind(user_id)
    .bind(cutoff_time)
    .fetch_one(pool)
    .await?;

    Ok(count > 0)
}

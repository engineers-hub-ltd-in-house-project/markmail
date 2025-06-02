use sqlx::{PgPool, Result};
use uuid::Uuid;

use crate::models::user::User;

/// メールアドレスでユーザーを検索
pub async fn find_user_by_email(pool: &PgPool, email: &str) -> Result<Option<User>> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, email, password_hash, name, avatar_url, is_active, email_verified, created_at, updated_at
        FROM users
        WHERE email = $1
        "#,
        email
    )
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

/// IDでユーザーを検索
pub async fn find_user_by_id(pool: &PgPool, user_id: Uuid) -> Result<Option<User>> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, email, password_hash, name, avatar_url, is_active, email_verified, created_at, updated_at
        FROM users
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

/// 新しいユーザーを作成
pub async fn create_user(
    pool: &PgPool,
    email: &str,
    password_hash: &str,
    name: &str,
) -> Result<User> {
    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (email, password_hash, name)
        VALUES ($1, $2, $3)
        RETURNING id, email, password_hash, name, avatar_url, is_active, email_verified, created_at, updated_at
        "#,
        email,
        password_hash,
        name
    )
    .fetch_one(pool)
    .await?;

    Ok(user)
}

/// ユーザー情報を更新
pub async fn update_user_profile(
    pool: &PgPool,
    user_id: Uuid,
    name: Option<&str>,
    avatar_url: Option<&str>,
) -> Result<User> {
    let user = sqlx::query_as!(
        User,
        r#"
        UPDATE users
        SET name = COALESCE($2, name),
            avatar_url = COALESCE($3, avatar_url),
            updated_at = NOW()
        WHERE id = $1
        RETURNING id, email, password_hash, name, avatar_url, is_active, email_verified, created_at, updated_at
        "#,
        user_id,
        name,
        avatar_url
    )
    .fetch_one(pool)
    .await?;

    Ok(user)
}

/// メールアドレスの重複をチェック
pub async fn email_exists(pool: &PgPool, email: &str) -> Result<bool> {
    let exists = sqlx::query!(
        r#"
        SELECT EXISTS(SELECT 1 FROM users WHERE email = $1) as "exists!"
        "#,
        email
    )
    .fetch_one(pool)
    .await?
    .exists;

    Ok(exists)
}

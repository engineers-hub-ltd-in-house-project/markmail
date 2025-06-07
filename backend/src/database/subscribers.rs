use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::subscriber::{
    CreateSubscriberRequest, ListSubscriberOptions, Subscriber, SubscriberStatus,
    UpdateSubscriberRequest,
};

/// 購読者一覧を取得（オプション指定版）
pub async fn list_user_subscribers(
    pool: &PgPool,
    user_id: Uuid,
    options: &ListSubscriberOptions,
) -> Result<Vec<Subscriber>, sqlx::Error> {
    let status = options.status.as_ref().and_then(|s| match s.as_str() {
        "active" => Some(SubscriberStatus::Active),
        "unsubscribed" => Some(SubscriberStatus::Unsubscribed),
        "bounced" => Some(SubscriberStatus::Bounced),
        "complained" => Some(SubscriberStatus::Complained),
        _ => None,
    });

    list_subscribers(
        pool,
        user_id,
        options.limit,
        options.offset,
        status,
        options.search.as_deref(),
        options.tag.as_deref(),
    )
    .await
}

/// 購読者一覧を取得（ユーザー別、ページネーション対応）
pub async fn list_subscribers(
    pool: &PgPool,
    user_id: Uuid,
    limit: Option<i64>,
    offset: Option<i64>,
    status: Option<SubscriberStatus>,
    search: Option<&str>,
    tag: Option<&str>,
) -> Result<Vec<Subscriber>, sqlx::Error> {
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);

    // 基本クエリ
    let mut query_string = r#"
        SELECT 
            id,
            user_id,
            email,
            name,
            status,
            tags,
            custom_fields,
            subscribed_at,
            unsubscribed_at,
            created_at,
            updated_at
        FROM subscribers 
        WHERE user_id = $1 
    "#
    .to_string();

    // フィルタリング条件を追加
    if let Some(status_filter) = status {
        query_string.push_str(&format!("AND status = '{:?}' ", status_filter));
    }

    // タグフィルタリング
    if let Some(tag_filter) = tag {
        query_string.push_str(&format!("AND '{}' = ANY(tags) ", tag_filter));
    }

    // 検索条件
    if let Some(search_term) = search {
        query_string.push_str(&format!(
            "AND (email ILIKE '%{}%' OR name ILIKE '%{}%') ",
            search_term, search_term
        ));
    }

    // ソート順
    query_string.push_str("ORDER BY created_at DESC LIMIT $2 OFFSET $3");

    // クエリ実行
    let query = sqlx::query_as::<_, Subscriber>(&query_string)
        .bind(user_id)
        .bind(limit)
        .bind(offset);

    let subscribers = query.fetch_all(pool).await?;
    Ok(subscribers)
}

/// 購読者総数を取得（ユーザー別、フィルタリング対応）
pub async fn count_subscribers(
    pool: &PgPool,
    user_id: Uuid,
    status: Option<SubscriberStatus>,
    search: Option<&str>,
    tag: Option<&str>,
) -> Result<i64, sqlx::Error> {
    // 基本クエリ
    let mut query_string =
        "SELECT COUNT(*) as count FROM subscribers WHERE user_id = $1 ".to_string();

    // フィルタリング条件を追加
    if let Some(status_filter) = status {
        query_string.push_str(&format!("AND status = '{:?}' ", status_filter));
    }

    // タグフィルタリング
    if let Some(tag_filter) = tag {
        query_string.push_str(&format!("AND '{}' = ANY(tags) ", tag_filter));
    }

    // 検索条件
    if let Some(search_term) = search {
        query_string.push_str(&format!(
            "AND (email ILIKE '%{}%' OR name ILIKE '%{}%') ",
            search_term, search_term
        ));
    }

    // クエリ実行
    let count = sqlx::query_scalar::<_, i64>(&query_string)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

    Ok(count)
}

/// 購読者を取得（ID指定）
pub async fn find_subscriber_by_id(
    pool: &PgPool,
    subscriber_id: Uuid,
    user_id: Uuid,
) -> Result<Option<Subscriber>, sqlx::Error> {
    let subscriber = sqlx::query_as::<_, Subscriber>(
        r#"
        SELECT 
            id,
            user_id,
            email,
            name,
            status,
            tags,
            custom_fields,
            subscribed_at,
            unsubscribed_at,
            created_at,
            updated_at
        FROM subscribers 
        WHERE id = $1 AND user_id = $2
        "#,
    )
    .bind(subscriber_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(subscriber)
}

/// 購読者を作成
pub async fn create_subscriber(
    pool: &PgPool,
    user_id: Uuid,
    request: &CreateSubscriberRequest,
) -> Result<Subscriber, sqlx::Error> {
    let custom_fields = request
        .custom_fields
        .clone()
        .unwrap_or_else(|| Value::Object(serde_json::Map::new()));

    let subscriber = sqlx::query_as::<_, Subscriber>(
        r#"
        INSERT INTO subscribers (
            user_id, 
            email, 
            name, 
            status,
            tags,
            custom_fields,
            subscribed_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING 
            id,
            user_id,
            email,
            name,
            status,
            tags,
            custom_fields,
            subscribed_at,
            unsubscribed_at,
            created_at,
            updated_at
        "#,
    )
    .bind(user_id)
    .bind(&request.email)
    .bind(&request.name)
    .bind(request.status.unwrap_or(SubscriberStatus::Active))
    .bind(request.tags.clone().unwrap_or_default())
    .bind(custom_fields)
    .bind(chrono::Utc::now())
    .fetch_one(pool)
    .await?;

    Ok(subscriber)
}

/// 購読者を更新
pub async fn update_subscriber(
    pool: &PgPool,
    subscriber_id: Uuid,
    user_id: Uuid,
    request: &UpdateSubscriberRequest,
) -> Result<Option<Subscriber>, sqlx::Error> {
    // トランザクション開始
    let mut tx = pool.begin().await?;

    // 既存レコードの存在確認
    let existing = sqlx::query_scalar::<_, Option<Uuid>>(
        "SELECT id FROM subscribers WHERE id = $1 AND user_id = $2",
    )
    .bind(subscriber_id)
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await?;

    if existing.is_none() {
        tx.rollback().await?;
        return Ok(None);
    }

    // 状態がUnsubscribedに変更された場合、unsubscribed_atを設定
    let unsubscribed_at = if let Some(SubscriberStatus::Unsubscribed) = request.status {
        Some(chrono::Utc::now())
    } else {
        None
    };

    // 更新クエリ実行
    let subscriber = sqlx::query_as::<_, Subscriber>(
        r#"
        UPDATE subscribers 
        SET 
            email = COALESCE($3, email),
            name = COALESCE($4, name),
            status = COALESCE($5, status),
            tags = COALESCE($6, tags),
            custom_fields = COALESCE($7, custom_fields),
            unsubscribed_at = COALESCE($8, unsubscribed_at),
            updated_at = NOW()
        WHERE id = $1 AND user_id = $2
        RETURNING 
            id,
            user_id,
            email,
            name,
            status,
            tags,
            custom_fields,
            subscribed_at,
            unsubscribed_at,
            created_at,
            updated_at
        "#,
    )
    .bind(subscriber_id)
    .bind(user_id)
    .bind(&request.email)
    .bind(&request.name)
    .bind(request.status)
    .bind(&request.tags)
    .bind(&request.custom_fields)
    .bind(unsubscribed_at)
    .fetch_optional(&mut *tx)
    .await?;

    // トランザクションをコミット
    tx.commit().await?;

    Ok(subscriber)
}

/// 購読者を削除
pub async fn delete_subscriber(
    pool: &PgPool,
    subscriber_id: Uuid,
    user_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM subscribers WHERE id = $1 AND user_id = $2")
        .bind(subscriber_id)
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

/// メールアドレスで購読者を検索（重複チェック用）
pub async fn find_by_email(
    pool: &PgPool,
    email: &str,
    user_id: Uuid,
) -> Result<Option<Subscriber>, sqlx::Error> {
    let subscriber = sqlx::query_as::<_, Subscriber>(
        r#"
        SELECT 
            id,
            user_id,
            email,
            name,
            status,
            tags,
            custom_fields,
            subscribed_at,
            unsubscribed_at,
            created_at,
            updated_at
        FROM subscribers 
        WHERE email = $1 AND user_id = $2
        "#,
    )
    .bind(email)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(subscriber)
}

/// ユーザーの購読者からタグ一覧を取得
pub async fn get_all_tags(pool: &PgPool, user_id: Uuid) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT DISTINCT UNNEST(tags) as tag
        FROM subscribers
        WHERE user_id = $1 AND array_length(tags, 1) > 0
        ORDER BY tag
        "#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    let tags = rows.into_iter().filter_map(|row| row.tag).collect();

    Ok(tags)
}

/// 一括インポート用関数
pub async fn bulk_import_subscribers(
    pool: &PgPool,
    user_id: Uuid,
    subscribers: &[CreateSubscriberRequest],
) -> Result<(u32, Vec<String>), sqlx::Error> {
    let mut tx = pool.begin().await?;
    let mut imported_count = 0;
    let mut errors = Vec::new();

    for (index, sub) in subscribers.iter().enumerate() {
        // メールアドレスが既に存在するか確認
        let existing = sqlx::query_scalar::<_, Option<Uuid>>(
            "SELECT id FROM subscribers WHERE user_id = $1 AND email = $2",
        )
        .bind(user_id)
        .bind(&sub.email)
        .fetch_optional(&mut *tx)
        .await?;

        // 既存のメールアドレスがある場合はスキップ
        if existing.is_some() {
            errors.push(format!(
                "行 {}: メールアドレス '{}' は既に登録されています",
                index + 1,
                sub.email
            ));
            continue;
        }

        // 新規購読者を追加
        let custom_fields = sub
            .custom_fields
            .clone()
            .unwrap_or_else(|| Value::Object(serde_json::Map::new()));
        let tags = sub.tags.clone().unwrap_or_default();
        let status = sub.status.unwrap_or(SubscriberStatus::Active);

        let result = sqlx::query(
            r#"
            INSERT INTO subscribers (
                user_id, 
                email, 
                name, 
                status,
                tags,
                custom_fields,
                subscribed_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(user_id)
        .bind(&sub.email)
        .bind(&sub.name)
        .bind(status)
        .bind(&tags)
        .bind(custom_fields)
        .bind(chrono::Utc::now())
        .execute(&mut *tx)
        .await;

        match result {
            Ok(_) => imported_count += 1,
            Err(e) => errors.push(format!("行 {}: エラー '{}' が発生しました", index + 1, e)),
        }
    }

    // コミット
    tx.commit().await?;

    Ok((imported_count, errors))
}

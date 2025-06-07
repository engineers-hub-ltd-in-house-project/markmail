use sqlx::PgPool;
use uuid::Uuid;

use crate::models::campaign::{
    Campaign, CampaignStatus, CreateCampaignRequest, ListCampaignOptions, UpdateCampaignRequest,
};

/// キャンペーン一覧を取得（ユーザー別）
pub async fn list_user_campaigns(
    pool: &PgPool,
    user_id: Uuid,
    options: &ListCampaignOptions,
) -> Result<Vec<Campaign>, sqlx::Error> {
    let status_filter = match &options.status {
        Some(status) => format!("AND status = '{}'", status),
        None => String::new(),
    };

    // クエリ文字列を構築
    let query_string = format!(
        r#"
        SELECT 
            id,
            user_id,
            template_id,
            name,
            description,
            subject,
            status,
            scheduled_at,
            sent_at,
            recipient_count,
            sent_count,
            opened_count,
            clicked_count,
            created_at,
            updated_at
        FROM campaigns 
        WHERE user_id = $1 {} 
        ORDER BY {} {}
        LIMIT $2 OFFSET $3
        "#,
        status_filter,
        options.sort_by.as_deref().unwrap_or("created_at"),
        options.sort_order.as_deref().unwrap_or("DESC")
    );

    let rows = sqlx::query_as::<_, Campaign>(&query_string)
        .bind(user_id)
        .bind(options.limit.unwrap_or(50))
        .bind(options.offset.unwrap_or(0))
        .fetch_all(pool)
        .await?;

    Ok(rows)
}

/// キャンペーン総数を取得（ユーザー別）
pub async fn count_user_campaigns(
    pool: &PgPool,
    user_id: Uuid,
    status: Option<String>,
) -> Result<i64, sqlx::Error> {
    let status_filter = match status {
        Some(status) => format!("AND status = '{}'", status),
        None => String::new(),
    };

    let query_string = format!(
        "SELECT COUNT(*) as count FROM campaigns WHERE user_id = $1 {}",
        status_filter
    );

    let count = sqlx::query_scalar::<_, i64>(&query_string)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

    Ok(count)
}

/// キャンペーンをIDで取得
pub async fn find_campaign_by_id(
    pool: &PgPool,
    campaign_id: Uuid,
    user_id: Uuid,
) -> Result<Option<Campaign>, sqlx::Error> {
    let row = sqlx::query_as::<_, Campaign>(
        r#"
        SELECT 
            id,
            user_id,
            template_id,
            name,
            description,
            subject,
            status,
            scheduled_at,
            sent_at,
            recipient_count,
            sent_count,
            opened_count,
            clicked_count,
            created_at,
            updated_at
        FROM campaigns 
        WHERE id = $1 AND user_id = $2
        "#,
    )
    .bind(campaign_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

/// キャンペーンを作成
pub async fn create_campaign(
    pool: &PgPool,
    user_id: Uuid,
    request: &CreateCampaignRequest,
) -> Result<Campaign, sqlx::Error> {
    let row = sqlx::query_as::<_, Campaign>(
        r#"
        INSERT INTO campaigns (
            user_id, 
            template_id, 
            name, 
            description, 
            subject,
            status
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING 
            id,
            user_id,
            template_id,
            name,
            description,
            subject,
            status,
            scheduled_at,
            sent_at,
            recipient_count,
            sent_count,
            opened_count,
            clicked_count,
            created_at,
            updated_at
        "#,
    )
    .bind(user_id)
    .bind(request.template_id)
    .bind(&request.name)
    .bind(&request.description)
    .bind(&request.subject)
    .bind(CampaignStatus::Draft.to_string())
    .fetch_one(pool)
    .await?;

    Ok(row)
}

/// キャンペーンを更新
pub async fn update_campaign(
    pool: &PgPool,
    campaign_id: Uuid,
    user_id: Uuid,
    request: &UpdateCampaignRequest,
) -> Result<Option<Campaign>, sqlx::Error> {
    // トランザクション開始
    let mut tx = pool.begin().await?;

    // まずキャンペーンを取得して状態を確認
    let current_campaign = sqlx::query_as::<_, Campaign>(
        r#"
        SELECT 
            id, user_id, template_id, name, description, subject, status, 
            scheduled_at, sent_at, recipient_count, sent_count, opened_count, 
            clicked_count, created_at, updated_at
        FROM campaigns 
        WHERE id = $1 AND user_id = $2
        FOR UPDATE
        "#,
    )
    .bind(campaign_id)
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await?;

    let Some(current_campaign) = current_campaign else {
        // キャンペーンが見つからない場合はトランザクションをロールバックして終了
        tx.rollback().await?;
        return Ok(None);
    };

    // 送信済みまたは送信中のキャンペーンは編集不可
    if matches!(
        current_campaign.status,
        CampaignStatus::Sent | CampaignStatus::Sending
    ) {
        // 送信済み/送信中は変更不可なのでトランザクションをロールバックして終了
        tx.rollback().await?;
        return Ok(Some(current_campaign));
    }

    // 更新クエリ構築（COALESCEを使って値が提供されていない場合は現在の値を使用）
    let row = sqlx::query_as::<_, Campaign>(
        r#"
        UPDATE campaigns 
        SET 
            name = COALESCE($3, name),
            description = COALESCE($4, description),
            subject = COALESCE($5, subject),
            template_id = COALESCE($6, template_id),
            status = COALESCE($7, status),
            scheduled_at = COALESCE($8, scheduled_at),
            updated_at = NOW()
        WHERE id = $1 AND user_id = $2
        RETURNING 
            id, user_id, template_id, name, description, subject, status, 
            scheduled_at, sent_at, recipient_count, sent_count, opened_count, 
            clicked_count, created_at, updated_at
        "#,
    )
    .bind(campaign_id)
    .bind(user_id)
    .bind(request.name.as_deref())
    .bind(request.description.as_deref())
    .bind(request.subject.as_deref())
    .bind(request.template_id)
    .bind(request.status.as_ref().map(|s| s.to_string()))
    .bind(request.scheduled_at)
    .fetch_one(&mut *tx)
    .await?;

    // トランザクションをコミット
    tx.commit().await?;

    Ok(Some(row))
}

/// キャンペーンを削除
pub async fn delete_campaign(
    pool: &PgPool,
    campaign_id: Uuid,
    user_id: Uuid,
) -> Result<bool, sqlx::Error> {
    // トランザクション開始
    let mut tx = pool.begin().await?;

    // まずキャンペーンを取得して状態を確認
    let current_campaign = sqlx::query_as::<_, Campaign>(
        r#"
        SELECT 
            id, user_id, template_id, name, description, subject, status, 
            scheduled_at, sent_at, recipient_count, sent_count, opened_count, 
            clicked_count, created_at, updated_at
        FROM campaigns 
        WHERE id = $1 AND user_id = $2
        FOR UPDATE
        "#,
    )
    .bind(campaign_id)
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await?;

    let Some(current_campaign) = current_campaign else {
        // キャンペーンが見つからない場合はトランザクションをロールバックして終了
        tx.rollback().await?;
        return Ok(false);
    };

    // 送信済みまたは送信中のキャンペーンは削除不可
    if matches!(
        current_campaign.status,
        CampaignStatus::Sent | CampaignStatus::Sending
    ) {
        // 送信済み/送信中は削除不可なのでトランザクションをロールバックして終了
        tx.rollback().await?;
        return Ok(false);
    }

    // 削除実行
    let result = sqlx::query("DELETE FROM campaigns WHERE id = $1 AND user_id = $2")
        .bind(campaign_id)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    // トランザクションをコミット
    tx.commit().await?;

    Ok(result.rows_affected() > 0)
}

/// キャンペーンステータスを更新
#[allow(dead_code)]
pub async fn update_campaign_status(
    pool: &PgPool,
    campaign_id: Uuid,
    user_id: Uuid,
    status: CampaignStatus,
) -> Result<Option<Campaign>, sqlx::Error> {
    // トランザクション開始
    let mut tx = pool.begin().await?;

    // まずキャンペーンを取得して状態を確認
    let current_campaign = sqlx::query_as::<_, Campaign>(
        r#"
        SELECT 
            id, user_id, template_id, name, description, subject, status, 
            scheduled_at, sent_at, recipient_count, sent_count, opened_count, 
            clicked_count, created_at, updated_at
        FROM campaigns 
        WHERE id = $1 AND user_id = $2
        FOR UPDATE
        "#,
    )
    .bind(campaign_id)
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await?;

    let Some(current_campaign) = current_campaign else {
        // キャンペーンが見つからない場合はトランザクションをロールバックして終了
        tx.rollback().await?;
        return Ok(None);
    };

    // 送信済みのキャンペーンは状態変更不可
    if current_campaign.status == CampaignStatus::Sent {
        // 送信済みは状態変更不可なのでトランザクションをロールバックして現在のキャンペーンを返す
        tx.rollback().await?;
        return Ok(Some(current_campaign));
    }

    // 送信中/送信済み時はsent_atを設定
    let sent_at = if status == CampaignStatus::Sending || status == CampaignStatus::Sent {
        Some("NOW()")
    } else {
        None
    };

    let sent_at_sql = match sent_at {
        Some(expr) => format!("sent_at = {}, ", expr),
        None => String::new(),
    };

    // 更新クエリ構築
    let query_string = format!(
        r#"
        UPDATE campaigns 
        SET 
            status = $3,
            {}
            updated_at = NOW()
        WHERE id = $1 AND user_id = $2
        RETURNING 
            id, user_id, template_id, name, description, subject, status, 
            scheduled_at, sent_at, recipient_count, sent_count, opened_count, 
            clicked_count, created_at, updated_at
        "#,
        sent_at_sql
    );

    // 更新実行
    let row = sqlx::query_as::<_, Campaign>(&query_string)
        .bind(campaign_id)
        .bind(user_id)
        .bind(status.to_string())
        .fetch_one(&mut *tx)
        .await?;

    // トランザクションをコミット
    tx.commit().await?;

    Ok(Some(row))
}

/// キャンペーンをスケジュール
pub async fn schedule_campaign(
    pool: &PgPool,
    campaign_id: Uuid,
    user_id: Uuid,
    scheduled_at: chrono::DateTime<chrono::Utc>,
) -> Result<Option<Campaign>, sqlx::Error> {
    // トランザクション開始
    let mut tx = pool.begin().await?;

    // まずキャンペーンを取得して状態を確認
    let current_campaign = sqlx::query_as::<_, Campaign>(
        r#"
        SELECT 
            id, user_id, template_id, name, description, subject, status, 
            scheduled_at, sent_at, recipient_count, sent_count, opened_count, 
            clicked_count, created_at, updated_at
        FROM campaigns 
        WHERE id = $1 AND user_id = $2
        FOR UPDATE
        "#,
    )
    .bind(campaign_id)
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await?;

    let Some(current_campaign) = current_campaign else {
        // キャンペーンが見つからない場合はトランザクションをロールバックして終了
        tx.rollback().await?;
        return Ok(None);
    };

    // 送信済みまたは送信中のキャンペーンはスケジュール変更不可
    if matches!(
        current_campaign.status,
        CampaignStatus::Sent | CampaignStatus::Sending
    ) {
        // 送信済み/送信中はスケジュール変更不可なのでトランザクションをロールバックして現在のキャンペーンを返す
        tx.rollback().await?;
        return Ok(Some(current_campaign));
    }

    // 更新実行
    let row = sqlx::query_as::<_, Campaign>(
        r#"
        UPDATE campaigns 
        SET 
            status = 'scheduled',
            scheduled_at = $3,
            updated_at = NOW()
        WHERE id = $1 AND user_id = $2
        RETURNING 
            id, user_id, template_id, name, description, subject, status, 
            scheduled_at, sent_at, recipient_count, sent_count, opened_count, 
            clicked_count, created_at, updated_at
        "#,
    )
    .bind(campaign_id)
    .bind(user_id)
    .bind(scheduled_at)
    .fetch_one(&mut *tx)
    .await?;

    // トランザクションをコミット
    tx.commit().await?;

    Ok(Some(row))
}

/// キャンペーン送信処理の開始
pub async fn start_campaign_sending(
    pool: &PgPool,
    campaign_id: Uuid,
    user_id: Uuid,
) -> Result<Option<Campaign>, sqlx::Error> {
    // トランザクション開始
    let mut tx = pool.begin().await?;

    // まずキャンペーンを取得して状態を確認
    let current_campaign = sqlx::query_as::<_, Campaign>(
        r#"
        SELECT 
            id, user_id, template_id, name, description, subject, status, 
            scheduled_at, sent_at, recipient_count, sent_count, opened_count, 
            clicked_count, created_at, updated_at
        FROM campaigns 
        WHERE id = $1 AND user_id = $2
        FOR UPDATE
        "#,
    )
    .bind(campaign_id)
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await?;

    let Some(current_campaign) = current_campaign else {
        // キャンペーンが見つからない場合はトランザクションをロールバックして終了
        tx.rollback().await?;
        return Ok(None);
    };

    // 送信済みまたは送信中のキャンペーンは再送信不可
    if matches!(
        current_campaign.status,
        CampaignStatus::Sent | CampaignStatus::Sending
    ) {
        // 送信済み/送信中は再送信不可なのでトランザクションをロールバックして現在のキャンペーンを返す
        tx.rollback().await?;
        return Ok(Some(current_campaign));
    }

    // 更新実行
    let row = sqlx::query_as::<_, Campaign>(
        r#"
        UPDATE campaigns 
        SET 
            status = 'sending',
            sent_at = NOW(),
            updated_at = NOW()
        WHERE id = $1 AND user_id = $2
        RETURNING 
            id, user_id, template_id, name, description, subject, status, 
            scheduled_at, sent_at, recipient_count, sent_count, opened_count, 
            clicked_count, created_at, updated_at
        "#,
    )
    .bind(campaign_id)
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await?;

    // トランザクションをコミット
    tx.commit().await?;

    Ok(Some(row))
}

/// キャンペーンの送信完了
#[allow(dead_code)]
pub async fn complete_campaign_sending(
    pool: &PgPool,
    campaign_id: Uuid,
) -> Result<Option<Campaign>, sqlx::Error> {
    let row = sqlx::query_as::<_, Campaign>(
        r#"
        UPDATE campaigns 
        SET 
            status = 'sent',
            updated_at = NOW()
        WHERE id = $1 AND status = 'sending'
        RETURNING 
            id, user_id, template_id, name, description, subject, status, 
            scheduled_at, sent_at, recipient_count, sent_count, opened_count, 
            clicked_count, created_at, updated_at
        "#,
    )
    .bind(campaign_id)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

/// キャンペーンの統計情報を更新
#[allow(dead_code)]
pub async fn update_campaign_stats(
    pool: &PgPool,
    campaign_id: Uuid,
    recipient_count: Option<i32>,
    sent_count: Option<i32>,
    opened_count: Option<i32>,
    clicked_count: Option<i32>,
) -> Result<Option<Campaign>, sqlx::Error> {
    // 更新クエリ構築
    let mut query_string = "UPDATE campaigns SET ".to_string();
    let mut needs_comma = false;

    if recipient_count.is_some() {
        query_string.push_str("recipient_count = $2");
        needs_comma = true;
    }

    if sent_count.is_some() {
        if needs_comma {
            query_string.push_str(", ");
        }
        query_string.push_str("sent_count = $3");
        needs_comma = true;
    }

    if opened_count.is_some() {
        if needs_comma {
            query_string.push_str(", ");
        }
        query_string.push_str("opened_count = $4");
        needs_comma = true;
    }

    if clicked_count.is_some() {
        if needs_comma {
            query_string.push_str(", ");
        }
        query_string.push_str("clicked_count = $5");
    }

    query_string.push_str(", updated_at = NOW() WHERE id = $1 RETURNING id, user_id, template_id, name, description, subject, status, scheduled_at, sent_at, recipient_count, sent_count, opened_count, clicked_count, created_at, updated_at");

    // クエリ実行
    let mut query = sqlx::query_as::<_, Campaign>(&query_string).bind(campaign_id);

    if let Some(count) = recipient_count {
        query = query.bind(count);
    }

    if let Some(count) = sent_count {
        query = query.bind(count);
    }

    if let Some(count) = opened_count {
        query = query.bind(count);
    }

    if let Some(count) = clicked_count {
        query = query.bind(count);
    }

    let row = query.fetch_optional(pool).await?;

    Ok(row)
}

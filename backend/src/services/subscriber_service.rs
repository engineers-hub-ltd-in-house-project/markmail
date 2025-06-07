use anyhow::Result;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::database::subscribers;
use crate::models::subscriber::{
    CreateSubscriberRequest, ImportSubscribersRequest, ImportSubscribersResponse, Subscriber,
    SubscriberListResponse, SubscriberStatus, UpdateSubscriberRequest,
};

/// 購読者一覧を取得（フィルタリング、ページネーション対応）
pub async fn list_subscribers(
    pool: &PgPool,
    user_id: Uuid,
    limit: Option<i64>,
    offset: Option<i64>,
    status: Option<SubscriberStatus>,
    search: Option<&str>,
    tag: Option<&str>,
) -> Result<SubscriberListResponse> {
    // 購読者データを取得
    let subscribers =
        subscribers::list_subscribers(pool, user_id, limit, offset, status, search, tag).await?;

    // 総数を取得
    let total = subscribers::count_subscribers(pool, user_id, status, search, tag).await?;

    // タグ一覧を取得
    let available_tags = subscribers::get_all_tags(pool, user_id).await?;

    Ok(SubscriberListResponse {
        subscribers,
        total,
        available_tags,
    })
}

/// 購読者を取得（ID指定）
pub async fn get_subscriber(
    pool: &PgPool,
    subscriber_id: Uuid,
    user_id: Uuid,
) -> Result<Option<Subscriber>> {
    let subscriber = subscribers::find_subscriber_by_id(pool, subscriber_id, user_id).await?;
    Ok(subscriber)
}

/// 購読者を作成
pub async fn create_subscriber(
    pool: &PgPool,
    user_id: Uuid,
    request: CreateSubscriberRequest,
) -> Result<Subscriber> {
    // 同じメールアドレスが既に登録されていないか確認
    let existing = subscribers::find_by_email(pool, &request.email, user_id).await?;
    if existing.is_some() {
        anyhow::bail!("このメールアドレスは既に登録されています");
    }

    // 購読者を作成
    let subscriber = subscribers::create_subscriber(pool, user_id, &request).await?;
    Ok(subscriber)
}

/// 購読者を更新
pub async fn update_subscriber(
    pool: &PgPool,
    subscriber_id: Uuid,
    user_id: Uuid,
    request: UpdateSubscriberRequest,
) -> Result<Option<Subscriber>> {
    // メールアドレス変更時の重複チェック
    if let Some(email) = &request.email {
        let existing = subscribers::find_by_email(pool, email, user_id).await?;
        if let Some(existing) = existing {
            if existing.id != subscriber_id {
                anyhow::bail!("このメールアドレスは既に別の購読者に登録されています");
            }
        }
    }

    // 購読者を更新
    let subscriber = subscribers::update_subscriber(pool, subscriber_id, user_id, &request).await?;
    Ok(subscriber)
}

/// 購読者を削除
pub async fn delete_subscriber(pool: &PgPool, subscriber_id: Uuid, user_id: Uuid) -> Result<bool> {
    let result = subscribers::delete_subscriber(pool, subscriber_id, user_id).await?;
    Ok(result)
}

/// CSVファイルから購読者をインポート
pub async fn import_subscribers_from_csv(
    pool: &PgPool,
    user_id: Uuid,
    request: ImportSubscribersRequest,
) -> Result<ImportSubscribersResponse> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(request.has_header.unwrap_or(true))
        .from_reader(request.csv_content.as_bytes());

    let mut subscribers = Vec::new();
    let mut errors = Vec::new();
    let mapping = &request.column_mapping;

    for (index, result) in reader.records().enumerate() {
        let record = match result {
            Ok(record) => record,
            Err(e) => {
                errors.push(format!("行 {}: CSVフォーマットエラー - {}", index + 1, e));
                continue;
            }
        };

        // 必須フィールド: メールアドレス
        let email = match record.get(mapping.email) {
            Some(email) if !email.trim().is_empty() => email.trim().to_string(),
            _ => {
                errors.push(format!("行 {}: メールアドレスは必須です", index + 1));
                continue;
            }
        };

        // オプションフィールド: 名前
        let name = mapping
            .name
            .and_then(|idx| record.get(idx))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        // タグ
        let tags = match &mapping.tags {
            Some(tag_indices) => {
                let mut tags = Vec::new();
                for &idx in tag_indices {
                    if let Some(tag) = record.get(idx) {
                        let tag = tag.trim();
                        if !tag.is_empty() {
                            tags.push(tag.to_string());
                        }
                    }
                }
                Some(tags)
            }
            None => None,
        };

        // カスタムフィールド
        let custom_fields = match &mapping.custom_fields {
            Some(field_mappings) => {
                let mut fields = serde_json::Map::new();
                for mapping in field_mappings {
                    if let Some(value) = record.get(mapping.column) {
                        let value = value.trim();
                        if !value.is_empty() {
                            fields.insert(mapping.name.clone(), Value::String(value.to_string()));
                        }
                    }
                }
                Some(Value::Object(fields))
            }
            None => None,
        };

        // 購読者オブジェクトを作成
        subscribers.push(CreateSubscriberRequest {
            email,
            name,
            status: Some(SubscriberStatus::Active),
            tags,
            custom_fields,
        });
    }

    // 一括インポート
    let (imported_count, import_errors) =
        subscribers::bulk_import_subscribers(pool, user_id, &subscribers).await?;
    errors.extend(import_errors);

    Ok(ImportSubscribersResponse {
        imported_count,
        errors,
    })
}

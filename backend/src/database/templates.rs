use sqlx::PgPool;
use uuid::Uuid;

use crate::models::template::{CreateTemplateRequest, Template, UpdateTemplateRequest};

/// テンプレート一覧を取得（ユーザー別）
pub async fn list_user_templates(
    pool: &PgPool,
    user_id: Uuid,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<Template>, sqlx::Error> {
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);

    let rows = sqlx::query!(
        r#"
        SELECT 
            id,
            user_id,
            name,
            subject_template,
            markdown_content,
            html_content,
            variables,
            is_public,
            created_at,
            updated_at
        FROM templates 
        WHERE user_id = $1 
        ORDER BY updated_at DESC 
        LIMIT $2 OFFSET $3
        "#,
        user_id,
        limit,
        offset
    )
    .fetch_all(pool)
    .await?;

    let templates = rows
        .into_iter()
        .map(|row| Template {
            id: row.id,
            user_id: row.user_id,
            name: row.name,
            subject_template: row.subject_template,
            markdown_content: row.markdown_content,
            html_content: row.html_content,
            variables: row.variables.unwrap_or_else(|| serde_json::json!({})),
            is_public: row.is_public.unwrap_or(false),
            created_at: row.created_at.unwrap_or_else(chrono::Utc::now),
            updated_at: row.updated_at.unwrap_or_else(chrono::Utc::now),
        })
        .collect();

    Ok(templates)
}

/// テンプレート総数を取得（ユーザー別）
pub async fn count_user_templates(pool: &PgPool, user_id: Uuid) -> Result<i64, sqlx::Error> {
    let count = sqlx::query!(
        "SELECT COUNT(*) as count FROM templates WHERE user_id = $1",
        user_id
    )
    .fetch_one(pool)
    .await?
    .count
    .unwrap_or(0);

    Ok(count)
}

/// テンプレートをIDで取得
pub async fn find_template_by_id(
    pool: &PgPool,
    template_id: Uuid,
    user_id: Option<Uuid>,
) -> Result<Option<Template>, sqlx::Error> {
    if let Some(user_id) = user_id {
        // ユーザー指定時は所有者チェック
        let row = sqlx::query!(
            r#"
            SELECT 
                id,
                user_id,
                name,
                subject_template,
                markdown_content,
                html_content,
                variables,
                is_public,
                created_at,
                updated_at
            FROM templates 
            WHERE id = $1 AND (user_id = $2 OR is_public = true)
            "#,
            template_id,
            user_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(row.map(|row| Template {
            id: row.id,
            user_id: row.user_id,
            name: row.name,
            subject_template: row.subject_template,
            markdown_content: row.markdown_content,
            html_content: row.html_content,
            variables: row.variables.unwrap_or_else(|| serde_json::json!({})),
            is_public: row.is_public.unwrap_or(false),
            created_at: row.created_at.unwrap_or_else(chrono::Utc::now),
            updated_at: row.updated_at.unwrap_or_else(chrono::Utc::now),
        }))
    } else {
        // 公開テンプレートのみ
        let row = sqlx::query!(
            r#"
            SELECT 
                id,
                user_id,
                name,
                subject_template,
                markdown_content,
                html_content,
                variables,
                is_public,
                created_at,
                updated_at
            FROM templates 
            WHERE id = $1 AND is_public = true
            "#,
            template_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(row.map(|row| Template {
            id: row.id,
            user_id: row.user_id,
            name: row.name,
            subject_template: row.subject_template,
            markdown_content: row.markdown_content,
            html_content: row.html_content,
            variables: row.variables.unwrap_or_else(|| serde_json::json!({})),
            is_public: row.is_public.unwrap_or(false),
            created_at: row.created_at.unwrap_or_else(chrono::Utc::now),
            updated_at: row.updated_at.unwrap_or_else(chrono::Utc::now),
        }))
    }
}

/// テンプレートを作成
pub async fn create_template(
    pool: &PgPool,
    user_id: Uuid,
    request: &CreateTemplateRequest,
) -> Result<Template, sqlx::Error> {
    let default_variables = serde_json::json!({});
    let variables = request.variables.as_ref().unwrap_or(&default_variables);

    let row = sqlx::query!(
        r#"
        INSERT INTO templates (
            user_id, 
            name, 
            subject_template, 
            markdown_content, 
            variables,
            is_public
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING 
            id,
            user_id,
            name,
            subject_template,
            markdown_content,
            html_content,
            variables,
            is_public,
            created_at,
            updated_at
        "#,
        user_id,
        request.name,
        request.subject_template,
        request.markdown_content,
        variables,
        request.is_public.unwrap_or(false)
    )
    .fetch_one(pool)
    .await?;

    Ok(Template {
        id: row.id,
        user_id: row.user_id,
        name: row.name,
        subject_template: row.subject_template,
        markdown_content: row.markdown_content,
        html_content: row.html_content,
        variables: row.variables.unwrap_or_else(|| serde_json::json!({})),
        is_public: row.is_public.unwrap_or(false),
        created_at: row.created_at.unwrap_or_else(chrono::Utc::now),
        updated_at: row.updated_at.unwrap_or_else(chrono::Utc::now),
    })
}

/// テンプレートを更新
pub async fn update_template(
    pool: &PgPool,
    template_id: Uuid,
    user_id: Uuid,
    request: &UpdateTemplateRequest,
) -> Result<Option<Template>, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        UPDATE templates 
        SET 
            name = COALESCE($3, name),
            subject_template = COALESCE($4, subject_template),
            markdown_content = COALESCE($5, markdown_content),
            html_content = COALESCE($6, html_content),
            variables = COALESCE($7, variables),
            is_public = COALESCE($8, is_public),
            updated_at = NOW()
        WHERE id = $1 AND user_id = $2
        RETURNING 
            id,
            user_id,
            name,
            subject_template,
            markdown_content,
            html_content,
            variables,
            is_public,
            created_at,
            updated_at
        "#,
        template_id,
        user_id,
        request.name.as_deref(),
        request.subject_template.as_deref(),
        request.markdown_content.as_deref(),
        request.html_content.as_deref(),
        request.variables.as_ref(),
        request.is_public
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|row| Template {
        id: row.id,
        user_id: row.user_id,
        name: row.name,
        subject_template: row.subject_template,
        markdown_content: row.markdown_content,
        html_content: row.html_content,
        variables: row.variables.unwrap_or_else(|| serde_json::json!({})),
        is_public: row.is_public.unwrap_or(false),
        created_at: row.created_at.unwrap_or_else(chrono::Utc::now),
        updated_at: row.updated_at.unwrap_or_else(chrono::Utc::now),
    }))
}

/// テンプレートを削除
pub async fn delete_template(
    pool: &PgPool,
    template_id: Uuid,
    user_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        "DELETE FROM templates WHERE id = $1 AND user_id = $2",
        template_id,
        user_id
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// HTMLコンテンツを更新（キャッシュ用）
pub async fn update_html_content(
    pool: &PgPool,
    template_id: Uuid,
    html_content: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE templates SET html_content = $2 WHERE id = $1",
        template_id,
        html_content
    )
    .execute(pool)
    .await?;

    Ok(())
}

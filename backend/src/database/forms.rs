use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::form::{CreateFormRequest, Form, FormSubmission, UpdateFormRequest};

pub async fn create_form(pool: &PgPool, user_id: Uuid, request: CreateFormRequest) -> Result<Form> {
    let slug = request.slug.unwrap_or_else(|| {
        // Generate slug from name
        request.name.to_lowercase().replace(' ', "-")
    });

    let form = sqlx::query_as!(
        Form,
        r#"
        INSERT INTO forms (user_id, name, description, slug, markdown_content, form_fields, settings, status)
        VALUES ($1, $2, $3, $4, $5, $6, $7, 'draft')
        RETURNING id, user_id, name, description, slug, markdown_content, form_fields, settings, status, submission_count, created_at, updated_at
        "#,
        user_id,
        request.name,
        request.description,
        slug,
        request.markdown_content,
        request.form_fields.unwrap_or(serde_json::json!([])),
        request.settings.unwrap_or(serde_json::json!({}))
    )
    .fetch_one(pool)
    .await?;

    Ok(form)
}

pub async fn get_form_by_id(pool: &PgPool, form_id: Uuid) -> Result<Option<Form>> {
    let form = sqlx::query_as!(
        Form,
        r#"
        SELECT id, user_id, name, description, slug, markdown_content, form_fields, settings, status, submission_count, created_at, updated_at
        FROM forms
        WHERE id = $1
        "#,
        form_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(form)
}

pub async fn get_forms_by_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<Form>> {
    let forms = sqlx::query_as!(
        Form,
        r#"
        SELECT id, user_id, name, description, slug, markdown_content, form_fields, settings, status, submission_count, created_at, updated_at
        FROM forms
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    Ok(forms)
}

pub async fn update_form(pool: &PgPool, form_id: Uuid, request: UpdateFormRequest) -> Result<Form> {
    let form = sqlx::query_as!(
        Form,
        r#"
        UPDATE forms
        SET name = COALESCE($2, name),
            description = COALESCE($3, description),
            markdown_content = COALESCE($4, markdown_content),
            form_fields = COALESCE($5, form_fields),
            settings = COALESCE($6, settings),
            status = COALESCE($7, status),
            updated_at = NOW()
        WHERE id = $1
        RETURNING id, user_id, name, description, slug, markdown_content, form_fields, settings, status, submission_count, created_at, updated_at
        "#,
        form_id,
        request.name,
        request.description,
        request.markdown_content,
        request.form_fields,
        request.settings,
        request.status
    )
    .fetch_one(pool)
    .await?;

    Ok(form)
}

pub async fn delete_form(pool: &PgPool, form_id: Uuid) -> Result<()> {
    sqlx::query!(
        r#"
        DELETE FROM forms
        WHERE id = $1
        "#,
        form_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn create_form_submission(
    pool: &PgPool,
    form_id: Uuid,
    data: serde_json::Value,
    subscriber_id: Option<Uuid>,
    ip_address: Option<String>,
    user_agent: Option<String>,
    referrer: Option<String>,
) -> Result<FormSubmission> {
    // Generate confirmation token
    let confirmation_token = uuid::Uuid::new_v4().to_string();

    let submission = sqlx::query_as!(
        FormSubmission,
        r#"
        INSERT INTO form_submissions (form_id, subscriber_id, data, ip_address, user_agent, referrer, confirmation_token)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, form_id, subscriber_id, data, ip_address, user_agent, referrer, confirmation_token, confirmed_at, created_at
        "#,
        form_id,
        subscriber_id,
        data,
        ip_address,
        user_agent,
        referrer,
        confirmation_token
    )
    .fetch_one(pool)
    .await?;

    // Update submission count
    sqlx::query!(
        r#"
        UPDATE forms
        SET submission_count = submission_count + 1
        WHERE id = $1
        "#,
        form_id
    )
    .execute(pool)
    .await?;

    Ok(submission)
}

pub async fn get_form_submissions(
    pool: &PgPool,
    form_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<FormSubmission>> {
    let submissions = sqlx::query_as!(
        FormSubmission,
        r#"
        SELECT id, form_id, subscriber_id, data, ip_address, user_agent, referrer, confirmation_token, confirmed_at, created_at
        FROM form_submissions
        WHERE form_id = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        form_id,
        limit,
        offset
    )
    .fetch_all(pool)
    .await?;

    Ok(submissions)
}

pub async fn count_form_submissions(pool: &PgPool, form_id: Uuid) -> Result<i64> {
    let count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as "count!"
        FROM form_submissions
        WHERE form_id = $1
        "#,
        form_id
    )
    .fetch_one(pool)
    .await?;

    Ok(count)
}

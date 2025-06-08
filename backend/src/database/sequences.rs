use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::sequence::{
    CreateSequenceRequest, CreateSequenceStepRequest, Sequence, SequenceEnrollment, SequenceStep,
    SequenceWithSteps, UpdateSequenceRequest, UpdateSequenceStepRequest,
};

pub async fn create_sequence(
    pool: &PgPool,
    user_id: Uuid,
    request: CreateSequenceRequest,
) -> Result<Sequence> {
    let sequence = sqlx::query_as!(
        Sequence,
        r#"
        INSERT INTO sequences (user_id, name, description, trigger_type, trigger_config, status)
        VALUES ($1, $2, $3, $4, $5, 'draft')
        RETURNING id, user_id, name, description, trigger_type, trigger_config, status, active_subscribers, completed_subscribers, created_at, updated_at
        "#,
        user_id,
        request.name,
        request.description,
        request.trigger_type,
        request.trigger_config.unwrap_or(serde_json::json!({}))
    )
    .fetch_one(pool)
    .await?;

    Ok(sequence)
}

pub async fn get_sequence_by_id(pool: &PgPool, sequence_id: Uuid) -> Result<Option<Sequence>> {
    let sequence = sqlx::query_as!(
        Sequence,
        r#"
        SELECT id, user_id, name, description, trigger_type, trigger_config, status, active_subscribers, completed_subscribers, created_at, updated_at
        FROM sequences
        WHERE id = $1
        "#,
        sequence_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(sequence)
}

pub async fn get_sequences_by_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<Sequence>> {
    let sequences = sqlx::query_as!(
        Sequence,
        r#"
        SELECT id, user_id, name, description, trigger_type, trigger_config, status, active_subscribers, completed_subscribers, created_at, updated_at
        FROM sequences
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    Ok(sequences)
}

pub async fn update_sequence(
    pool: &PgPool,
    sequence_id: Uuid,
    request: UpdateSequenceRequest,
) -> Result<Sequence> {
    let sequence = sqlx::query_as!(
        Sequence,
        r#"
        UPDATE sequences
        SET name = COALESCE($2, name),
            description = COALESCE($3, description),
            trigger_type = COALESCE($4, trigger_type),
            trigger_config = COALESCE($5, trigger_config),
            status = COALESCE($6, status),
            updated_at = NOW()
        WHERE id = $1
        RETURNING id, user_id, name, description, trigger_type, trigger_config, status, active_subscribers, completed_subscribers, created_at, updated_at
        "#,
        sequence_id,
        request.name,
        request.description,
        request.trigger_type,
        request.trigger_config,
        request.status
    )
    .fetch_one(pool)
    .await?;

    Ok(sequence)
}

pub async fn delete_sequence(pool: &PgPool, sequence_id: Uuid) -> Result<()> {
    sqlx::query!(
        r#"
        DELETE FROM sequences
        WHERE id = $1
        "#,
        sequence_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn create_sequence_step(
    pool: &PgPool,
    sequence_id: Uuid,
    request: CreateSequenceStepRequest,
) -> Result<SequenceStep> {
    let step = sqlx::query_as!(
        SequenceStep,
        r#"
        INSERT INTO sequence_steps (sequence_id, name, step_order, step_type, delay_value, delay_unit, template_id, subject, conditions, action_config)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id, sequence_id, name, step_order, step_type, delay_value, delay_unit, template_id, subject, conditions, action_config, created_at, updated_at
        "#,
        sequence_id,
        request.name,
        request.step_order,
        request.step_type,
        request.delay_value.unwrap_or(0),
        request.delay_unit.unwrap_or("hours".to_string()),
        request.template_id,
        request.subject,
        request.conditions.unwrap_or(serde_json::json!({})),
        request.action_config.unwrap_or(serde_json::json!({}))
    )
    .fetch_one(pool)
    .await?;

    Ok(step)
}

pub async fn get_sequence_steps(pool: &PgPool, sequence_id: Uuid) -> Result<Vec<SequenceStep>> {
    let steps = sqlx::query_as!(
        SequenceStep,
        r#"
        SELECT id, sequence_id, name, step_order, step_type, delay_value, delay_unit, template_id, subject, conditions, action_config, created_at, updated_at
        FROM sequence_steps
        WHERE sequence_id = $1
        ORDER BY step_order ASC
        "#,
        sequence_id
    )
    .fetch_all(pool)
    .await?;

    Ok(steps)
}

pub async fn update_sequence_step(
    pool: &PgPool,
    step_id: Uuid,
    request: UpdateSequenceStepRequest,
) -> Result<SequenceStep> {
    let step = sqlx::query_as!(
        SequenceStep,
        r#"
        UPDATE sequence_steps
        SET name = COALESCE($2, name),
            step_order = COALESCE($3, step_order),
            step_type = COALESCE($4, step_type),
            delay_value = COALESCE($5, delay_value),
            delay_unit = COALESCE($6, delay_unit),
            template_id = COALESCE($7, template_id),
            subject = COALESCE($8, subject),
            conditions = COALESCE($9, conditions),
            action_config = COALESCE($10, action_config),
            updated_at = NOW()
        WHERE id = $1
        RETURNING id, sequence_id, name, step_order, step_type, delay_value, delay_unit, template_id, subject, conditions, action_config, created_at, updated_at
        "#,
        step_id,
        request.name,
        request.step_order,
        request.step_type,
        request.delay_value,
        request.delay_unit,
        request.template_id,
        request.subject,
        request.conditions,
        request.action_config
    )
    .fetch_one(pool)
    .await?;

    Ok(step)
}

pub async fn delete_sequence_step(pool: &PgPool, step_id: Uuid) -> Result<()> {
    sqlx::query!(
        r#"
        DELETE FROM sequence_steps
        WHERE id = $1
        "#,
        step_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_sequence_with_steps(
    pool: &PgPool,
    sequence_id: Uuid,
) -> Result<Option<SequenceWithSteps>> {
    let sequence = get_sequence_by_id(pool, sequence_id).await?;

    match sequence {
        Some(sequence) => {
            let steps = get_sequence_steps(pool, sequence_id).await?;
            Ok(Some(SequenceWithSteps { sequence, steps }))
        }
        None => Ok(None),
    }
}

pub async fn create_sequence_enrollment(
    pool: &PgPool,
    sequence_id: Uuid,
    subscriber_id: Uuid,
) -> Result<SequenceEnrollment> {
    let enrollment = sqlx::query_as!(
        SequenceEnrollment,
        r#"
        INSERT INTO sequence_enrollments (sequence_id, subscriber_id, status, metadata)
        VALUES ($1, $2, 'active', '{}')
        RETURNING id, sequence_id, subscriber_id, current_step_id, status, enrolled_at, completed_at, cancelled_at, next_step_at, metadata, created_at, updated_at
        "#,
        sequence_id,
        subscriber_id
    )
    .fetch_one(pool)
    .await?;

    Ok(enrollment)
}

pub async fn get_active_sequences_by_trigger(
    pool: &PgPool,
    trigger_type: &str,
) -> Result<Vec<Sequence>> {
    let sequences = sqlx::query_as!(
        Sequence,
        r#"
        SELECT id, user_id, name, description, trigger_type, trigger_config, status, active_subscribers, completed_subscribers, created_at, updated_at
        FROM sequences
        WHERE trigger_type = $1 AND status = 'active'
        "#,
        trigger_type
    )
    .fetch_all(pool)
    .await?;

    Ok(sequences)
}

pub async fn get_sequence_enrollment(
    pool: &PgPool,
    sequence_id: Uuid,
    subscriber_id: Uuid,
) -> Result<Option<SequenceEnrollment>> {
    let enrollment = sqlx::query_as!(
        SequenceEnrollment,
        r#"
        SELECT id, sequence_id, subscriber_id, current_step_id, status, enrolled_at, completed_at, cancelled_at, next_step_at, metadata, created_at, updated_at
        FROM sequence_enrollments
        WHERE sequence_id = $1 AND subscriber_id = $2
        "#,
        sequence_id,
        subscriber_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(enrollment)
}

pub async fn update_sequence_enrollment_status(
    pool: &PgPool,
    enrollment_id: Uuid,
    status: &str,
    current_step_id: Option<Uuid>,
) -> Result<()> {
    if status == "completed" {
        sqlx::query!(
            r#"
            UPDATE sequence_enrollments
            SET status = $2,
                current_step_id = $3,
                completed_at = NOW(),
                updated_at = NOW()
            WHERE id = $1
            "#,
            enrollment_id,
            status,
            current_step_id
        )
        .execute(pool)
        .await?;
    } else if status == "cancelled" {
        sqlx::query!(
            r#"
            UPDATE sequence_enrollments
            SET status = $2,
                current_step_id = $3,
                cancelled_at = NOW(),
                updated_at = NOW()
            WHERE id = $1
            "#,
            enrollment_id,
            status,
            current_step_id
        )
        .execute(pool)
        .await?;
    } else {
        sqlx::query!(
            r#"
            UPDATE sequence_enrollments
            SET status = $2,
                current_step_id = $3,
                updated_at = NOW()
            WHERE id = $1
            "#,
            enrollment_id,
            status,
            current_step_id
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}

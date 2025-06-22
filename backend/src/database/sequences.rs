use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::sequence::{
    CreateSequenceEnrollmentRequest, CreateSequenceRequest, CreateSequenceStepRequest, Sequence,
    SequenceEnrollment, SequenceStep, SequenceStepLog, SequenceStepWithTemplate, SequenceWithSteps,
    SequenceWithStepsAndTemplates, TriggerType, UpdateSequenceRequest, UpdateSequenceStepRequest,
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

pub async fn update_sequence_status(pool: &PgPool, sequence_id: Uuid, status: &str) -> Result<()> {
    sqlx::query!(
        r#"
        UPDATE sequences
        SET status = $2,
            updated_at = NOW()
        WHERE id = $1
        "#,
        sequence_id,
        status
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

// 新しく追加する関数

pub async fn find_active_sequences_by_trigger(
    pool: &PgPool,
    user_id: Uuid,
    trigger_type: TriggerType,
) -> Result<Vec<Sequence>> {
    let sequences = sqlx::query_as!(
        Sequence,
        r#"
        SELECT id, user_id, name, description, trigger_type, trigger_config, status, active_subscribers, completed_subscribers, created_at, updated_at
        FROM sequences
        WHERE user_id = $1 AND trigger_type = $2 AND status = 'active'
        "#,
        user_id,
        trigger_type.as_str()
    )
    .fetch_all(pool)
    .await?;

    Ok(sequences)
}

pub async fn find_sequence_by_id(
    pool: &PgPool,
    sequence_id: Uuid,
    _user_id: Option<Uuid>,
) -> Result<Option<Sequence>> {
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

pub async fn find_sequence_steps(pool: &PgPool, sequence_id: Uuid) -> Result<Vec<SequenceStep>> {
    get_sequence_steps(pool, sequence_id).await
}

pub async fn find_pending_sequence_enrollments(pool: &PgPool) -> Result<Vec<SequenceEnrollment>> {
    let enrollments = sqlx::query_as!(
        SequenceEnrollment,
        r#"
        SELECT id, sequence_id, subscriber_id, current_step_id, status, enrolled_at, completed_at, cancelled_at, next_step_at, metadata, created_at, updated_at
        FROM sequence_enrollments
        WHERE status = 'active' 
        AND (next_step_at IS NULL OR next_step_at <= NOW())
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(enrollments)
}

pub async fn complete_sequence_enrollment(pool: &PgPool, enrollment_id: Uuid) -> Result<()> {
    update_sequence_enrollment_status(pool, enrollment_id, "completed", None).await
}

pub async fn update_enrollment_progress(
    pool: &PgPool,
    enrollment_id: Uuid,
    current_step_order: i32,
) -> Result<()> {
    // 現在のステップIDを取得
    let step = sqlx::query!(
        r#"
        SELECT s.id
        FROM sequence_steps s
        JOIN sequence_enrollments e ON e.sequence_id = s.sequence_id
        WHERE e.id = $1 AND s.step_order = $2
        "#,
        enrollment_id,
        current_step_order
    )
    .fetch_optional(pool)
    .await?;

    let current_step_id = step.map(|s| s.id);

    sqlx::query!(
        r#"
        UPDATE sequence_enrollments
        SET current_step_id = $2,
            next_step_at = NULL,
            updated_at = NOW()
        WHERE id = $1
        "#,
        enrollment_id,
        current_step_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn schedule_next_enrollment_step(
    pool: &PgPool,
    enrollment_id: Uuid,
    next_step_order: i32,
    next_execution_at: DateTime<Utc>,
) -> Result<()> {
    // 次のステップIDを取得
    let step = sqlx::query!(
        r#"
        SELECT s.id
        FROM sequence_steps s
        JOIN sequence_enrollments e ON e.sequence_id = s.sequence_id
        WHERE e.id = $1 AND s.step_order = $2
        "#,
        enrollment_id,
        next_step_order
    )
    .fetch_optional(pool)
    .await?;

    let next_step_id = step.map(|s| s.id);

    sqlx::query!(
        r#"
        UPDATE sequence_enrollments
        SET current_step_id = $2,
            next_step_at = $3,
            updated_at = NOW()
        WHERE id = $1
        "#,
        enrollment_id,
        next_step_id,
        next_execution_at
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn create_sequence_step_log(
    pool: &PgPool,
    enrollment_id: Uuid,
    step_id: Uuid,
    status: &str,
    error_message: Option<String>,
) -> Result<SequenceStepLog> {
    let log = sqlx::query_as!(
        SequenceStepLog,
        r#"
        INSERT INTO sequence_step_logs (enrollment_id, step_id, status, error_message)
        VALUES ($1, $2, $3, $4)
        RETURNING id, enrollment_id, step_id, status, error_message, executed_at
        "#,
        enrollment_id,
        step_id,
        status,
        error_message
    )
    .fetch_one(pool)
    .await?;

    Ok(log)
}

// create_sequence_enrollment関数を更新
pub async fn create_sequence_enrollment(
    pool: &PgPool,
    sequence_id: Uuid,
    request: &CreateSequenceEnrollmentRequest,
) -> Result<SequenceEnrollment> {
    let metadata = request
        .trigger_data
        .clone()
        .unwrap_or(serde_json::json!({}));

    let enrollment = sqlx::query_as!(
        SequenceEnrollment,
        r#"
        INSERT INTO sequence_enrollments (sequence_id, subscriber_id, status, metadata)
        VALUES ($1, $2, 'active', $3)
        RETURNING id, sequence_id, subscriber_id, current_step_id, status, enrolled_at, completed_at, cancelled_at, next_step_at, metadata, created_at, updated_at
        "#,
        sequence_id,
        request.subscriber_id,
        metadata
    )
    .fetch_one(pool)
    .await?;

    Ok(enrollment)
}

pub async fn get_sequence_with_steps_and_templates(
    pool: &PgPool,
    sequence_id: Uuid,
) -> Result<Option<SequenceWithStepsAndTemplates>> {
    let sequence = get_sequence_by_id(pool, sequence_id).await?;

    match sequence {
        Some(sequence) => {
            let user_id = sequence.user_id; // シーケンス所有者のユーザーIDを保存
            let steps = get_sequence_steps(pool, sequence_id).await?;
            let mut steps_with_templates = Vec::new();

            for step in steps {
                let template = if let Some(template_id) = step.template_id {
                    // シーケンス所有者のユーザーIDを使用してテンプレートを取得
                    crate::database::templates::find_template_by_id(
                        pool,
                        template_id,
                        Some(user_id),
                    )
                    .await?
                } else {
                    None
                };

                steps_with_templates.push(SequenceStepWithTemplate {
                    step: step.clone(),
                    template,
                });
            }

            Ok(Some(SequenceWithStepsAndTemplates {
                sequence,
                steps: steps_with_templates,
            }))
        }
        None => Ok(None),
    }
}

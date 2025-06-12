use chrono::{DateTime, Duration, Utc};
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    database::{sequences, subscribers, templates},
    models::{
        sequence::{
            CreateSequenceEnrollmentRequest, Sequence, SequenceEnrollment, SequenceStep,
            SequenceStepLog, TriggerType,
        },
        subscriber::Subscriber,
    },
    services::{
        email_service::{EmailMessage, EmailService},
        markdown_service::MarkdownService,
    },
};

pub struct SequenceService;

impl Default for SequenceService {
    fn default() -> Self {
        Self
    }
}

impl SequenceService {
    pub fn new() -> Self {
        Self
    }

    // シーケンスへの自動エンロールメント処理
    pub async fn process_trigger_enrollment(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        trigger_type: TriggerType,
        subscriber_id: Uuid,
        trigger_data: Option<Value>,
    ) -> Result<Vec<SequenceEnrollment>, String> {
        // 該当するトリガータイプのアクティブなシーケンスを取得
        let sequences = sequences::find_active_sequences_by_trigger(pool, user_id, trigger_type)
            .await
            .map_err(|e| format!("シーケンスの取得に失敗しました: {}", e))?;

        let mut enrollments = Vec::new();

        for sequence in sequences {
            // トリガー条件を評価
            if self.evaluate_trigger_conditions(&sequence, trigger_data.as_ref())? {
                // エンロールメントを作成
                let request = CreateSequenceEnrollmentRequest {
                    subscriber_id,
                    trigger_data: trigger_data.clone(),
                };

                match sequences::create_sequence_enrollment(pool, sequence.id, &request).await {
                    Ok(enrollment) => {
                        tracing::info!(
                            "Subscriber {} enrolled in sequence {} ({})",
                            subscriber_id,
                            sequence.id,
                            sequence.name
                        );
                        enrollments.push(enrollment);
                    }
                    Err(e) => {
                        tracing::error!(
                            "Failed to enroll subscriber {} in sequence {}: {}",
                            subscriber_id,
                            sequence.id,
                            e
                        );
                    }
                }
            }
        }

        Ok(enrollments)
    }

    // トリガー条件の評価
    fn evaluate_trigger_conditions(
        &self,
        sequence: &Sequence,
        trigger_data: Option<&Value>,
    ) -> Result<bool, String> {
        // trigger_configが空の場合は常にtrue
        if sequence.trigger_config.is_null() || sequence.trigger_config == json!({}) {
            return Ok(true);
        }

        // フォーム送信トリガーの場合
        if sequence.trigger_type == TriggerType::FormSubmission.as_str() {
            if let Some(data) = trigger_data {
                if let (Some(form_id_config), Some(form_id_data)) =
                    (sequence.trigger_config.get("form_id"), data.get("form_id"))
                {
                    return Ok(form_id_config == form_id_data);
                }
            }
        }

        // タグ追加トリガーの場合
        if sequence.trigger_type == TriggerType::TagAdded.as_str() {
            if let Some(data) = trigger_data {
                if let (Some(tag_config), Some(tag_data)) =
                    (sequence.trigger_config.get("tag"), data.get("tag"))
                {
                    return Ok(tag_config == tag_data);
                }
            }
        }

        // その他のトリガーは現時点では常にtrue
        Ok(true)
    }

    // 実行待ちのシーケンスステップを処理
    pub async fn process_pending_sequence_steps(&self, pool: &PgPool) -> Result<(), String> {
        // 実行待ちのエンロールメントを取得
        let pending_enrollments = sequences::find_pending_sequence_enrollments(pool)
            .await
            .map_err(|e| format!("実行待ちエンロールメントの取得に失敗しました: {}", e))?;

        for enrollment in pending_enrollments {
            if let Err(e) = self.process_enrollment_step(pool, &enrollment).await {
                tracing::error!("Failed to process enrollment {} step: {}", enrollment.id, e);
            }
        }

        Ok(())
    }

    // エンロールメントの次のステップを処理
    async fn process_enrollment_step(
        &self,
        pool: &PgPool,
        enrollment: &SequenceEnrollment,
    ) -> Result<(), String> {
        // シーケンスとステップ情報を取得
        let sequence = sequences::find_sequence_by_id(pool, enrollment.sequence_id, None)
            .await
            .map_err(|e| format!("シーケンスの取得に失敗しました: {}", e))?
            .ok_or_else(|| "シーケンスが見つかりません".to_string())?;

        let steps = sequences::find_sequence_steps(pool, enrollment.sequence_id)
            .await
            .map_err(|e| format!("シーケンスステップの取得に失敗しました: {}", e))?;

        // 現在のステップ順序を取得
        let current_step_order = if let Some(current_step_id) = enrollment.current_step_id {
            steps
                .iter()
                .find(|s| s.id == current_step_id)
                .map(|s| s.step_order)
                .unwrap_or(0)
        } else {
            0
        };

        // 次のステップを特定
        let next_step_order = current_step_order + 1;
        let next_step = steps
            .iter()
            .find(|s| s.step_order == next_step_order)
            .ok_or_else(|| "次のステップが見つかりません".to_string())?;

        // ステップの条件を評価
        if !self
            .evaluate_step_conditions(pool, next_step, enrollment)
            .await?
        {
            // 条件を満たさない場合はスキップして次のステップへ
            self.move_to_next_step(pool, enrollment, next_step_order + 1)
                .await?;
            return Ok(());
        }

        // ステップタイプに応じて処理
        match next_step.step_type.as_str() {
            "email" => {
                self.process_email_step(pool, &sequence, next_step, enrollment)
                    .await?;
            }
            "wait" => {
                self.process_wait_step(pool, next_step, enrollment).await?;
            }
            "condition" => {
                self.process_condition_step(pool, next_step, enrollment)
                    .await?;
            }
            "tag" => {
                self.process_tag_step(pool, next_step, enrollment).await?;
            }
            _ => {
                return Err(format!("不明なステップタイプ: {}", next_step.step_type));
            }
        }

        Ok(())
    }

    // ステップ条件の評価
    async fn evaluate_step_conditions(
        &self,
        _pool: &PgPool,
        step: &SequenceStep,
        _enrollment: &SequenceEnrollment,
    ) -> Result<bool, String> {
        // 条件が設定されていない場合は常にtrue
        if step.conditions.is_null() || step.conditions == json!({}) {
            return Ok(true);
        }

        // TODO: 条件評価ロジックの実装
        // 現時点では常にtrueを返す
        Ok(true)
    }

    // メール送信ステップの処理
    async fn process_email_step(
        &self,
        pool: &PgPool,
        sequence: &Sequence,
        step: &SequenceStep,
        enrollment: &SequenceEnrollment,
    ) -> Result<(), String> {
        // テンプレートIDが必要
        let template_id = step
            .template_id
            .ok_or_else(|| "メールステップにテンプレートIDが設定されていません".to_string())?;

        // テンプレートを取得
        let template = templates::find_template_by_id(pool, template_id, Some(sequence.user_id))
            .await
            .map_err(|e| format!("テンプレートの取得に失敗しました: {}", e))?
            .ok_or_else(|| "テンプレートが見つかりません".to_string())?;

        // 購読者情報を取得
        let subscriber =
            subscribers::find_subscriber_by_id(pool, enrollment.subscriber_id, sequence.user_id)
                .await
                .map_err(|e| format!("購読者情報の取得に失敗しました: {}", e))?
                .ok_or_else(|| "購読者が見つかりません".to_string())?;

        // メール送信
        self.send_sequence_email(pool, sequence, step, &template, &subscriber)
            .await?;

        // ステップログを記録
        self.log_step_execution(pool, enrollment.id, step.id, "sent", None)
            .await?;

        // 次のステップへ移動
        self.move_to_next_step(pool, enrollment, step.step_order + 1)
            .await?;

        Ok(())
    }

    // シーケンスメールの送信
    async fn send_sequence_email(
        &self,
        _pool: &PgPool,
        sequence: &Sequence,
        step: &SequenceStep,
        template: &crate::models::template::Template,
        subscriber: &Subscriber,
    ) -> Result<(), String> {
        // メールサービスを初期化
        let email_config = EmailService::from_env()
            .map_err(|e| format!("メール設定の読み込みに失敗しました: {}", e))?;
        let email_service = EmailService::new(email_config)
            .await
            .map_err(|e| format!("メールサービスの初期化に失敗しました: {}", e))?;

        // マークダウンサービスを初期化
        let markdown_service = MarkdownService::new();

        // 変数を準備
        let mut variables = if template.variables.is_null() {
            json!({})
        } else {
            template.variables.clone()
        };

        // 変数にマージ
        if let Value::Object(ref mut map) = variables {
            map.insert(
                "name".to_string(),
                json!(subscriber
                    .name
                    .clone()
                    .unwrap_or_else(|| "お客様".to_string())),
            );
            map.insert("email".to_string(), json!(subscriber.email.clone()));

            // カスタムフィールドを追加
            if let Value::Object(custom_fields) = &subscriber.custom_fields {
                for (key, value) in custom_fields {
                    map.insert(key.clone(), value.clone());
                }
            }

            // シーケンス関連の変数を追加
            map.insert("sequence_name".to_string(), json!(sequence.name));
            map.insert("step_name".to_string(), json!(step.name));

            // 配信停止URL
            map.insert(
                "unsubscribe_url".to_string(),
                json!(format!(
                    "https://markmail.example.com/unsubscribe/{}",
                    subscriber.id
                )),
            );
        }

        // HTMLとテキストをレンダリング
        let html_body = markdown_service
            .render_with_variables(&template.markdown_content, &variables)
            .map_err(|e| format!("HTMLレンダリングに失敗しました: {}", e))?;

        let text_body = html2text::from_read(html_body.as_bytes(), 80);

        // 件名の変数を置換
        let subject = if let Some(custom_subject) = &step.subject {
            self.replace_variables(custom_subject, &variables)
        } else {
            self.replace_variables(&template.subject_template, &variables)
        };

        let email_message = EmailMessage {
            to: vec![subscriber.email.clone()],
            subject,
            html_body,
            text_body: Some(text_body),
            reply_to: None,
            headers: None,
        };

        // メール送信
        email_service
            .send_email(&email_message)
            .await
            .map_err(|e| format!("メール送信に失敗しました: {}", e))?;

        Ok(())
    }

    // 変数置換
    fn replace_variables(&self, text: &str, variables: &Value) -> String {
        let mut result = text.to_string();
        if let Value::Object(vars) = variables {
            for (key, value) in vars {
                if let Value::String(val) = value {
                    result = result.replace(&format!("{{{{{}}}}}", key), val);
                }
            }
        }
        result
    }

    // 待機ステップの処理
    async fn process_wait_step(
        &self,
        pool: &PgPool,
        step: &SequenceStep,
        enrollment: &SequenceEnrollment,
    ) -> Result<(), String> {
        // 待機時間を計算
        let delay_minutes = step.delay_value;
        let delay_unit = step.delay_unit.as_str();

        let delay_duration = match delay_unit {
            "minutes" => Duration::minutes(delay_minutes as i64),
            "hours" => Duration::hours(delay_minutes as i64),
            "days" => Duration::days(delay_minutes as i64),
            _ => Duration::minutes(delay_minutes as i64),
        };

        let next_execution_at = Utc::now() + delay_duration;

        // 次の実行時刻を設定
        self.schedule_next_step(pool, enrollment, step.step_order + 1, next_execution_at)
            .await?;

        // ステップログを記録
        self.log_step_execution(pool, enrollment.id, step.id, "wait_scheduled", None)
            .await?;

        Ok(())
    }

    // 条件ステップの処理
    async fn process_condition_step(
        &self,
        pool: &PgPool,
        step: &SequenceStep,
        enrollment: &SequenceEnrollment,
    ) -> Result<(), String> {
        // TODO: 条件評価ロジックの実装
        // 現時点では次のステップへ進む
        self.move_to_next_step(pool, enrollment, step.step_order + 1)
            .await?;

        // ステップログを記録
        self.log_step_execution(pool, enrollment.id, step.id, "condition_evaluated", None)
            .await?;

        Ok(())
    }

    // タグステップの処理
    async fn process_tag_step(
        &self,
        pool: &PgPool,
        step: &SequenceStep,
        enrollment: &SequenceEnrollment,
    ) -> Result<(), String> {
        // シーケンスからuser_idを取得
        let sequence = sequences::find_sequence_by_id(pool, enrollment.sequence_id, None)
            .await
            .map_err(|e| format!("シーケンスの取得に失敗しました: {}", e))?
            .ok_or_else(|| "シーケンスが見つかりません".to_string())?;

        // action_configからタグを取得
        if let Some(tag) = step.action_config.get("tag").and_then(|v| v.as_str()) {
            // 購読者にタグを追加
            let current_subscriber = subscribers::find_subscriber_by_id(
                pool,
                enrollment.subscriber_id,
                sequence.user_id,
            )
            .await
            .map_err(|e| format!("購読者の取得に失敗しました: {}", e))?
            .ok_or_else(|| "購読者が見つかりません".to_string())?;

            let mut tags = current_subscriber.tags;
            if !tags.contains(&tag.to_string()) {
                tags.push(tag.to_string());
            }

            // タグを更新
            let update_request = crate::models::subscriber::UpdateSubscriberRequest {
                name: None,
                email: None,
                tags: Some(tags),
                custom_fields: None,
                status: None,
            };

            subscribers::update_subscriber(
                pool,
                enrollment.subscriber_id,
                sequence.user_id,
                &update_request,
            )
            .await
            .map_err(|e| format!("タグの更新に失敗しました: {}", e))?;
        }

        // 次のステップへ移動
        self.move_to_next_step(pool, enrollment, step.step_order + 1)
            .await?;

        // ステップログを記録
        self.log_step_execution(pool, enrollment.id, step.id, "tag_added", None)
            .await?;

        Ok(())
    }

    // 次のステップへ移動
    async fn move_to_next_step(
        &self,
        pool: &PgPool,
        enrollment: &SequenceEnrollment,
        next_step_order: i32,
    ) -> Result<(), String> {
        // シーケンスの全ステップ数を取得
        let steps = sequences::find_sequence_steps(pool, enrollment.sequence_id)
            .await
            .map_err(|e| format!("ステップ数の取得に失敗しました: {}", e))?;

        if next_step_order > steps.len() as i32 {
            // すべてのステップが完了
            sequences::complete_sequence_enrollment(pool, enrollment.id)
                .await
                .map_err(|e| format!("エンロールメントの完了に失敗しました: {}", e))?;
        } else {
            // 次のステップへ更新
            sequences::update_enrollment_progress(pool, enrollment.id, next_step_order)
                .await
                .map_err(|e| format!("進捗の更新に失敗しました: {}", e))?;
        }

        Ok(())
    }

    // 次のステップをスケジュール
    async fn schedule_next_step(
        &self,
        pool: &PgPool,
        enrollment: &SequenceEnrollment,
        next_step_order: i32,
        next_execution_at: DateTime<Utc>,
    ) -> Result<(), String> {
        sequences::schedule_next_enrollment_step(
            pool,
            enrollment.id,
            next_step_order,
            next_execution_at,
        )
        .await
        .map_err(|e| format!("次のステップのスケジューリングに失敗しました: {}", e))?;

        Ok(())
    }

    // ステップ実行ログの記録
    async fn log_step_execution(
        &self,
        pool: &PgPool,
        enrollment_id: Uuid,
        step_id: Uuid,
        status: &str,
        error_message: Option<String>,
    ) -> Result<SequenceStepLog, String> {
        sequences::create_sequence_step_log(pool, enrollment_id, step_id, status, error_message)
            .await
            .map_err(|e| format!("ステップログの記録に失敗しました: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace_variables() {
        let service = SequenceService::new();
        let text = "Hello {{name}}, your email is {{email}}";
        let variables = json!({
            "name": "John Doe",
            "email": "john@example.com"
        });

        let result = service.replace_variables(text, &variables);
        assert_eq!(result, "Hello John Doe, your email is john@example.com");
    }

    #[test]
    fn test_evaluate_trigger_conditions_empty() {
        let service = SequenceService::new();
        let sequence = Sequence {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            name: "Test Sequence".to_string(),
            description: None,
            trigger_type: TriggerType::Manual.as_str().to_string(),
            trigger_config: json!({}),
            status: crate::models::sequence::SequenceStatus::Active
                .as_str()
                .to_string(),
            active_subscribers: 0,
            completed_subscribers: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let result = service.evaluate_trigger_conditions(&sequence, None);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_evaluate_trigger_conditions_form_submission() {
        let service = SequenceService::new();
        let form_id = Uuid::new_v4();
        let sequence = Sequence {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            name: "Test Sequence".to_string(),
            description: None,
            trigger_type: TriggerType::FormSubmission.as_str().to_string(),
            trigger_config: json!({
                "form_id": form_id.to_string()
            }),
            status: crate::models::sequence::SequenceStatus::Active
                .as_str()
                .to_string(),
            active_subscribers: 0,
            completed_subscribers: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // 一致するフォームID
        let trigger_data = json!({
            "form_id": form_id.to_string()
        });
        let result = service.evaluate_trigger_conditions(&sequence, Some(&trigger_data));
        assert!(result.is_ok());
        assert!(result.unwrap());

        // 一致しないフォームID
        let wrong_trigger_data = json!({
            "form_id": Uuid::new_v4().to_string()
        });
        let result = service.evaluate_trigger_conditions(&sequence, Some(&wrong_trigger_data));
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
}

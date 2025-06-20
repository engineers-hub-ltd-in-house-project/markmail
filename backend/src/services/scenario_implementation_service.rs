use anyhow::Result;
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    ai::models::{
        GenerateScenarioResponse, GeneratedForm, GeneratedSequenceStep, GeneratedTemplate,
    },
    database::{forms, sequences, templates},
    models::{
        form::{CreateFormRequest, Form},
        sequence::{
            CreateSequenceRequest, CreateSequenceStepRequest, Sequence, StepType, TriggerType,
        },
        template::{CreateTemplateRequest, Template},
    },
};

#[derive(Default)]
pub struct ScenarioImplementationService;

impl ScenarioImplementationService {
    pub fn new() -> Self {
        Self
    }

    /// AI生成シナリオから実際のエンティティを作成
    pub async fn implement_scenario(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        scenario: &GenerateScenarioResponse,
    ) -> Result<ScenarioImplementationResult> {
        tracing::info!("シナリオ実装開始: {}", scenario.scenario_name);

        // 1. テンプレートを作成
        let mut templates_map = std::collections::HashMap::new();
        for template in &scenario.templates {
            let created_template = self
                .create_template_from_scenario(pool, user_id, template)
                .await?;
            templates_map.insert(template.name.clone(), created_template);
        }

        // 2. フォームを作成
        let form = if let Some(form_config) = scenario.forms.first() {
            Some(
                self.create_form_from_scenario(pool, user_id, form_config)
                    .await?,
            )
        } else {
            None
        };

        // 3. シーケンスを作成
        let sequence = self
            .create_sequence_from_scenario(
                pool,
                user_id,
                &scenario.sequence,
                &templates_map,
                form.as_ref().map(|f| f.id),
            )
            .await?;

        Ok(ScenarioImplementationResult {
            sequence_id: sequence.id,
            form_id: form.map(|f| f.id),
            template_ids: templates_map.values().map(|t| t.id).collect(),
        })
    }

    /// テンプレートを作成
    async fn create_template_from_scenario(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        template: &GeneratedTemplate,
    ) -> Result<Template> {
        let request = CreateTemplateRequest {
            name: template.name.clone(),
            subject_template: template.subject.clone(),
            markdown_content: template.content.clone(),
            variables: Some(Value::Array(
                template
                    .variables
                    .iter()
                    .map(|v| Value::String(v.clone()))
                    .collect(),
            )),
            is_public: Some(false),
        };

        templates::create_template(pool, user_id, &request)
            .await
            .map_err(|e| anyhow::anyhow!("テンプレート作成エラー: {}", e))
    }

    /// フォームを作成
    async fn create_form_from_scenario(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        form_config: &GeneratedForm,
    ) -> Result<Form> {
        // フォームフィールドをJSON形式に変換
        let form_fields = form_config.fields.iter().map(|field| {
            json!({
                "field_type": field.field_type,
                "name": field.name,
                "label": field.label,
                "required": field.required,
                "placeholder": field.options.as_ref().map(|opts| opts.join(", ")).unwrap_or_default()
            })
        }).collect::<Vec<_>>();

        let request = CreateFormRequest {
            name: form_config.name.clone(),
            description: Some(form_config.description.clone()),
            slug: None, // 自動生成される
            markdown_content: self
                .generate_form_markdown(&form_config.name, &form_config.description)
                .unwrap_or_default(),
            form_fields: Some(Value::Array(form_fields)),
            settings: Some(json!({
                "submit_button_text": "送信",
                "success_message": "フォームを送信しました。ありがとうございます。"
            })),
        };

        forms::create_form(pool, user_id, request).await
    }

    /// シーケンスを作成
    async fn create_sequence_from_scenario(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        sequence_config: &crate::ai::models::GeneratedSequence,
        templates_map: &std::collections::HashMap<String, Template>,
        form_id: Option<Uuid>,
    ) -> Result<Sequence> {
        // トリガー設定を決定
        let (trigger_type, trigger_config) = if let Some(form_id) = form_id {
            (
                TriggerType::FormSubmission.as_str().to_string(),
                Some(json!({
                    "form_id": form_id.to_string()
                })),
            )
        } else {
            (TriggerType::SubscriberCreated.as_str().to_string(), None)
        };

        let request = CreateSequenceRequest {
            name: sequence_config.name.clone(),
            description: Some(sequence_config.description.clone()),
            trigger_type,
            trigger_config,
        };

        let sequence = sequences::create_sequence(pool, user_id, request)
            .await
            .map_err(|e| anyhow::anyhow!("シーケンス作成エラー: {}", e))?;

        // ステップを作成
        for (index, step) in sequence_config.steps.iter().enumerate() {
            let step_request =
                self.convert_scenario_step_to_request(step, index as i32 + 1, templates_map)?;

            sequences::create_sequence_step(pool, sequence.id, step_request)
                .await
                .map_err(|e| anyhow::anyhow!("シーケンスステップ作成エラー: {}", e))?;
        }

        Ok(sequence)
    }

    /// シナリオステップをリクエストに変換
    fn convert_scenario_step_to_request(
        &self,
        step: &GeneratedSequenceStep,
        step_order: i32,
        templates_map: &std::collections::HashMap<String, Template>,
    ) -> Result<CreateSequenceStepRequest> {
        let step_type = match &step.step_type[..] {
            "email" => StepType::Email.as_str().to_string(),
            "wait" => StepType::Wait.as_str().to_string(),
            _ => return Err(anyhow::anyhow!("不明なステップタイプ: {}", step.step_type)),
        };

        // テンプレートIDを取得（メールステップの場合）
        let template_id = if step.step_type == "email" {
            if let Some(template_index) = step.template_index {
                // インデックスを使って対応するテンプレートを探す
                templates_map
                    .values()
                    .nth(template_index)
                    .map(|t| t.id)
                    .ok_or_else(|| {
                        anyhow::anyhow!("テンプレートインデックス {} が範囲外です", template_index)
                    })?
            } else {
                return Err(anyhow::anyhow!(
                    "メールステップにテンプレートが指定されていません"
                ));
            }
        } else {
            Uuid::nil()
        };

        Ok(CreateSequenceStepRequest {
            name: step.name.clone(),
            step_type,
            step_order,
            delay_value: Some(step.delay_value),
            delay_unit: Some(step.delay_unit.clone()),
            template_id: if step.step_type == "email" {
                Some(template_id)
            } else {
                None
            },
            subject: None,
            conditions: step.conditions.clone(),
            action_config: None,
        })
    }

    /// フォーム用のマークダウンを生成
    fn generate_form_markdown(&self, name: &str, description: &str) -> Option<String> {
        Some(format!(
            r#"# {}

{}

以下のフォームにご記入ください。"#,
            name, description
        ))
    }
}

/// シナリオ実装結果
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ScenarioImplementationResult {
    pub sequence_id: Uuid,
    pub form_id: Option<Uuid>,
    pub template_ids: Vec<Uuid>,
}

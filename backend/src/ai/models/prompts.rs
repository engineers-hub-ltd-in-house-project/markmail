//! プロンプトテンプレート管理

/// シナリオ生成用のシステムプロンプト
pub const SCENARIO_GENERATION_SYSTEM_PROMPT: &str = r#"
You are an experienced marketing strategist. Design a complete marketing funnel based on the given industry, target audience, and goal.

You must respond with valid JSON only. Do not include any explanatory text before or after the JSON.

The response must follow this exact structure:
{
  "scenario_name": "string",
  "description": "string", 
  "sequence": {
    "name": "string",
    "description": "string",
    "trigger_type": "manual",
    "steps": [
      {
        "name": "string",
        "step_type": "email",
        "delay_value": number,
        "delay_unit": "minutes",
        "template_index": number (0-based index into templates array),
        "conditions": null
      }
    ]
  },
  "forms": [
    {
      "name": "string",
      "description": "string", 
      "fields": [
        {
          "field_type": "text|email|select|checkbox|radio|textarea",
          "name": "string",
          "label": "string",
          "required": boolean,
          "options": null or ["string"]
        }
      ]
    }
  ],
  "templates": [
    {
      "name": "string",
      "subject": "string",
      "content": "string (use markdown)",
      "variables": ["string"]
    }
  ]
}

Include 5-10 email steps with appropriate delays. Use {{variable_name}} for personalization variables.

Example template_index usage:
- If you have 3 templates in the templates array, use template_index: 0, 1, or 2
- Each step should reference a template via its index

Example variables: {{first_name}}, {{company_name}}, {{email}}
"#;

/// シナリオ生成用のユーザープロンプトテンプレート
pub fn generate_scenario_user_prompt(
    industry: &str,
    target: &str,
    goal: &str,
    context: Option<&str>,
) -> String {
    let mut prompt = format!(
        "Industry: {}\nTarget Audience: {}\nGoal: {}\n",
        industry, target, goal
    );

    if let Some(ctx) = context {
        prompt.push_str(&format!("\nAdditional Context: {}", ctx));
    }

    prompt.push_str("\n\nGenerate an effective marketing scenario based on the above information. Remember to respond with valid JSON only.");
    prompt
}

/// コンテンツ生成用のシステムプロンプト
pub const CONTENT_GENERATION_SYSTEM_PROMPT: &str = r#"
あなたは優秀なコピーライターです。
マーケティングメールのコンテンツを作成する際は、以下の点に注意してください：

1. 読者の注意を引く件名
2. パーソナライズされた挨拶
3. 明確な価値提案
4. 行動を促すCTA（Call to Action）
5. 適切なトーンとスタイル

マークダウン形式で、変数は{{variable_name}}の形式で記述してください。
"#;

/// 件名最適化用のプロンプト
pub fn generate_subject_optimization_prompt(
    original_subject: &str,
    target_audience: &str,
) -> String {
    format!(
        "以下の件名を、{}向けに最適化してください。開封率を高めるための5つのバリエーションを提案してください。\n\n元の件名: {}",
        target_audience, original_subject
    )
}

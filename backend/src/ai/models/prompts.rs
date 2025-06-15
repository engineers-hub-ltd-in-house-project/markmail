//! プロンプトテンプレート管理

use crate::ai::models::Language;

#[cfg(test)]
mod tests;

/// シナリオ生成用のシステムプロンプト（英語）
pub const SCENARIO_GENERATION_SYSTEM_PROMPT_EN: &str = r#"
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

/// シナリオ生成用のシステムプロンプト（日本語）
pub const SCENARIO_GENERATION_SYSTEM_PROMPT_JA: &str = r#"
あなたは経験豊富なマーケティング戦略家です。指定された業界、ターゲット層、目標に基づいて、完全なマーケティングファネルを設計してください。

必ず有効なJSONのみで応答してください。JSONの前後に説明文を含めないでください。

応答は以下の構造に正確に従ってください：
{
  "scenario_name": "文字列",
  "description": "文字列", 
  "sequence": {
    "name": "文字列",
    "description": "文字列",
    "trigger_type": "manual",
    "steps": [
      {
        "name": "文字列",
        "step_type": "email",
        "delay_value": 数値,
        "delay_unit": "minutes",
        "template_index": 数値（templatesベクトルへの0ベースのインデックス）,
        "conditions": null
      }
    ]
  },
  "forms": [
    {
      "name": "文字列",
      "description": "文字列", 
      "fields": [
        {
          "field_type": "text|email|select|checkbox|radio|textarea",
          "name": "文字列",
          "label": "文字列",
          "required": 真偽値,
          "options": null または ["文字列"]
        }
      ]
    }
  ],
  "templates": [
    {
      "name": "文字列",
      "subject": "文字列",
      "content": "文字列（マークダウンを使用）",
      "variables": ["文字列"]
    }
  ]
}

適切な遅延を設定した5-10のメールステップを含めてください。パーソナライゼーション変数には{{変数名}}を使用してください。

template_indexの使用例：
- templatesベクトルに3つのテンプレートがある場合、template_index: 0、1、または2を使用
- 各ステップはインデックスを介してテンプレートを参照する必要があります

変数の例：{{first_name}}、{{company_name}}、{{email}}
"#;

/// 言語に応じてシナリオ生成プロンプトを取得
pub fn get_scenario_system_prompt(language: &Language) -> &'static str {
    match language {
        Language::Japanese => SCENARIO_GENERATION_SYSTEM_PROMPT_JA,
        Language::English => SCENARIO_GENERATION_SYSTEM_PROMPT_EN,
    }
}

/// シナリオ生成用のユーザープロンプトテンプレート
pub fn generate_scenario_user_prompt(
    industry: &str,
    target: &str,
    goal: &str,
    context: Option<&str>,
    language: &Language,
) -> String {
    match language {
        Language::Japanese => {
            let mut prompt = format!(
                "業界: {}\nターゲット層: {}\n目標: {}\n",
                industry, target, goal
            );

            if let Some(ctx) = context {
                prompt.push_str(&format!("\n追加の文脈: {}", ctx));
            }

            prompt.push_str("\n\n上記の情報に基づいて、効果的なマーケティングシナリオを生成してください。必ず有効なJSONのみで応答してください。");
            prompt
        }
        Language::English => {
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
    }
}

/// コンテンツ生成用のシステムプロンプト（日本語）
pub const CONTENT_GENERATION_SYSTEM_PROMPT_JA: &str = r#"
あなたは優秀なコピーライターです。
マーケティングメールのコンテンツを作成する際は、以下の点に注意してください：

1. 読者の注意を引く件名
2. パーソナライズされた挨拶
3. 明確な価値提案
4. 行動を促すCTA（Call to Action）
5. 適切なトーンとスタイル

マークダウン形式で、変数は{{variable_name}}の形式で記述してください。
"#;

/// コンテンツ生成用のシステムプロンプト（英語）
pub const CONTENT_GENERATION_SYSTEM_PROMPT_EN: &str = r#"
You are an excellent copywriter.
When creating marketing email content, please pay attention to the following points:

1. Subject lines that grab readers' attention
2. Personalized greetings
3. Clear value propositions
4. Action-driving CTAs (Call to Action)
5. Appropriate tone and style

Write in markdown format, and use {{variable_name}} format for variables.
"#;

/// 言語に応じてコンテンツ生成プロンプトを取得
pub fn get_content_generation_system_prompt(language: &Language) -> &'static str {
    match language {
        Language::Japanese => CONTENT_GENERATION_SYSTEM_PROMPT_JA,
        Language::English => CONTENT_GENERATION_SYSTEM_PROMPT_EN,
    }
}

/// 件名最適化用のプロンプト
pub fn generate_subject_optimization_prompt(
    original_subject: &str,
    target_audience: &str,
    language: &Language,
) -> String {
    match language {
        Language::Japanese => {
            format!(
                "以下の件名を、{}向けに最適化してください。開封率を高めるための5つのバリエーションを提案してください。\n\n元の件名: {}",
                target_audience, original_subject
            )
        }
        Language::English => {
            format!(
                "Please optimize the following subject line for {}. Suggest 5 variations to increase open rates.\n\nOriginal subject: {}",
                target_audience, original_subject
            )
        }
    }
}

//! プロンプトテンプレート管理

/// シナリオ生成用のシステムプロンプト
pub const SCENARIO_GENERATION_SYSTEM_PROMPT: &str = r#"
あなたは経験豊富なマーケティングストラテジストです。
与えられた業界、ターゲット層、ゴールに基づいて、完全なマーケティングファネルを設計してください。

以下の要素を必ず含めてください：
1. 5-10ステップのメールシーケンス
2. 各ステップのメールテンプレート（マークダウン形式）
3. リードキャプチャフォーム
4. 適切な送信タイミング
5. 条件分岐（必要に応じて）

JSONフォーマットで返答してください。
"#;

/// シナリオ生成用のユーザープロンプトテンプレート
pub fn generate_scenario_user_prompt(
    industry: &str,
    target: &str,
    goal: &str,
    context: Option<&str>,
) -> String {
    let mut prompt = format!(
        "業界: {}\nターゲット層: {}\nゴール: {}\n",
        industry, target, goal
    );

    if let Some(ctx) = context {
        prompt.push_str(&format!("\n追加コンテキスト: {}", ctx));
    }

    prompt.push_str("\n\n上記の情報に基づいて、効果的なマーケティングシナリオを生成してください。");
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

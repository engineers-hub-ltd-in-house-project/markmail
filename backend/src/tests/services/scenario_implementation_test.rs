use crate::ai::models::{
    GenerateScenarioResponse, GeneratedForm, GeneratedFormField, GeneratedSequence,
    GeneratedSequenceStep, GeneratedTemplate,
};
use crate::services::scenario_implementation_service::ScenarioImplementationService;
use crate::AppState;
use uuid::Uuid;

#[tokio::test]
async fn test_implement_scenario_full_flow() {
    let app_state = AppState::new_for_test().await;
    let pool = app_state.db.clone();

    // テストユーザーを作成
    let user_id = Uuid::new_v4();
    let hashed_password = crate::utils::password::hash_password("password123").unwrap();
    sqlx::query!(
        "INSERT INTO users (id, name, email, password_hash) VALUES ($1, $2, $3, $4)",
        user_id,
        "Test User",
        format!("test-{}@example.com", user_id),
        hashed_password
    )
    .execute(&pool)
    .await
    .expect("Failed to create test user");

    // テスト用のシナリオを作成
    let scenario = GenerateScenarioResponse {
        scenario_name: "ECサイト新規登録キャンペーン".to_string(),
        description: "新規登録者向けの自動フォローアップシーケンス".to_string(),
        sequence: GeneratedSequence {
            name: "新規登録フォローアップシーケンス".to_string(),
            description: "登録から購入までをサポートする自動メールシーケンス".to_string(),
            trigger_type: "form_submission".to_string(),
            steps: vec![
                GeneratedSequenceStep {
                    name: "ウェルカムメール送信".to_string(),
                    step_type: "email".to_string(),
                    delay_value: 0,
                    delay_unit: "minutes".to_string(),
                    template_index: Some(0),
                    conditions: None,
                },
                GeneratedSequenceStep {
                    name: "3日間待機".to_string(),
                    step_type: "wait".to_string(),
                    delay_value: 3,
                    delay_unit: "days".to_string(),
                    template_index: None,
                    conditions: None,
                },
                GeneratedSequenceStep {
                    name: "特別オファーメール".to_string(),
                    step_type: "email".to_string(),
                    delay_value: 0,
                    delay_unit: "minutes".to_string(),
                    template_index: Some(1),
                    conditions: None,
                },
            ],
        },
        forms: vec![GeneratedForm {
            name: "新規登録フォーム".to_string(),
            description: "メールアドレスと基本情報を収集".to_string(),
            fields: vec![
                GeneratedFormField {
                    field_type: "email".to_string(),
                    name: "email".to_string(),
                    label: "メールアドレス".to_string(),
                    required: true,
                    options: None,
                },
                GeneratedFormField {
                    field_type: "text".to_string(),
                    name: "name".to_string(),
                    label: "お名前".to_string(),
                    required: true,
                    options: None,
                },
                GeneratedFormField {
                    field_type: "select".to_string(),
                    name: "interest".to_string(),
                    label: "興味のあるカテゴリ".to_string(),
                    required: false,
                    options: Some(vec![
                        "ファッション".to_string(),
                        "家電".to_string(),
                        "食品".to_string(),
                        "その他".to_string(),
                    ]),
                },
            ],
        }],
        templates: vec![
            GeneratedTemplate {
                name: "ウェルカムメール".to_string(),
                subject: "{{name}}様、ご登録ありがとうございます！".to_string(),
                content: r#"# ご登録ありがとうございます！

こんにちは、{{name}}様

この度は弊社ECサイトにご登録いただき、誠にありがとうございます。
これから素敵な商品との出会いをお手伝いさせていただきます。

## 初回購入特典
- 全商品10%OFF
- 送料無料

ご不明な点がございましたら、お気軽にお問い合わせください。"#
                    .to_string(),
                variables: vec!["name".to_string()],
            },
            GeneratedTemplate {
                name: "特別オファーメール".to_string(),
                subject: "【期間限定】{{name}}様だけの特別オファー".to_string(),
                content: r#"# 特別なお知らせです

{{name}}様

登録から3日間、まだお買い物をされていないようですね。
今なら期間限定で、さらにお得な特典をご用意しました！

## 限定オファー
- 追加5%OFF（合計15%OFF）
- プレゼント付き

このチャンスをお見逃しなく！"#
                    .to_string(),
                variables: vec!["name".to_string()],
            },
        ],
    };

    // サービスを実行
    let service = ScenarioImplementationService::new();
    let result = service
        .implement_scenario(&pool, user_id, &scenario)
        .await
        .expect("シナリオ実装に失敗");

    // 結果を検証
    assert!(result.sequence_id != Uuid::nil());
    assert!(result.form_id.is_some());
    assert_eq!(result.template_ids.len(), 2);

    // 作成されたシーケンスを確認
    let sequence = sqlx::query!(
        "SELECT name, trigger_type FROM sequences WHERE id = $1",
        result.sequence_id
    )
    .fetch_one(&pool)
    .await
    .expect("シーケンスが見つかりません");

    assert_eq!(sequence.name, "新規登録フォローアップシーケンス");
    assert_eq!(sequence.trigger_type, "form_submission");

    // シーケンスステップを確認
    let steps = sqlx::query!(
        "SELECT name, step_type, step_order FROM sequence_steps WHERE sequence_id = $1 ORDER BY step_order",
        result.sequence_id
    )
    .fetch_all(&pool)
    .await
    .expect("ステップの取得に失敗");

    assert_eq!(steps.len(), 3);
    assert_eq!(steps[0].name, "ウェルカムメール送信");
    assert_eq!(steps[0].step_type, "email");
    assert_eq!(steps[1].name, "3日間待機");
    assert_eq!(steps[1].step_type, "wait");
    assert_eq!(steps[2].name, "特別オファーメール");
    assert_eq!(steps[2].step_type, "email");

    // 作成されたフォームを確認
    if let Some(form_id) = result.form_id {
        let form = sqlx::query!("SELECT name, form_fields FROM forms WHERE id = $1", form_id)
            .fetch_one(&pool)
            .await
            .expect("フォームが見つかりません");

        assert_eq!(form.name, "新規登録フォーム");

        let fields = form
            .form_fields
            .as_array()
            .expect("フィールドが配列ではありません");
        assert_eq!(fields.len(), 3);
    }

    // 作成されたテンプレートを確認
    for template_id in &result.template_ids {
        let template = sqlx::query!("SELECT name FROM templates WHERE id = $1", template_id)
            .fetch_one(&pool)
            .await
            .expect("テンプレートが見つかりません");

        assert!(
            template.name == "ウェルカムメール" || template.name == "特別オファーメール",
            "予期しないテンプレート名: {}",
            template.name
        );
    }

    // テストデータをクリーンアップ
    sqlx::query!(
        "DELETE FROM sequence_steps WHERE sequence_id = $1",
        result.sequence_id
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query!("DELETE FROM sequences WHERE id = $1", result.sequence_id)
        .execute(&pool)
        .await
        .unwrap();
    if let Some(form_id) = result.form_id {
        sqlx::query!("DELETE FROM forms WHERE id = $1", form_id)
            .execute(&pool)
            .await
            .unwrap();
    }
    for template_id in &result.template_ids {
        sqlx::query!("DELETE FROM templates WHERE id = $1", template_id)
            .execute(&pool)
            .await
            .unwrap();
    }
    sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
        .execute(&pool)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_implement_scenario_without_form() {
    let app_state = AppState::new_for_test().await;
    let pool = app_state.db.clone();

    // テストユーザーを作成
    let user_id = Uuid::new_v4();
    let hashed_password = crate::utils::password::hash_password("password123").unwrap();
    sqlx::query!(
        "INSERT INTO users (id, name, email, password_hash) VALUES ($1, $2, $3, $4)",
        user_id,
        "Test User 2",
        format!("test-{}@example.com", user_id),
        hashed_password
    )
    .execute(&pool)
    .await
    .expect("Failed to create test user");

    // フォームなしのシナリオ
    let scenario = GenerateScenarioResponse {
        scenario_name: "既存顧客向けキャンペーン".to_string(),
        description: "既存顧客への再購入促進".to_string(),
        sequence: GeneratedSequence {
            name: "再購入促進シーケンス".to_string(),
            description: "購読者作成時にトリガーされるシーケンス".to_string(),
            trigger_type: "subscriber_created".to_string(),
            steps: vec![GeneratedSequenceStep {
                name: "再購入案内メール".to_string(),
                step_type: "email".to_string(),
                delay_value: 1,
                delay_unit: "days".to_string(),
                template_index: Some(0),
                conditions: None,
            }],
        },
        forms: vec![], // フォームなし
        templates: vec![GeneratedTemplate {
            name: "再購入案内".to_string(),
            subject: "お得な情報があります".to_string(),
            content: "# 再購入のご案内\n\nいつもご利用ありがとうございます。".to_string(),
            variables: vec![],
        }],
    };

    let service = ScenarioImplementationService::new();
    let result = service
        .implement_scenario(&pool, user_id, &scenario)
        .await
        .expect("シナリオ実装に失敗");

    // フォームIDがNoneであることを確認
    assert!(result.form_id.is_none());
    assert_eq!(result.template_ids.len(), 1);

    // シーケンスのトリガータイプを確認
    let sequence = sqlx::query!(
        "SELECT trigger_type FROM sequences WHERE id = $1",
        result.sequence_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(sequence.trigger_type, "subscriber_created");

    // テストデータをクリーンアップ
    sqlx::query!(
        "DELETE FROM sequence_steps WHERE sequence_id = $1",
        result.sequence_id
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query!("DELETE FROM sequences WHERE id = $1", result.sequence_id)
        .execute(&pool)
        .await
        .unwrap();
    for template_id in &result.template_ids {
        sqlx::query!("DELETE FROM templates WHERE id = $1", template_id)
            .execute(&pool)
            .await
            .unwrap();
    }
    sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
        .execute(&pool)
        .await
        .unwrap();
}

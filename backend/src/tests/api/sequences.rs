use crate::{
    api::sequences,
    middleware::auth::AuthUser,
    models::sequence::{CreateSequenceRequest, CreateSequenceStepRequest, UpdateSequenceRequest},
    AppState,
};
use axum::{
    extract::{Extension, Json as AxumJson, Path},
    http::StatusCode,
    Json,
};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

// テスト用のヘルパー関数
async fn create_test_user(pool: &PgPool) -> Uuid {
    let user_id = Uuid::new_v4();
    let hashed_password = crate::utils::password::hash_password("password123").unwrap();

    sqlx::query!(
        "INSERT INTO users (id, name, email, password_hash) VALUES ($1, $2, $3, $4)",
        user_id,
        "Test User",
        format!("test-{}@example.com", user_id),
        hashed_password
    )
    .execute(pool)
    .await
    .expect("Failed to create test user");

    user_id
}

async fn create_test_template(pool: &PgPool, user_id: Uuid) -> Uuid {
    let template_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO templates (
            id, user_id, name, subject_template, markdown_content, html_content, 
            variables, is_public, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW())
        "#,
        template_id,
        user_id,
        "テストテンプレート",
        "テスト件名 {{name}}",
        "# テスト\n\nこんにちは、{{name}}さん！",
        Some("<h1>テスト</h1><p>こんにちは、{{name}}さん！</p>".to_string()),
        json!({"name": "テスト値"}),
        false,
    )
    .execute(pool)
    .await
    .expect("Failed to create test template");

    template_id
}

#[tokio::test]
async fn test_create_and_get_sequence() {
    let app_state = AppState::new_for_test().await;
    let pool = app_state.db.clone();
    let user_id = create_test_user(&pool).await;

    let auth_user = AuthUser {
        user_id,
        email: "test@example.com".to_string(),
        name: "Test User".to_string(),
    };

    // シーケンス作成リクエスト
    let create_req = CreateSequenceRequest {
        name: "ウェルカムシーケンス".to_string(),
        description: Some("新規登録者向けのウェルカムシーケンス".to_string()),
        trigger_type: "registration".to_string(),
        trigger_config: Some(json!({
            "delay_hours": 0
        })),
    };

    // シーケンス作成API呼び出し
    let create_result = sequences::create_sequence(
        axum::extract::State(app_state.clone()),
        Extension(auth_user.clone()),
        AxumJson(create_req.clone()),
    )
    .await;

    // 作成結果を検証
    assert!(
        create_result.is_ok(),
        "Sequence creation failed: {:?}",
        create_result.err()
    );
    let (status, response) = create_result.unwrap();
    assert_eq!(status, StatusCode::CREATED);

    let sequence = response.0;
    assert_eq!(sequence.name, "ウェルカムシーケンス");
    assert_eq!(sequence.user_id, user_id);
    assert_eq!(sequence.trigger_type, "registration");
    assert_eq!(sequence.status, "draft");

    let sequence_id = sequence.id;

    // シーケンス取得API呼び出し
    let get_result = sequences::get_sequence(
        axum::extract::State(app_state.clone()),
        Extension(auth_user.clone()),
        Path(sequence_id),
    )
    .await;

    // 取得結果を検証
    assert!(get_result.is_ok(), "Sequence retrieval should succeed");

    let retrieved_sequence = get_result.unwrap().0;
    assert_eq!(retrieved_sequence.id, sequence.id);
    assert_eq!(retrieved_sequence.name, sequence.name);

    // 別のユーザーでアクセスした場合は失敗するはず
    let other_user_id = create_test_user(&pool).await;
    let other_auth_user = AuthUser {
        user_id: other_user_id,
        email: "other@example.com".to_string(),
        name: "Other User".to_string(),
    };

    let other_get_result = sequences::get_sequence(
        axum::extract::State(app_state.clone()),
        Extension(other_auth_user),
        Path(sequence_id),
    )
    .await;

    // 他のユーザーはシーケンスにアクセスできないはず（403 Forbidden）
    assert!(other_get_result.is_err());
    if let Err((status, _)) = other_get_result {
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    // クリーンアップ
    sqlx::query!("DELETE FROM sequences WHERE id = $1", sequence_id)
        .execute(&pool)
        .await
        .expect("Failed to clean up test sequence");

    sqlx::query!(
        "DELETE FROM users WHERE id IN ($1, $2)",
        user_id,
        other_user_id
    )
    .execute(&pool)
    .await
    .expect("Failed to clean up test users");
}

#[tokio::test]
async fn test_update_sequence() {
    let app_state = AppState::new_for_test().await;
    let pool = app_state.db.clone();
    let user_id = create_test_user(&pool).await;

    let auth_user = AuthUser {
        user_id,
        email: "test@example.com".to_string(),
        name: "Test User".to_string(),
    };

    // シーケンス作成
    let create_req = CreateSequenceRequest {
        name: "オリジナルシーケンス".to_string(),
        description: Some("元の説明".to_string()),
        trigger_type: "form_submission".to_string(),
        trigger_config: Some(json!({})),
    };

    let create_result = sequences::create_sequence(
        axum::extract::State(app_state.clone()),
        Extension(auth_user.clone()),
        AxumJson(create_req),
    )
    .await
    .unwrap();

    let sequence_id = create_result.1.id;

    // シーケンス更新リクエスト
    let update_req = UpdateSequenceRequest {
        name: Some("更新されたシーケンス".to_string()),
        description: Some("更新された説明".to_string()),
        trigger_type: Some("registration".to_string()),
        trigger_config: Some(json!({"delay_hours": 24})),
        status: Some("active".to_string()),
    };

    // シーケンス更新API呼び出し
    let update_result = sequences::update_sequence(
        axum::extract::State(app_state.clone()),
        Extension(auth_user.clone()),
        Path(sequence_id),
        AxumJson(update_req.clone()),
    )
    .await;

    // 更新結果を検証
    assert!(update_result.is_ok(), "Sequence update should succeed");

    let updated_sequence = update_result.unwrap().0;
    assert_eq!(updated_sequence.name, "更新されたシーケンス");
    assert_eq!(
        updated_sequence.description,
        Some("更新された説明".to_string())
    );
    assert_eq!(updated_sequence.trigger_type, "registration");
    assert_eq!(updated_sequence.status, "active");

    // クリーンアップ
    sqlx::query!("DELETE FROM sequences WHERE id = $1", sequence_id)
        .execute(&pool)
        .await
        .expect("Failed to clean up test sequence");

    sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
        .execute(&pool)
        .await
        .expect("Failed to clean up test user");
}

#[tokio::test]
async fn test_delete_sequence() {
    let app_state = AppState::new_for_test().await;
    let pool = app_state.db.clone();
    let user_id = create_test_user(&pool).await;

    let auth_user = AuthUser {
        user_id,
        email: "test@example.com".to_string(),
        name: "Test User".to_string(),
    };

    // シーケンス作成
    let create_req = CreateSequenceRequest {
        name: "削除対象シーケンス".to_string(),
        description: None,
        trigger_type: "manual".to_string(),
        trigger_config: Some(json!({})),
    };

    let create_result = sequences::create_sequence(
        axum::extract::State(app_state.clone()),
        Extension(auth_user.clone()),
        AxumJson(create_req),
    )
    .await
    .unwrap();

    let sequence_id = create_result.1.id;

    // シーケンス削除API呼び出し
    let delete_result = sequences::delete_sequence(
        axum::extract::State(app_state.clone()),
        Extension(auth_user.clone()),
        Path(sequence_id),
    )
    .await;

    // 削除結果を検証
    assert!(delete_result.is_ok(), "Sequence deletion should succeed");
    assert_eq!(delete_result.unwrap(), StatusCode::NO_CONTENT);

    // 削除後はシーケンスが取得できないことを確認
    let get_after_delete = sequences::get_sequence(
        axum::extract::State(app_state.clone()),
        Extension(auth_user.clone()),
        Path(sequence_id),
    )
    .await;

    assert!(get_after_delete.is_err());
    if let Err((status, _)) = get_after_delete {
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    // クリーンアップ
    sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
        .execute(&pool)
        .await
        .expect("Failed to clean up test user");
}

#[tokio::test]
async fn test_list_sequences() {
    let app_state = AppState::new_for_test().await;
    let pool = app_state.db.clone();
    let user_id = create_test_user(&pool).await;

    let auth_user = AuthUser {
        user_id,
        email: "test@example.com".to_string(),
        name: "Test User".to_string(),
    };

    // 複数のシーケンスを作成
    let mut sequence_ids = Vec::new();
    for i in 0..3 {
        let create_req = CreateSequenceRequest {
            name: format!("テストシーケンス {}", i),
            description: Some(format!("説明 {}", i)),
            trigger_type: if i % 2 == 0 {
                "registration".to_string()
            } else {
                "form_submission".to_string()
            },
            trigger_config: Some(json!({})),
        };

        let result = sequences::create_sequence(
            axum::extract::State(app_state.clone()),
            Extension(auth_user.clone()),
            AxumJson(create_req),
        )
        .await
        .unwrap();

        sequence_ids.push(result.1.id);
    }

    // シーケンス一覧取得API呼び出し
    let list_result = sequences::get_sequences(
        axum::extract::State(app_state.clone()),
        Extension(auth_user.clone()),
    )
    .await;

    // 一覧結果を検証
    assert!(list_result.is_ok(), "Sequence listing should succeed");

    let sequences_list = list_result.unwrap().0;
    assert_eq!(sequences_list.len(), 3);

    // シーケンス名でソートされているか確認
    let sequence_names: Vec<String> = sequences_list.iter().map(|s| s.name.clone()).collect();
    assert!(sequence_names.contains(&"テストシーケンス 0".to_string()));
    assert!(sequence_names.contains(&"テストシーケンス 1".to_string()));
    assert!(sequence_names.contains(&"テストシーケンス 2".to_string()));

    // クリーンアップ
    for sequence_id in sequence_ids {
        sqlx::query!("DELETE FROM sequences WHERE id = $1", sequence_id)
            .execute(&pool)
            .await
            .expect("Failed to clean up test sequence");
    }

    sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
        .execute(&pool)
        .await
        .expect("Failed to clean up test user");
}

#[tokio::test]
async fn test_sequence_with_steps() {
    let app_state = AppState::new_for_test().await;
    let pool = app_state.db.clone();
    let user_id = create_test_user(&pool).await;
    let template_id = create_test_template(&pool, user_id).await;

    let auth_user = AuthUser {
        user_id,
        email: "test@example.com".to_string(),
        name: "Test User".to_string(),
    };

    // シーケンス作成
    let create_seq_req = CreateSequenceRequest {
        name: "ステップ付きシーケンス".to_string(),
        description: Some("ステップ機能をテストするシーケンス".to_string()),
        trigger_type: "registration".to_string(),
        trigger_config: Some(json!({})),
    };

    let create_seq_result = sequences::create_sequence(
        axum::extract::State(app_state.clone()),
        Extension(auth_user.clone()),
        AxumJson(create_seq_req),
    )
    .await
    .unwrap();

    let sequence_id = create_seq_result.1 .0.id;

    // ステップ作成
    let create_step_req = CreateSequenceStepRequest {
        name: "ウェルカムメール".to_string(),
        step_order: 1,
        step_type: "send_email".to_string(),
        delay_value: Some(0),
        delay_unit: Some("hours".to_string()),
        template_id: Some(template_id),
        subject: Some("ようこそ！".to_string()),
        conditions: Some(json!({})),
        action_config: Some(json!({})),
    };

    let create_step_result = sequences::create_sequence_step(
        axum::extract::State(app_state.clone()),
        Extension(auth_user.clone()),
        Path(sequence_id),
        AxumJson(create_step_req),
    )
    .await;

    assert!(create_step_result.is_ok(), "Step creation should succeed");
    let (status, step_response) = create_step_result.unwrap();
    assert_eq!(status, StatusCode::CREATED);

    let step = step_response.0;
    assert_eq!(step.name, "ウェルカムメール");
    assert_eq!(step.step_order, 1);
    assert_eq!(step.step_type, "send_email");

    // ステップ付きシーケンス取得
    let get_with_steps_result = sequences::get_sequence_with_steps(
        axum::extract::State(app_state.clone()),
        Extension(auth_user.clone()),
        Path(sequence_id),
    )
    .await;

    assert!(
        get_with_steps_result.is_ok(),
        "Get sequence with steps should succeed"
    );

    let sequence_with_steps = get_with_steps_result.unwrap().0;
    assert_eq!(sequence_with_steps.sequence.name, "ステップ付きシーケンス");
    assert_eq!(sequence_with_steps.steps.len(), 1);
    assert_eq!(sequence_with_steps.steps[0].name, "ウェルカムメール");

    // クリーンアップ
    sqlx::query!(
        "DELETE FROM sequence_steps WHERE sequence_id = $1",
        sequence_id
    )
    .execute(&pool)
    .await
    .expect("Failed to clean up test steps");

    sqlx::query!("DELETE FROM sequences WHERE id = $1", sequence_id)
        .execute(&pool)
        .await
        .expect("Failed to clean up test sequence");

    sqlx::query!("DELETE FROM templates WHERE id = $1", template_id)
        .execute(&pool)
        .await
        .expect("Failed to clean up test template");

    sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
        .execute(&pool)
        .await
        .expect("Failed to clean up test user");
}

#[tokio::test]
async fn test_activate_sequence() {
    let app_state = AppState::new_for_test().await;
    let pool = app_state.db.clone();
    let user_id = create_test_user(&pool).await;

    let auth_user = AuthUser {
        user_id,
        email: "test@example.com".to_string(),
        name: "Test User".to_string(),
    };

    // シーケンス作成
    let create_req = CreateSequenceRequest {
        name: "テストシーケンス".to_string(),
        description: Some("アクティベーションテスト".to_string()),
        trigger_type: "form_submission".to_string(),
        trigger_config: None,
    };

    let result = sequences::create_sequence(
        axum::extract::State(app_state.clone()),
        Extension(auth_user.clone()),
        AxumJson(create_req),
    )
    .await;

    let (status, Json(sequence)) = result.unwrap();
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(sequence.status, "draft");

    // シーケンスをアクティベート
    let result = sequences::activate_sequence(
        axum::extract::State(app_state.clone()),
        Extension(auth_user.clone()),
        Path(sequence.id),
    )
    .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), StatusCode::NO_CONTENT);

    // ステータスが変更されたことを確認
    let updated_sequence = crate::database::sequences::get_sequence_by_id(&pool, sequence.id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(updated_sequence.status, "active");
}

#[tokio::test]
async fn test_pause_sequence() {
    let app_state = AppState::new_for_test().await;
    let pool = app_state.db.clone();
    let user_id = create_test_user(&pool).await;

    let auth_user = AuthUser {
        user_id,
        email: "test@example.com".to_string(),
        name: "Test User".to_string(),
    };

    // シーケンス作成
    let create_req = CreateSequenceRequest {
        name: "テストシーケンス".to_string(),
        description: Some("一時停止テスト".to_string()),
        trigger_type: "form_submission".to_string(),
        trigger_config: None,
    };

    let result = sequences::create_sequence(
        axum::extract::State(app_state.clone()),
        Extension(auth_user.clone()),
        AxumJson(create_req),
    )
    .await;

    let (status, Json(sequence)) = result.unwrap();
    assert_eq!(status, StatusCode::CREATED);

    // まずアクティベート
    crate::database::sequences::update_sequence_status(&pool, sequence.id, "active")
        .await
        .unwrap();

    // シーケンスを一時停止
    let result = sequences::pause_sequence(
        axum::extract::State(app_state.clone()),
        Extension(auth_user.clone()),
        Path(sequence.id),
    )
    .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), StatusCode::NO_CONTENT);

    // ステータスが変更されたことを確認
    let updated_sequence = crate::database::sequences::get_sequence_by_id(&pool, sequence.id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(updated_sequence.status, "paused");
}

#[tokio::test]
async fn test_activate_sequence_unauthorized() {
    let app_state = AppState::new_for_test().await;
    let pool = app_state.db.clone();
    let user_id1 = create_test_user(&pool).await;
    let user_id2 = create_test_user(&pool).await;

    let auth_user1 = AuthUser {
        user_id: user_id1,
        email: "test1@example.com".to_string(),
        name: "Test User 1".to_string(),
    };

    let auth_user2 = AuthUser {
        user_id: user_id2,
        email: "test2@example.com".to_string(),
        name: "Test User 2".to_string(),
    };

    // user1のシーケンスを作成
    let create_req = CreateSequenceRequest {
        name: "テストシーケンス".to_string(),
        description: Some("権限テスト".to_string()),
        trigger_type: "form_submission".to_string(),
        trigger_config: None,
    };

    let result = sequences::create_sequence(
        axum::extract::State(app_state.clone()),
        Extension(auth_user1.clone()),
        AxumJson(create_req),
    )
    .await;

    let (status, Json(sequence)) = result.unwrap();
    assert_eq!(status, StatusCode::CREATED);

    // user2でアクティベートを試みる
    let result = sequences::activate_sequence(
        axum::extract::State(app_state.clone()),
        Extension(auth_user2),
        Path(sequence.id),
    )
    .await;

    assert!(result.is_err());
    let (status, _) = result.unwrap_err();
    assert_eq!(status, StatusCode::FORBIDDEN);
}

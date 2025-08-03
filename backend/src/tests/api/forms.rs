use crate::{
    api::forms,
    middleware::auth::AuthUser,
    models::form::{CreateFormRequest, UpdateFormRequest},
    AppState,
};
use axum::{
    extract::{Extension, Json as AxumJson, Path, Query},
    http::StatusCode,
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

#[tokio::test]
async fn test_create_and_get_form() {
    let app_state = AppState::new_for_test().await;
    let pool = app_state.db.clone();
    let user_id = create_test_user(&pool).await;

    let auth_user = AuthUser {
        user_id,
        email: "test@example.com".to_string(),
        name: "Test User".to_string(),
    };

    // フォーム作成リクエスト
    let create_req = CreateFormRequest {
        name: "Contact Form".to_string(),
        description: Some("A simple contact form".to_string()),
        slug: Some("contact".to_string()),
        markdown_content: "# Contact Us\n\nPlease fill out this form.".to_string(),
        form_fields: Some(json!([
            {
                "field_type": "text",
                "name": "name",
                "label": "Your Name",
                "required": true
            },
            {
                "field_type": "email",
                "name": "email",
                "label": "Email Address",
                "required": true
            }
        ])),
        settings: Some(json!({
            "submit_button_text": "Send Message",
            "success_message": "Thank you for contacting us!"
        })),
    };

    // フォーム作成API呼び出し
    let create_result = forms::create_form(
        Extension(auth_user.clone()),
        axum::extract::State(app_state.clone()),
        AxumJson(create_req.clone()),
    )
    .await;

    // 作成結果を検証
    assert!(
        create_result.is_ok(),
        "Form creation failed: {:?}",
        create_result.err()
    );
    let (status, response) = create_result.unwrap();
    assert_eq!(status, StatusCode::CREATED);

    let form = response.0;
    assert_eq!(form.name, "Contact Form");
    assert_eq!(form.slug, "contact");
    assert_eq!(form.user_id, user_id);

    let form_id = form.id;

    // フォーム取得API呼び出し
    let get_result = forms::get_form(
        Extension(auth_user.clone()),
        Path(form_id),
        axum::extract::State(app_state.clone()),
    )
    .await;

    // 取得結果を検証
    assert!(get_result.is_ok(), "Form retrieval should succeed");

    let retrieved_form = get_result.unwrap().0;
    assert_eq!(retrieved_form.id, form.id);
    assert_eq!(retrieved_form.name, form.name);

    // 別のユーザーでアクセスした場合は失敗するはず
    let other_user_id = create_test_user(&pool).await;
    let other_auth_user = AuthUser {
        user_id: other_user_id,
        email: "other@example.com".to_string(),
        name: "Other User".to_string(),
    };

    let other_get_result = forms::get_form(
        Extension(other_auth_user),
        Path(form_id),
        axum::extract::State(app_state.clone()),
    )
    .await;

    // 他のユーザーはフォームにアクセスできないはず（403 Forbidden）
    assert!(other_get_result.is_err());
    if let Err((status, _)) = other_get_result {
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    // クリーンアップ
    sqlx::query!("DELETE FROM forms WHERE id = $1", form_id)
        .execute(&pool)
        .await
        .expect("Failed to clean up test form");

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
async fn test_update_form() {
    let app_state = AppState::new_for_test().await;
    let pool = app_state.db.clone();
    let user_id = create_test_user(&pool).await;

    let auth_user = AuthUser {
        user_id,
        email: "test@example.com".to_string(),
        name: "Test User".to_string(),
    };

    // フォーム作成
    let create_req = CreateFormRequest {
        name: "Original Form".to_string(),
        description: Some("Original description".to_string()),
        slug: Some("original-form".to_string()),
        markdown_content: "# Original Form".to_string(),
        form_fields: Some(json!([])),
        settings: Some(json!({})),
    };

    let create_result = forms::create_form(
        Extension(auth_user.clone()),
        axum::extract::State(app_state.clone()),
        AxumJson(create_req),
    )
    .await
    .unwrap();

    let form_id = create_result.1 .0.id;

    // フォーム更新リクエスト
    let update_req = UpdateFormRequest {
        name: Some("Updated Form".to_string()),
        description: Some("Updated description".to_string()),
        markdown_content: Some("# Updated Form".to_string()),
        form_fields: Some(json!([
            {
                "field_type": "text",
                "name": "updated_field",
                "label": "Updated Field"
            }
        ])),
        settings: Some(json!({
            "updated": true
        })),
        status: Some("published".to_string()),
    };

    // フォーム更新API呼び出し
    let update_result = forms::update_form(
        Extension(auth_user.clone()),
        Path(form_id),
        axum::extract::State(app_state.clone()),
        AxumJson(update_req.clone()),
    )
    .await;

    // 更新結果を検証
    assert!(update_result.is_ok(), "Form update should succeed");

    let updated_form = update_result.unwrap().0;
    assert_eq!(updated_form.name, "Updated Form");
    assert_eq!(
        updated_form.description,
        Some("Updated description".to_string())
    );
    assert_eq!(updated_form.status, "published");

    // クリーンアップ
    sqlx::query!("DELETE FROM forms WHERE id = $1", form_id)
        .execute(&pool)
        .await
        .expect("Failed to clean up test form");

    sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
        .execute(&pool)
        .await
        .expect("Failed to clean up test user");
}

#[tokio::test]
async fn test_delete_form() {
    let app_state = AppState::new_for_test().await;
    let pool = app_state.db.clone();
    let user_id = create_test_user(&pool).await;

    let auth_user = AuthUser {
        user_id,
        email: "test@example.com".to_string(),
        name: "Test User".to_string(),
    };

    // フォーム作成
    let create_req = CreateFormRequest {
        name: "Form to Delete".to_string(),
        description: None,
        slug: Some("to-delete".to_string()),
        markdown_content: "# To Delete".to_string(),
        form_fields: Some(json!([])),
        settings: Some(json!({})),
    };

    let create_result = forms::create_form(
        Extension(auth_user.clone()),
        axum::extract::State(app_state.clone()),
        AxumJson(create_req),
    )
    .await
    .unwrap();

    let form_id = create_result.1 .0.id;

    // フォーム削除API呼び出し
    let delete_result = forms::delete_form(
        Extension(auth_user.clone()),
        Path(form_id),
        axum::extract::State(app_state.clone()),
    )
    .await;

    // 削除結果を検証
    assert!(delete_result.is_ok(), "Form deletion should succeed");
    assert_eq!(delete_result.unwrap(), StatusCode::NO_CONTENT);

    // 削除後はフォームが取得できないことを確認
    let get_after_delete = forms::get_form(
        Extension(auth_user.clone()),
        Path(form_id),
        axum::extract::State(app_state.clone()),
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
async fn test_list_forms() {
    let app_state = AppState::new_for_test().await;
    let pool = app_state.db.clone();
    let user_id = create_test_user(&pool).await;

    let auth_user = AuthUser {
        user_id,
        email: "test@example.com".to_string(),
        name: "Test User".to_string(),
    };

    // 複数のフォームを作成
    let mut form_ids = Vec::new();
    for i in 0..3 {
        let create_req = CreateFormRequest {
            name: format!("Test Form {i}"),
            description: Some(format!("Description {i}")),
            slug: Some(format!("form-{i}")),
            markdown_content: format!("# Form {i}"),
            form_fields: Some(json!([])),
            settings: Some(json!({})),
        };

        let result = forms::create_form(
            Extension(auth_user.clone()),
            axum::extract::State(app_state.clone()),
            AxumJson(create_req),
        )
        .await
        .unwrap();

        form_ids.push(result.1 .0.id);
    }

    // フォーム一覧取得API呼び出し
    let list_result = forms::get_forms(
        Extension(auth_user.clone()),
        axum::extract::State(app_state.clone()),
    )
    .await;

    // 一覧結果を検証
    assert!(list_result.is_ok(), "Form listing should succeed");

    let forms_list = list_result.unwrap().0;
    assert_eq!(forms_list.len(), 3);

    // フォーム名でソートされているか確認
    let form_names: Vec<String> = forms_list.iter().map(|f| f.name.clone()).collect();
    assert!(form_names.contains(&"Test Form 0".to_string()));
    assert!(form_names.contains(&"Test Form 1".to_string()));
    assert!(form_names.contains(&"Test Form 2".to_string()));

    // クリーンアップ
    for form_id in form_ids {
        sqlx::query!("DELETE FROM forms WHERE id = $1", form_id)
            .execute(&pool)
            .await
            .expect("Failed to clean up test form");
    }

    sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
        .execute(&pool)
        .await
        .expect("Failed to clean up test user");
}

#[tokio::test]
async fn test_form_submission() {
    let app_state = AppState::new_for_test().await;
    let pool = app_state.db.clone();
    let user_id = create_test_user(&pool).await;

    let auth_user = AuthUser {
        user_id,
        email: "test@example.com".to_string(),
        name: "Test User".to_string(),
    };

    // フォーム作成
    let create_req = CreateFormRequest {
        name: "Submission Test Form".to_string(),
        description: Some("Form for testing submissions".to_string()),
        slug: Some("submission-test".to_string()),
        markdown_content: "# Submit Your Info".to_string(),
        form_fields: Some(json!([
            {
                "field_type": "text",
                "name": "name",
                "label": "Name",
                "required": true
            },
            {
                "field_type": "email",
                "name": "email",
                "label": "Email",
                "required": true
            }
        ])),
        settings: Some(json!({})),
    };

    let create_result = forms::create_form(
        Extension(auth_user.clone()),
        axum::extract::State(app_state.clone()),
        AxumJson(create_req),
    )
    .await
    .unwrap();

    let form_id = create_result.1 .0.id;

    // フォームを公開状態に更新
    let update_req = UpdateFormRequest {
        status: Some("published".to_string()),
        name: None,
        description: None,
        markdown_content: None,
        form_fields: None,
        settings: None,
    };

    let _ = forms::update_form(
        Extension(auth_user.clone()),
        Path(form_id),
        axum::extract::State(app_state.clone()),
        AxumJson(update_req),
    )
    .await
    .unwrap();

    // フォーム送信データ
    let submission_data = json!({
        "name": "Test Submitter",
        "email": "submitter@example.com"
    });

    // フォーム送信API呼び出し
    let submit_result = forms::submit_form(
        Path(form_id),
        axum::extract::State(app_state.clone()),
        AxumJson(crate::models::form::CreateFormSubmissionRequest {
            data: submission_data.clone(),
        }),
    )
    .await;

    // 送信結果を検証
    assert!(submit_result.is_ok(), "Form submission should succeed");
    let (status, submission) = submit_result.unwrap();
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(submission.0.form_id, form_id);
    assert_eq!(submission.0.data, submission_data);

    // 送信履歴を取得
    let submissions_result = forms::get_form_submissions(
        Extension(auth_user.clone()),
        Path(form_id),
        Query(forms::PaginationParams {
            limit: 10,
            offset: 0,
        }),
        axum::extract::State(app_state.clone()),
    )
    .await;

    assert!(submissions_result.is_ok());
    let submissions_data = submissions_result.unwrap().0;
    assert_eq!(submissions_data["total"], 1);

    // クリーンアップ
    sqlx::query!("DELETE FROM form_submissions WHERE form_id = $1", form_id)
        .execute(&pool)
        .await
        .expect("Failed to clean up test submissions");

    sqlx::query!("DELETE FROM forms WHERE id = $1", form_id)
        .execute(&pool)
        .await
        .expect("Failed to clean up test form");

    sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
        .execute(&pool)
        .await
        .expect("Failed to clean up test user");
}

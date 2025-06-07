use crate::{
    api::templates,
    middleware::auth::AuthUser,
    models::template::{CreateTemplateRequest, PreviewTemplateRequest, UpdateTemplateRequest},
    AppState,
};
use axum::{
    extract::{Extension, Json as AxumJson, Path, Query},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::sync::Arc;
use uuid::Uuid;

// テスト用のヘルパー関数
async fn setup_test_db() -> PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL environment variable must be set for tests");

    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database")
}

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

// 各テストは独立した環境で実行する必要があるため、モジュール全体のセットアップではなく
// 各テスト関数内でテスト環境をセットアップします。

#[tokio::test]
async fn test_create_and_get_template() {
    let pool = setup_test_db().await;
    let user_id = create_test_user(&pool).await;

    let app_state = AppState {
        db: pool.clone(),
        // 他のフィールドは必要に応じて設定
        config: Arc::new(crate::utils::config::Config::default()),
    };

    let auth_user = AuthUser {
        user_id,
        email: "test@example.com".to_string(),
        name: "Test User".to_string(),
    };

    // テンプレート作成リクエスト
    let create_req = CreateTemplateRequest {
        name: "Test Template".to_string(),
        subject_template: "Test Subject with {{variable}}".to_string(),
        markdown_content: "# Test Content\n\nWith {{variable}}".to_string(),
        variables: Some(json!({
            "variable": "test value"
        })),
        is_public: Some(false),
    };

    // テンプレート作成API呼び出し
    let create_result = templates::create_template(
        Extension(auth_user.clone()),
        axum::extract::State(app_state.clone()),
        AxumJson(create_req.clone()),
    )
    .await;

    // 作成結果を検証
    assert!(create_result.is_ok(), "Template creation should succeed");

    let template_response = create_result.unwrap().0;
    let template_id = Uuid::parse_str(&template_response.id).unwrap();

    assert_eq!(template_response.name, create_req.name);
    assert_eq!(
        template_response.subject_template,
        create_req.subject_template
    );
    assert_eq!(
        template_response.markdown_content,
        create_req.markdown_content
    );

    // テンプレート取得API呼び出し
    let get_result = templates::get_template(
        Extension(auth_user.clone()),
        axum::extract::State(app_state.clone()),
        Path(template_id),
    )
    .await;

    // 取得結果を検証
    assert!(get_result.is_ok(), "Template retrieval should succeed");

    let retrieved_template = get_result.unwrap().0;
    assert_eq!(retrieved_template.id, template_response.id);
    assert_eq!(retrieved_template.name, create_req.name);

    // 別のユーザーでアクセスした場合は失敗するはず
    let other_user_id = create_test_user(&pool).await;
    let other_auth_user = AuthUser {
        user_id: other_user_id,
        email: "other@example.com".to_string(),
        name: "Other User".to_string(),
    };

    let other_get_result = templates::get_template(
        Extension(other_auth_user),
        axum::extract::State(app_state.clone()),
        Path(template_id),
    )
    .await;

    // 他のユーザーは非公開テンプレートにアクセスできないはず
    assert!(other_get_result.is_err());
    if let Err((status, _)) = other_get_result {
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    // テスト後のクリーンアップ
    sqlx::query!("DELETE FROM templates WHERE id = $1", template_id)
        .execute(&pool)
        .await
        .expect("Failed to clean up test template");

    sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
        .execute(&pool)
        .await
        .expect("Failed to clean up test user");

    sqlx::query!("DELETE FROM users WHERE id = $1", other_user_id)
        .execute(&pool)
        .await
        .expect("Failed to clean up other test user");
}

#[tokio::test]
async fn test_update_template() {
    let pool = setup_test_db().await;
    let user_id = create_test_user(&pool).await;

    let app_state = AppState {
        db: pool.clone(),
        config: Arc::new(crate::utils::config::Config::default()),
    };

    let auth_user = AuthUser {
        user_id,
        email: "test@example.com".to_string(),
        name: "Test User".to_string(),
    };

    // テンプレート作成
    let create_req = CreateTemplateRequest {
        name: "Template to Update".to_string(),
        subject_template: "Original Subject".to_string(),
        markdown_content: "# Original Content".to_string(),
        variables: None,
        is_public: None,
    };

    let create_result = templates::create_template(
        Extension(auth_user.clone()),
        axum::extract::State(app_state.clone()),
        AxumJson(create_req),
    )
    .await
    .unwrap()
    .0;

    let template_id = Uuid::parse_str(&create_result.id).unwrap();

    // テンプレート更新リクエスト
    let update_req = UpdateTemplateRequest {
        name: Some("Updated Template".to_string()),
        subject_template: Some("Updated Subject".to_string()),
        markdown_content: Some("# Updated Content".to_string()),
        variables: Some(json!({
            "new_var": "new value"
        })),
        is_public: Some(true),
    };

    // テンプレート更新API呼び出し
    let update_result = templates::update_template(
        Extension(auth_user.clone()),
        axum::extract::State(app_state.clone()),
        Path(template_id),
        AxumJson(update_req.clone()),
    )
    .await;

    // 更新結果を検証
    assert!(update_result.is_ok(), "Template update should succeed");

    let updated_template = update_result.unwrap().0;
    assert_eq!(updated_template.name, update_req.name.unwrap());
    assert_eq!(
        updated_template.subject_template,
        update_req.subject_template.unwrap()
    );
    assert_eq!(
        updated_template.markdown_content,
        update_req.markdown_content.unwrap()
    );
    assert_eq!(updated_template.is_public, update_req.is_public.unwrap());

    // テンプレート取得して検証
    let get_result = templates::get_template(
        Extension(auth_user.clone()),
        axum::extract::State(app_state.clone()),
        Path(template_id),
    )
    .await
    .unwrap()
    .0;

    assert_eq!(get_result.name, update_req.name.unwrap());

    // テスト後のクリーンアップ
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
async fn test_delete_template() {
    let pool = setup_test_db().await;
    let user_id = create_test_user(&pool).await;

    let app_state = AppState {
        db: pool.clone(),
        config: Arc::new(crate::utils::config::Config::default()),
    };

    let auth_user = AuthUser {
        user_id,
        email: "test@example.com".to_string(),
        name: "Test User".to_string(),
    };

    // テンプレート作成
    let create_req = CreateTemplateRequest {
        name: "Template to Delete".to_string(),
        subject_template: "Subject".to_string(),
        markdown_content: "# Content".to_string(),
        variables: None,
        is_public: None,
    };

    let create_result = templates::create_template(
        Extension(auth_user.clone()),
        axum::extract::State(app_state.clone()),
        AxumJson(create_req),
    )
    .await
    .unwrap()
    .0;

    let template_id = Uuid::parse_str(&create_result.id).unwrap();

    // テンプレート削除API呼び出し
    let delete_result = templates::delete_template(
        Extension(auth_user.clone()),
        axum::extract::State(app_state.clone()),
        Path(template_id),
    )
    .await;

    // 削除結果を検証
    assert!(delete_result.is_ok(), "Template deletion should succeed");

    // 削除後はテンプレートが取得できないことを確認
    let get_after_delete = templates::get_template(
        Extension(auth_user.clone()),
        axum::extract::State(app_state.clone()),
        Path(template_id),
    )
    .await;

    assert!(get_after_delete.is_err());
    if let Err((status, _)) = get_after_delete {
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    // テスト後のクリーンアップ
    sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
        .execute(&pool)
        .await
        .expect("Failed to clean up test user");
}

#[tokio::test]
async fn test_template_preview() {
    let pool = setup_test_db().await;
    let user_id = create_test_user(&pool).await;

    let app_state = AppState {
        db: pool.clone(),
        config: Arc::new(crate::utils::config::Config::default()),
    };

    let auth_user = AuthUser {
        user_id,
        email: "test@example.com".to_string(),
        name: "Test User".to_string(),
    };

    // テンプレート作成（変数含む）
    let create_req = CreateTemplateRequest {
        name: "Template with Variables".to_string(),
        subject_template: "Subject with {{var1}}".to_string(),
        markdown_content: "# Content with {{var1}} and {{var2}}".to_string(),
        variables: Some(json!({
            "var1": "default1",
            "var2": "default2"
        })),
        is_public: None,
    };

    let create_result = templates::create_template(
        Extension(auth_user.clone()),
        axum::extract::State(app_state.clone()),
        AxumJson(create_req),
    )
    .await
    .unwrap()
    .0;

    let template_id = Uuid::parse_str(&create_result.id).unwrap();

    // プレビューリクエスト作成
    let preview_req = PreviewTemplateRequest {
        variables: json!({
            "var1": "preview1",
            "var2": "preview2"
        }),
    };

    // プレビューAPI呼び出し
    let preview_result = templates::preview_template(
        Extension(auth_user.clone()),
        axum::extract::State(app_state.clone()),
        Path(template_id),
        AxumJson(preview_req),
    )
    .await;

    // プレビュー結果を検証
    assert!(preview_result.is_ok(), "Template preview should succeed");

    let preview_response = preview_result.unwrap().0;
    assert!(preview_response.html.contains("preview1"));
    assert!(preview_response.html.contains("preview2"));
    assert_eq!(preview_response.subject, "Subject with preview1");

    // テスト後のクリーンアップ
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
async fn test_list_templates() {
    let pool = setup_test_db().await;
    let user_id = create_test_user(&pool).await;

    let app_state = AppState {
        db: pool.clone(),
        config: Arc::new(crate::utils::config::Config::default()),
    };

    let auth_user = AuthUser {
        user_id,
        email: "test@example.com".to_string(),
        name: "Test User".to_string(),
    };

    // 複数のテンプレートを作成
    for i in 1..4 {
        let create_req = CreateTemplateRequest {
            name: format!("Test Template {}", i),
            subject_template: format!("Test Subject {}", i),
            markdown_content: format!("# Test Content {}", i),
            variables: None,
            is_public: None,
        };

        templates::create_template(
            Extension(auth_user.clone()),
            axum::extract::State(app_state.clone()),
            AxumJson(create_req),
        )
        .await
        .unwrap();
    }

    // クエリパラメータ作成
    let query = Query(templates::ListTemplatesQuery {
        limit: Some(10),
        offset: Some(0),
    });

    // 一覧API呼び出し
    let list_result = templates::list_templates(
        Extension(auth_user.clone()),
        query,
        axum::extract::State(app_state.clone()),
    )
    .await;

    // 一覧結果を検証
    assert!(list_result.is_ok(), "Template listing should succeed");

    let templates_list = list_result.unwrap().0;
    assert_eq!(templates_list.templates.len(), 3);
    assert_eq!(templates_list.total, 3);
    assert_eq!(templates_list.limit, 10);
    assert_eq!(templates_list.offset, 0);

    // テスト後のクリーンアップ
    sqlx::query!("DELETE FROM templates WHERE user_id = $1", user_id)
        .execute(&pool)
        .await
        .expect("Failed to clean up test templates");

    sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
        .execute(&pool)
        .await
        .expect("Failed to clean up test user");
}

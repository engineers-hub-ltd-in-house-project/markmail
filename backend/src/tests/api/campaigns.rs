use axum::{
    body::{self, Body},
    extract::rejection::JsonRejection,
    http::{Method, Request, StatusCode},
};
use chrono::Utc;
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

use crate::{
    create_app,
    models::{
        campaign::{CampaignResponse, CreateCampaignRequest},
        template::Template,
        user::User,
    },
    tests::api::templates::{create_test_template, get_test_user_with_jwt},
    AppState,
};

// テスト用のキャンペーン作成リクエスト
fn get_test_campaign_request(template_id: Uuid) -> CreateCampaignRequest {
    CreateCampaignRequest {
        name: "テストキャンペーン".to_string(),
        description: Some("テスト用のキャンペーンです".to_string()),
        subject: "テストメールの件名".to_string(),
        template_id,
    }
}

// テスト用のキャンペーンを作成
pub async fn create_test_campaign(
    app: &axum::Router,
    user: &User,
    auth_token: &str,
    template_id: Uuid,
) -> Result<CampaignResponse, (StatusCode, Value)> {
    let request = get_test_campaign_request(template_id);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/campaigns")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", auth_token))
                .body(Body::from(serde_json::to_string(&request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = body::to_bytes(response.into_body()).await.unwrap();
    let body: Value = serde_json::from_slice(&body).unwrap();

    if status != StatusCode::OK {
        return Err((status, body));
    }

    let campaign: CampaignResponse = serde_json::from_value(body).unwrap();
    Ok(campaign)
}

#[tokio::test]
async fn test_create_campaign() {
    // テスト用のアプリとDBプールの準備
    let app_state = AppState::new_for_test().await;
    let app = create_app(app_state.clone());

    // テストユーザーとJWTの取得
    let (user, token) = get_test_user_with_jwt(&app).await;

    // テストテンプレートの作成
    let template = create_test_template(&app, &user, &token).await.unwrap();

    // キャンペーン作成
    let campaign_request = get_test_campaign_request(template.id);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/campaigns")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::from(
                    serde_json::to_string(&campaign_request).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = body::to_bytes(response.into_body()).await.unwrap();
    let campaign_response: CampaignResponse = serde_json::from_slice(&body).unwrap();

    // レスポンスの検証
    assert_eq!(campaign_response.name, campaign_request.name);
    assert_eq!(campaign_response.subject, campaign_request.subject);
    assert_eq!(campaign_response.description, campaign_request.description);
    assert_eq!(campaign_response.template_id, template.id);
    assert_eq!(campaign_response.status, "draft");

    // デフォルト値の検証
    assert_eq!(campaign_response.stats.recipient_count, 0);
    assert_eq!(campaign_response.stats.sent_count, 0);
    assert_eq!(campaign_response.stats.opened_count, 0);
    assert_eq!(campaign_response.stats.clicked_count, 0);
    assert_eq!(campaign_response.stats.open_rate, 0.0);
    assert_eq!(campaign_response.stats.click_rate, 0.0);
}

#[tokio::test]
async fn test_get_campaign() {
    // テスト用のアプリとDBプールの準備
    let app_state = AppState::new_for_test().await;
    let app = create_app(app_state.clone());

    // テストユーザーとJWTの取得
    let (user, token) = get_test_user_with_jwt(&app).await;

    // テストテンプレートの作成
    let template = create_test_template(&app, &user, &token).await.unwrap();

    // テストキャンペーンの作成
    let campaign = create_test_campaign(&app, &user, &token, template.id)
        .await
        .unwrap();

    // キャンペーン取得
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/campaigns/{}", campaign.id))
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = body::to_bytes(response.into_body()).await.unwrap();
    let campaign_response: CampaignResponse = serde_json::from_slice(&body).unwrap();

    // レスポンスの検証
    assert_eq!(campaign_response.id, campaign.id);
    assert_eq!(campaign_response.name, campaign.name);
    assert_eq!(campaign_response.template_id, template.id);
}

#[tokio::test]
async fn test_update_campaign() {
    // テスト用のアプリとDBプールの準備
    let app_state = AppState::new_for_test().await;
    let app = create_app(app_state.clone());

    // テストユーザーとJWTの取得
    let (user, token) = get_test_user_with_jwt(&app).await;

    // テストテンプレートの作成
    let template = create_test_template(&app, &user, &token).await.unwrap();

    // テストキャンペーンの作成
    let campaign = create_test_campaign(&app, &user, &token, template.id)
        .await
        .unwrap();

    // キャンペーン更新リクエスト
    let update_request = json!({
        "name": "更新されたキャンペーン名",
        "description": "更新された説明文",
        "subject": "更新された件名"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri(format!("/api/campaigns/{}", campaign.id))
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::from(serde_json::to_string(&update_request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = body::to_bytes(response.into_body()).await.unwrap();
    let updated_campaign: CampaignResponse = serde_json::from_slice(&body).unwrap();

    // 更新が反映されていることを確認
    assert_eq!(updated_campaign.id, campaign.id);
    assert_eq!(updated_campaign.name, "更新されたキャンペーン名");
    assert_eq!(
        updated_campaign.description,
        Some("更新された説明文".to_string())
    );
    assert_eq!(updated_campaign.subject, "更新された件名");
    assert_eq!(updated_campaign.template_id, template.id);
    assert_eq!(updated_campaign.status, "draft");
}

#[tokio::test]
async fn test_schedule_campaign() {
    // テスト用のアプリとDBプールの準備
    let app_state = AppState::new_for_test().await;
    let app = create_app(app_state.clone());

    // テストユーザーとJWTの取得
    let (user, token) = get_test_user_with_jwt(&app).await;

    // テストテンプレートの作成
    let template = create_test_template(&app, &user, &token).await.unwrap();

    // テストキャンペーンの作成
    let campaign = create_test_campaign(&app, &user, &token, template.id)
        .await
        .unwrap();

    // スケジュールリクエスト（未来の日時）
    let schedule_date = (Utc::now() + chrono::Duration::days(1)).to_rfc3339();
    let schedule_request = json!({
        "scheduled_at": schedule_date
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/campaigns/{}/schedule", campaign.id))
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::from(
                    serde_json::to_string(&schedule_request).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = body::to_bytes(response.into_body()).await.unwrap();
    let scheduled_campaign: CampaignResponse = serde_json::from_slice(&body).unwrap();

    // ステータスとスケジュール日時が更新されていることを確認
    assert_eq!(scheduled_campaign.status, "scheduled");
    assert!(scheduled_campaign.scheduled_at.is_some());
}

#[tokio::test]
#[ignore] // 実際のDBが必要なのでignore
async fn test_list_campaigns() {
    // テスト用のアプリとDBプールの準備
    let app_state = AppState::new_for_test().await;
    let app = create_app(app_state.clone());

    // テストユーザーとJWTの取得
    let (user, token) = get_test_user_with_jwt(&app).await;

    // テストテンプレートの作成
    let template = create_test_template(&app, &user, &token).await.unwrap();

    // 複数のテストキャンペーンを作成
    for i in 1..=3 {
        let campaign_request = CreateCampaignRequest {
            name: format!("テストキャンペーン {}", i),
            description: Some(format!("テスト説明 {}", i)),
            subject: format!("テスト件名 {}", i),
            template_id: template.id,
        };

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/campaigns")
                    .header("Content-Type", "application/json")
                    .header("Authorization", format!("Bearer {}", token))
                    .body(Body::from(
                        serde_json::to_string(&campaign_request).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    // キャンペーン一覧を取得
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/campaigns")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = body::to_bytes(response.into_body()).await.unwrap();
    let campaigns_response: Value = serde_json::from_slice(&body).unwrap();

    // 少なくとも作成したキャンペーンの数だけあることを確認
    let campaigns = campaigns_response["campaigns"].as_array().unwrap();
    assert!(campaigns.len() >= 3);

    // ページネーションパラメータの確認
    assert!(campaigns_response["total"].as_i64().unwrap() >= 3);
    assert_eq!(campaigns_response["limit"].as_i64().unwrap(), 50);
    assert_eq!(campaigns_response["offset"].as_i64().unwrap(), 0);
}

#[tokio::test]
#[ignore] // 実際のDBが必要なのでignore
async fn test_campaign_preview() {
    // テスト用のアプリとDBプールの準備
    let app_state = AppState::new_for_test().await;
    let app = create_app(app_state.clone());

    // テストユーザーとJWTの取得
    let (user, token) = get_test_user_with_jwt(&app).await;

    // マークダウンを含むテストテンプレートの作成
    let template_request = json!({
        "name": "プレビューテスト用テンプレート",
        "subject_template": "{{name}}様 テストメール",
        "markdown_content": "# こんにちは {{name}} 様\n\n{{company}}からのお知らせです。\n\n**重要なお知らせ**があります。\n\n[ログイン]({{login_url}})\n\n配信停止は[こちら]({{unsubscribe_url}})",
        "variables": {
            "name": "ユーザー名",
            "company": "会社名",
            "login_url": "ログインURL",
            "unsubscribe_url": "配信停止URL"
        }
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/templates")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::from(
                    serde_json::to_string(&template_request).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = body::to_bytes(response.into_body()).await.unwrap();
    let template_response: Value = serde_json::from_slice(&body).unwrap();
    let template_id = Uuid::parse_str(template_response["id"].as_str().unwrap()).unwrap();

    // テストキャンペーンの作成
    let campaign = create_test_campaign(&app, &user, &token, template_id)
        .await
        .unwrap();

    // キャンペーンプレビューを取得
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/campaigns/{}/preview", campaign.id))
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = body::to_bytes(response.into_body()).await.unwrap();
    let preview_response: Value = serde_json::from_slice(&body).unwrap();

    // HTMLが生成されていることを確認
    let html = preview_response["html"].as_str().unwrap();
    assert!(html.contains("<h1>こんにちは テストユーザー 様</h1>"));
    assert!(html.contains("サンプル株式会社からのお知らせです"));
    assert!(html.contains("<strong>重要なお知らせ</strong>"));
    assert!(html.contains("href=\"https://markmail.example.com/login\""));
    assert!(html.contains("href=\"https://markmail.example.com/unsubscribe?id=12345\""));
}

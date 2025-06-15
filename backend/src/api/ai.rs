use axum::{
    extract::{Extension, Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::{
    ai::{
        models::{
            ai_responses::{
                GenerateContentRequest, GenerateContentResponse, OptimizeSubjectRequest,
                OptimizeSubjectResponse,
            },
            GenerateScenarioRequest, GenerateScenarioResponse,
        },
        providers::{create_ai_provider, AIProviderConfig, AIProviderType},
        services::{
            content_generator::ContentGeneratorService, scenario_builder::ScenarioBuilderService,
        },
    },
    middleware::auth::AuthUser,
    AppState,
};

/// AIプロバイダーを取得または作成
async fn get_ai_provider(
    _state: &AppState,
) -> Result<Arc<dyn crate::ai::AIProvider>, (StatusCode, Json<Value>)> {
    // 環境変数から設定を読み込み
    let provider_type = std::env::var("AI_PROVIDER").unwrap_or_else(|_| "openai".to_string());

    // デバッグ: 環境変数の確認
    tracing::debug!("AI_PROVIDER: {}", provider_type);
    tracing::debug!(
        "OPENAI_API_KEY exists: {}",
        std::env::var("OPENAI_API_KEY").is_ok()
    );

    let api_key = match provider_type.as_str() {
        "anthropic" => std::env::var("ANTHROPIC_API_KEY").map_err(|e| {
            tracing::error!("ANTHROPIC_API_KEY not found: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "ANTHROPIC_API_KEY not set"
                })),
            )
        })?,
        _ => std::env::var("OPENAI_API_KEY").map_err(|e| {
            tracing::error!("OPENAI_API_KEY not found: {:?}", e);
            tracing::error!(
                "Available env vars: {:?}",
                std::env::vars().collect::<Vec<_>>()
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "OPENAI_API_KEY not set"
                })),
            )
        })?,
    };

    let model = match provider_type.as_str() {
        "anthropic" => std::env::var("ANTHROPIC_MODEL")
            .unwrap_or_else(|_| "claude-3-opus-20240229".to_string()),
        _ => std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4".to_string()),
    };

    let config = AIProviderConfig {
        provider_type: match provider_type.as_str() {
            "anthropic" => AIProviderType::Anthropic,
            _ => AIProviderType::OpenAI,
        },
        api_key,
        model,
        max_retries: 3,
        timeout_seconds: 60,
    };

    create_ai_provider(config).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": format!("Failed to create AI provider: {}", e)
            })),
        )
    })
}

/// シナリオ生成エンドポイント
pub async fn generate_scenario(
    Extension(_auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Json(request): Json<GenerateScenarioRequest>,
) -> Result<Json<GenerateScenarioResponse>, (StatusCode, Json<Value>)> {
    tracing::info!("generate_scenario called with request: {:?}", request);
    let provider = get_ai_provider(&state).await?;
    let service = ScenarioBuilderService::new(provider);

    let response = service.generate_scenario(request).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": format!("Failed to generate scenario: {}", e)
            })),
        )
    })?;

    Ok(Json(response))
}

/// コンテンツ生成エンドポイント
pub async fn generate_content(
    Extension(_auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Json(request): Json<GenerateContentRequest>,
) -> Result<Json<GenerateContentResponse>, (StatusCode, Json<Value>)> {
    let provider = get_ai_provider(&state).await?;
    let service = ContentGeneratorService::new(provider);

    let response = service.generate_content(request).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": format!("Failed to generate content: {}", e)
            })),
        )
    })?;

    Ok(Json(response))
}

/// 件名最適化エンドポイント
pub async fn optimize_subject(
    Extension(_auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Json(request): Json<OptimizeSubjectRequest>,
) -> Result<Json<OptimizeSubjectResponse>, (StatusCode, Json<Value>)> {
    let provider = get_ai_provider(&state).await?;
    let service = ContentGeneratorService::new(provider);

    let response = service.optimize_subject(request).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": format!("Failed to optimize subject: {}", e)
            })),
        )
    })?;

    Ok(Json(response))
}

/// ヘルスチェックエンドポイント
pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, "AI service is healthy")
}

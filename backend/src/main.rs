use axum::{http::StatusCode, response::Json, routing::get, Router};
use serde_json::{json, Value};
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use markmail_backend::{api, database, utils, workers, AppState};

#[tokio::main]
async fn main() {
    // ログ設定
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "markmail_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 環境変数読み込み
    dotenvy::dotenv().ok();

    // 設定ロード
    let config = std::sync::Arc::new(utils::config::Config::new());

    // データベース接続プール作成
    let pool = database::connection::create_pool(&config.database_url, 20)
        .await
        .expect("データベースに接続できませんでした");

    // Redis接続
    let redis_client = redis::Client::open(config.redis_url.clone())
        .expect("Redisクライアントの作成に失敗しました");

    // アプリケーション状態
    let app_state = AppState {
        db: pool.clone(),
        redis: redis_client,
        config,
    };

    // シーケンスワーカーを起動
    workers::sequence_worker::spawn_sequence_worker(std::sync::Arc::new(pool));

    // ルーター作成
    let app = create_app(app_state);

    // サーバー起動
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .expect("PORTは有効な数値である必要があります");

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    tracing::info!("MarkMail バックエンドサーバーを起動中... http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn create_app(state: AppState) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .merge(api::create_routes())
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(
                    CorsLayer::new()
                        .allow_origin(Any)
                        .allow_methods(Any)
                        .allow_headers(Any),
                ),
        )
        .with_state(state)
}

async fn root() -> Json<Value> {
    Json(json!({
        "message": "MarkMail API へようこそ！",
        "version": "0.1.0",
        "status": "running"
    }))
}

async fn health_check() -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

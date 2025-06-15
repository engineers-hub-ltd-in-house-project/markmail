//! MarkMail バックエンドライブラリ

pub mod ai;
pub mod api;
pub mod database;
pub mod middleware;
pub mod models;
pub mod services;
pub mod utils;
pub mod workers;

#[cfg(test)]
pub mod tests;

#[cfg(test)]
pub use create_test_app as create_app;

#[cfg(test)]
pub async fn create_test_app() -> (
    axum::Router,
    sqlx::PgPool,
    redis::Client,
    std::sync::Arc<utils::config::Config>,
) {
    use middleware::{cors::cors_layer, logging::logging_layer};

    dotenvy::dotenv().ok();

    // テスト用のAppStateを作成
    let app_state = AppState::new_for_test().await;
    let pool = app_state.db.clone();
    let redis = app_state.redis.clone();
    let config = app_state.config.clone();

    // ルーターを構築
    let router = api::create_routes()
        .layer(logging_layer())
        .layer(cors_layer())
        .with_state(app_state);

    (router, pool, redis, config)
}

use axum::extract::FromRef;
use sqlx::PgPool;
use std::sync::Arc;
use utils::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: redis::Client,
    pub config: Arc<Config>,
}

impl AppState {
    #[cfg(test)]
    pub async fn new_for_test() -> Self {
        dotenvy::dotenv().ok();

        // テスト用の設定
        let config = Arc::new(Config::new());

        // テスト用のDBに接続
        let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
            "postgres://markmail:markmail_password@localhost:5432/markmail_test".to_string()
        });

        let pool = database::connection::create_pool(&database_url, 5)
            .await
            .expect("テストDB接続に失敗しました");

        // テスト用のRedis
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

        let redis_client =
            redis::Client::open(redis_url.clone()).expect("Redisクライアントの作成に失敗しました");

        Self {
            db: pool,
            redis: redis_client,
            config,
        }
    }
}

// FromRefの実装により、State<AppState>からState<PgPool>などの抽出が可能になる
impl FromRef<AppState> for PgPool {
    fn from_ref(state: &AppState) -> Self {
        state.db.clone()
    }
}

impl FromRef<AppState> for Arc<Config> {
    fn from_ref(state: &AppState) -> Self {
        state.config.clone()
    }
}

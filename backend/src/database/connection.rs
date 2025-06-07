use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use std::str::FromStr;
use std::time::Duration;

pub async fn create_pool(database_url: &str, max_connections: u32) -> Result<PgPool, sqlx::Error> {
    tracing::debug!("データベース接続プールを作成します: {}", database_url);

    // 開発環境ではプリペアドステートメントキャッシュを無効化
    let mut connect_options = PgConnectOptions::from_str(database_url)?;
    if cfg!(debug_assertions) || std::env::var("DISABLE_STATEMENT_CACHE").is_ok() {
        connect_options = connect_options.statement_cache_capacity(0);
        tracing::warn!("プリペアドステートメントキャッシュを無効化しました");
    }

    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .acquire_timeout(Duration::from_secs(30))
        .connect_with(connect_options)
        .await?;

    tracing::debug!("データベースマイグレーションを実行します");

    // マイグレーションを実行する
    match sqlx::migrate!().run(&pool).await {
        Ok(_) => {
            tracing::info!("マイグレーションが完了しました");
            Ok(pool)
        }
        Err(e) => {
            tracing::error!("マイグレーションエラー: {:?}", e);
            Err(e.into())
        }
    }
}

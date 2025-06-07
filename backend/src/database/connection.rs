use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

pub async fn create_pool(database_url: &str, max_connections: u32) -> Result<PgPool, sqlx::Error> {
    tracing::debug!("データベース接続プールを作成します: {}", database_url);

    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .acquire_timeout(Duration::from_secs(30))
        .connect(database_url)
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

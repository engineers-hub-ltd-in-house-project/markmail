use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::{error, info};

use crate::services::sequence_service::SequenceService;

pub struct SequenceWorker {
    pool: Arc<PgPool>,
    service: SequenceService,
    interval_seconds: u64,
}

impl SequenceWorker {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self {
            pool,
            service: SequenceService::new(),
            interval_seconds: 60, // 1分ごとに実行
        }
    }

    pub fn with_interval(mut self, seconds: u64) -> Self {
        self.interval_seconds = seconds;
        self
    }

    /// ワーカーを開始
    pub async fn start(self) {
        info!(
            "Starting sequence worker with {}s interval",
            self.interval_seconds
        );

        let mut ticker = interval(Duration::from_secs(self.interval_seconds));

        loop {
            ticker.tick().await;

            if let Err(e) = self.process_sequences().await {
                error!("Error processing sequences: {}", e);
            }
        }
    }

    /// シーケンスの処理を実行
    async fn process_sequences(&self) -> Result<(), String> {
        info!("Processing pending sequence steps...");

        // 実行待ちのシーケンスステップを処理
        self.service
            .process_pending_sequence_steps(&self.pool)
            .await?;

        info!("Sequence processing completed");
        Ok(())
    }
}

/// バックグラウンドワーカーを起動する関数
pub fn spawn_sequence_worker(pool: Arc<PgPool>) {
    let worker = SequenceWorker::new(pool);

    tokio::spawn(async move {
        worker.start().await;
    });

    info!("Sequence worker spawned");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_worker_creation() {
        // ダミーのプールを作成（実際のテストでは適切なテストDBを使用）
        let pool = Arc::new(PgPool::new(""));
        let worker = SequenceWorker::new(pool);

        assert_eq!(worker.interval_seconds, 60);
    }

    #[tokio::test]
    async fn test_worker_with_custom_interval() {
        let pool = Arc::new(PgPool::new(""));
        let worker = SequenceWorker::new(pool).with_interval(30);

        assert_eq!(worker.interval_seconds, 30);
    }
}

use std::sync::Arc;

use lapin::options::{BasicAckOptions, BasicRejectOptions};
use serde::Serialize;
use tokio::sync::{Semaphore, mpsc};

use crate::config::Config;
use crate::db::Database;
use crate::queue::{QueueError, RabbitMQ};
use crate::services::job_worker::{JobWorker, JobWorkerResult};

#[derive(Debug, thiserror::Error)]
pub enum JobManagerError {
    #[error("queue error: {0}")]
    Queue(#[from] QueueError),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Serialize)]
struct JobNotificationError {
    message: String,
    error: String,
}

pub struct JobManager<DB: sqlx::Database> {
    db: Database<DB>,
    config: Config,
    rabbitmq: RabbitMQ,
}

crate::db::impl_with_db! {
    impl<DB> JobManager<DB> {
    pub fn new(db: Database<DB>, config: Config, rabbitmq: RabbitMQ) -> Self {
        Self { db, config, rabbitmq }
    }

    pub async fn start(&mut self, concurrency: usize) -> Result<(), JobManagerError> {
        tracing::info!(concurrency, "job manager started, waiting for messages");
        let mut consumer_rx = self.rabbitmq.consume().await?;
        let semaphore = Arc::new(Semaphore::new(concurrency));
        let (result_tx, mut result_rx) = mpsc::channel::<JobWorkerResult>(concurrency);
        let worker = Arc::new(JobWorker::new(self.db.clone(), self.config.clone()));

        tokio::spawn(async move {
            while let Some(delivery) = consumer_rx.recv().await {
                let permit = match semaphore.clone().acquire_owned().await {
                    Ok(p) => p,
                    Err(_) => break,
                };
                let tx = result_tx.clone();
                let w = Arc::clone(&worker);

                tokio::spawn(async move {
                    let _permit = permit;
                    let result = w.process(delivery).await;
                    let _ = tx.send(result).await;
                });
            }
        });

        while let Some(result) = result_rx.recv().await {
            let outcome = if result.error.is_some() {
                self.handle_error(&result).await
            } else {
                self.notify_success(&result).await
            };

            if let Err(e) = outcome {
                tracing::error!("notification failed: {e}");
                let _ = result
                    .delivery
                    .reject(BasicRejectOptions { requeue: false })
                    .await;
            }
        }

        Ok(())
    }

    async fn notify_success(&self, result: &JobWorkerResult) -> Result<(), JobManagerError> {
        if let Some(ref job) = result.job {
            tracing::info!(
                job_id = %job.id,
                video_id = %job.video.id,
                status = %job.status,
                "job completed successfully"
            );
            let payload = serde_json::to_string(job)?;
            self.rabbitmq.notify_default(&payload).await?;
        }

        result
            .delivery
            .ack(BasicAckOptions::default())
            .await
            .map_err(QueueError::from)?;

        Ok(())
    }

    async fn handle_error(&self, result: &JobWorkerResult) -> Result<(), JobManagerError> {
        let error_str = result.error.as_deref().unwrap_or("unknown error");

        if let Some(ref job) = result.job {
            tracing::error!(
                job_id = %job.id,
                video_id = %job.video.id,
                "job failed: {error_str}"
            );
        } else {
            tracing::error!("message parse error: {error_str}");
        }

        let notification = JobNotificationError {
            message: String::from_utf8_lossy(&result.delivery.data).to_string(),
            error: error_str.to_string(),
        };

        let payload = serde_json::to_string(&notification)?;
        self.rabbitmq.notify_default(&payload).await?;

        result
            .delivery
            .reject(BasicRejectOptions { requeue: false })
            .await
            .map_err(QueueError::from)?;

        Ok(())
    }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::queue::QueueConfig;

    fn test_config() -> Config {
        Config {
            database_url: String::new(),
            local_storage_path: "/tmp".to_string(),
            input_bucket_name: "test".to_string(),
            output_bucket_name: "test-out".to_string(),
            concurrency: 1,
            concurrency_workers: 2,
            auto_migrate: false,
        }
    }

    fn test_queue_config() -> QueueConfig {
        QueueConfig {
            user: "guest".to_string(),
            password: "guest".to_string(),
            host: "localhost".to_string(),
            port: "5672".to_string(),
            vhost: "/".to_string(),
            consumer_queue_name: "videos".to_string(),
            consumer_name: "test-consumer".to_string(),
            dlx: "dlx".to_string(),
            notification_exchange: "amq.direct".to_string(),
            notification_routing_key: "jobs".to_string(),
        }
    }

    #[tokio::test]
    async fn new_creates_manager() {
        let db =
            crate::db::Database::<sqlx::Sqlite>::new("sqlite::memory:".to_string(), Some(true))
                .await
                .unwrap();

        let rabbitmq = RabbitMQ::new(test_queue_config());
        let _manager = JobManager::new(db, test_config(), rabbitmq);
    }

    #[tokio::test]
    async fn start_without_rabbitmq_connection_returns_error() {
        let db =
            crate::db::Database::<sqlx::Sqlite>::new("sqlite::memory:".to_string(), Some(true))
                .await
                .unwrap();

        let rabbitmq = RabbitMQ::new(test_queue_config());
        let mut manager = JobManager::new(db, test_config(), rabbitmq);

        let result = manager.start(2).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("channel unavailable")
        );
    }
}

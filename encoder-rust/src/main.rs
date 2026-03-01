use sqlx::Postgres;
use tracing_subscriber::EnvFilter;

use encoder_rust::config::Config;
use encoder_rust::db::Database;
use encoder_rust::queue::{QueueConfig, RabbitMQ};
use encoder_rust::services::JobManager;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    init_logs();

    let config = Config::from_env()?;
    let queue_config = QueueConfig::from_env()?;

    tracing::info!(
        storage = %config.local_storage_path,
        concurrency_upload = config.concurrency,
        concurrency_workers = config.concurrency_workers,
        auto_migrate = config.auto_migrate,
        "encoder started"
    );

    let db =
        Database::<Postgres>::new(config.database_url.clone(), Some(config.auto_migrate)).await?;
    tracing::info!("database connected");

    let mut rabbitmq = RabbitMQ::new(queue_config);
    rabbitmq.connect().await?;
    tracing::info!("rabbitmq connected");

    let workers = config.concurrency_workers;
    let mut job_manager = JobManager::new(db, config, rabbitmq);

    tokio::select! {
        result = job_manager.start(workers) => {
            if let Err(e) = result {
                tracing::error!("job manager error: {e}");
            }
        }
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("shutting down");
        }
    }

    Ok(())
}

fn init_logs() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .try_init()
        .ok();
}

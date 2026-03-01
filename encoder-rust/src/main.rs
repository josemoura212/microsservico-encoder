use tracing_subscriber::EnvFilter;

use encoder_rust::config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    init_logs();

    let config = Config::from_env()?;
    tracing::info!(
        storage = %config.local_storage_path,
        concurrency = config.concurrency,
        "encoder started"
    );

    tokio::signal::ctrl_c().await?;
    tracing::info!("shutting down");

    Ok(())
}

fn init_logs() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init()
        .ok();
}

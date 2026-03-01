use std::env;

pub const ENV_DATABASE_URL: &str = "DATABASE_URL";
pub const ENV_DATABASE_URL_TEST: &str = "DATABASE_URL_TEST";
pub const ENV_AUTO_MIGRATE_DB: &str = "AUTO_MIGRATE_DB";
pub const ENV_LOCAL_STORAGE_PATH: &str = "localStoragePath";
pub const ENV_INPUT_BUCKET_NAME: &str = "inputBucketName";
pub const ENV_OUTPUT_BUCKET_NAME: &str = "outputBucketName";
pub const ENV_CONCURRENCY_UPLOAD: &str = "CONCURRENCY_UPLOAD";
pub const ENV_CONCURRENCY_WORKERS: &str = "CONCURRENCY_WORKERS";

pub const BUCKET_NAME: &str = "micro-admin-typescript-josemoura212";

const DEFAULT_LOCAL_STORAGE_PATH: &str = "/tmp";
const DEFAULT_CONCURRENCY: usize = 4;
const DEFAULT_CONCURRENCY_WORKERS: usize = 2;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub local_storage_path: String,
    pub input_bucket_name: String,
    pub output_bucket_name: String,
    pub concurrency: usize,
    pub concurrency_workers: usize,
    pub auto_migrate: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("missing required env var: {0}")]
    Missing(String),
    #[error("invalid value for {key}: {reason}")]
    Invalid { key: String, reason: String },
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let database_url = required_env(ENV_DATABASE_URL)?;
        let local_storage_path = env::var(ENV_LOCAL_STORAGE_PATH)
            .unwrap_or_else(|_| DEFAULT_LOCAL_STORAGE_PATH.to_string());
        let input_bucket_name = required_env(ENV_INPUT_BUCKET_NAME)?;
        let output_bucket_name = required_env(ENV_OUTPUT_BUCKET_NAME)?;
        let concurrency = env::var(ENV_CONCURRENCY_UPLOAD)
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(DEFAULT_CONCURRENCY);
        let concurrency_workers = env::var(ENV_CONCURRENCY_WORKERS)
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(DEFAULT_CONCURRENCY_WORKERS);
        let auto_migrate = env::var(ENV_AUTO_MIGRATE_DB)
            .ok()
            .and_then(|v| v.parse::<bool>().ok())
            .unwrap_or(false);

        Ok(Config {
            database_url,
            local_storage_path,
            input_bucket_name,
            output_bucket_name,
            concurrency,
            concurrency_workers,
            auto_migrate,
        })
    }
}

fn required_env(key: &str) -> Result<String, ConfigError> {
    env::var(key).map_err(|_| ConfigError::Missing(key.to_string()))
}

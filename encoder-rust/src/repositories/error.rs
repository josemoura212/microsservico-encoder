#[derive(Debug, thiserror::Error)]
pub enum VideoRepositoryError {
    #[error("video not found")]
    NotFound,
    #[error("database error: {0}")]
    Database(String),
}

#[derive(Debug, thiserror::Error)]
pub enum JobRepositoryError {
    #[error("job not found")]
    NotFound,
    #[error("database error: {0}")]
    Database(String),
}

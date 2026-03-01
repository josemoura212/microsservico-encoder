#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("validation failed: {0}")]
    Validation(String),
}

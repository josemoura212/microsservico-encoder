#[derive(Debug)]
pub struct VideoRepositoryError(pub String);

impl std::fmt::Display for VideoRepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "VideoRepository error: {}", self.0)
    }
}

impl std::error::Error for VideoRepositoryError {}

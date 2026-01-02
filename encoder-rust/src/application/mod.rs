pub mod repositories {
    mod repository_error;
    mod repository_trait;
    mod video_repository;

    pub use repository_error::VideoRepositoryError;
    pub use repository_trait::Repository;
    pub use video_repository::VideoRepository;
}

pub use repositories::{Repository, VideoRepository, VideoRepositoryError};

pub mod repositories {
    mod job_repository;
    mod repository_error;
    mod repository_trait;
    mod video_repository;

    pub use job_repository::JobRepository;
    pub use repository_error::{JobRepositoryError, VideoRepositoryError};
    pub use repository_trait::Repository;
    pub use video_repository::VideoRepository;
}

pub use repositories::{
    JobRepository, JobRepositoryError, Repository, VideoRepository, VideoRepositoryError,
};

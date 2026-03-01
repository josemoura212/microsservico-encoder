use std::sync::Arc;
use uuid::Uuid;

use crate::{
    db::Database,
    domain::{Job, JobStatus, Video},
    repositories::{JobRepositoryError, Repository},
};

type JobRow = (
    Uuid,
    String,
    String,
    Uuid,
    Option<String>,
    chrono::DateTime<chrono::Utc>,
    chrono::DateTime<chrono::Utc>,
);
type VideoRow = (Uuid, String, String, chrono::DateTime<chrono::Utc>);

const INSERT_JOB_QUERY: &str = "INSERT INTO jobs (id, output_bucket_path, status, video_id, error, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)";

const FIND_JOB_QUERY: &str = "SELECT id, output_bucket_path, status, video_id, error, created_at, updated_at FROM jobs WHERE id = $1";

const FIND_VIDEO_QUERY: &str =
    "SELECT id, resource_id, file_path, created_at FROM videos WHERE id = $1";

const UPDATE_JOB_QUERY: &str = "UPDATE jobs SET output_bucket_path = $1, status = $2, error = $3, updated_at = $4 WHERE id = $5";

pub struct JobRepository<DB>
where
    DB: sqlx::Database,
{
    pub db: Database<DB>,
}

impl<DB> JobRepository<DB>
where
    DB: sqlx::Database,
{
    pub fn new(db: Database<DB>) -> Self {
        Self { db }
    }

    fn map_job_from_row(row: JobRow, video: Arc<Video>) -> Job {
        Job {
            id: row.0,
            output_bucket_path: row.1,
            status: row.2.parse::<JobStatus>().unwrap_or(JobStatus::Pending),
            video,
            video_id: row.3,
            error: row.4,
            created_at: row.5,
            updated_at: row.6,
        }
    }
}

crate::db::impl_with_db! {
    impl<DB> Repository<Job> for JobRepository<DB> {
    type Error = JobRepositoryError;

    async fn insert(&self, item: &Job) -> Result<Job, Self::Error> {
        sqlx::query(INSERT_JOB_QUERY)
            .bind(item.id)
            .bind(&item.output_bucket_path)
            .bind(item.status.to_string())
            .bind(item.video_id)
            .bind(&item.error)
            .bind(item.created_at)
            .bind(item.updated_at)
            .execute(&self.db.conn)
            .await
            .map_err(|e| JobRepositoryError::Database(e.to_string()))?;

        Ok(item.clone())
    }

    async fn find(&self, id: &Uuid) -> Result<Job, Self::Error> {
        let job_row = sqlx::query_as::<_, JobRow>(FIND_JOB_QUERY)
            .bind(id)
            .fetch_one(&self.db.conn)
            .await
            .map_err(|e| JobRepositoryError::Database(e.to_string()))?;

        let video_id = job_row.3;
        let video_row = sqlx::query_as::<_, VideoRow>(FIND_VIDEO_QUERY)
            .bind(video_id)
            .fetch_one(&self.db.conn)
            .await
            .map_err(|e| JobRepositoryError::Database(e.to_string()))?;

        let video = Arc::new(Video {
            id: video_row.0,
            resource_id: video_row.1,
            file_path: video_row.2,
            created_at: video_row.3,
            jobs: Vec::new(),
        });

        Ok(Self::map_job_from_row(job_row, video))
    }

    async fn update(&self, item: &Job) -> Result<Job, Self::Error> {
        sqlx::query(UPDATE_JOB_QUERY)
            .bind(&item.output_bucket_path)
            .bind(item.status.to_string())
            .bind(&item.error)
            .bind(item.updated_at)
            .bind(item.id)
            .execute(&self.db.conn)
            .await
            .map_err(|e| JobRepositoryError::Database(e.to_string()))?;

        Ok(item.clone())
    }
    }
}

#[cfg(test)]
mod tests {
    use sqlx::Sqlite;
    use std::{env, sync::Arc};

    use crate::{
        config::ENV_DATABASE_URL_TEST,
        db::Database,
        domain::{Job, JobStatus, Video},
        repositories::Repository,
    };

    async fn setup_test_db() -> Database<Sqlite> {
        let database_url =
            env::var(ENV_DATABASE_URL_TEST).unwrap_or_else(|_| "sqlite::memory:".to_string());

        Database::<Sqlite>::new(database_url, Some(true))
            .await
            .expect("Failed to create test database connection")
    }

    #[tokio::test]
    async fn test_job_repository_insert_and_find() {
        let db = setup_test_db().await;

        let video_repo = crate::repositories::VideoRepository {
            db: Database {
                conn: db.conn.clone(),
            },
        };
        let new_video = Video::new(
            "resource_456".to_string(),
            "/path/to/video2.mp4".to_string(),
        )
        .unwrap();
        video_repo
            .insert(&new_video)
            .await
            .expect("Failed to insert video");

        let job_repo = super::JobRepository { db };
        let video_arc = Arc::new(new_video);
        let new_job = Job::new("/output/path".to_string(), video_arc);

        let inserted_job = job_repo
            .insert(&new_job)
            .await
            .expect("Failed to insert job");

        assert_eq!(inserted_job.id, new_job.id);

        let found_job = job_repo
            .find(&new_job.id)
            .await
            .expect("Failed to find job");

        assert_eq!(found_job.id, new_job.id);
        assert_eq!(found_job.output_bucket_path, new_job.output_bucket_path);
        assert_eq!(found_job.status, new_job.status);
        assert_eq!(found_job.video.id, new_job.video.id);
    }

    #[tokio::test]
    async fn test_job_repository_update() {
        let db = setup_test_db().await;

        let video_repo = crate::repositories::VideoRepository {
            db: Database {
                conn: db.conn.clone(),
            },
        };
        let new_video = Video::new(
            "resource_789".to_string(),
            "/path/to/video3.mp4".to_string(),
        )
        .unwrap();
        video_repo
            .insert(&new_video)
            .await
            .expect("Failed to insert video");

        let job_repo = super::JobRepository { db };
        let video_arc = Arc::new(new_video);
        let new_job = Job::new("/output/path2".to_string(), video_arc);

        job_repo
            .insert(&new_job)
            .await
            .expect("Failed to insert job");

        let completed_job = new_job.complete();

        let updated_job = job_repo
            .update(&completed_job)
            .await
            .expect("Failed to update job");

        assert_eq!(updated_job.status, JobStatus::Completed);

        let found_job = job_repo
            .find(&new_job.id)
            .await
            .expect("Failed to find updated job");

        assert_eq!(found_job.status, JobStatus::Completed);
    }
}

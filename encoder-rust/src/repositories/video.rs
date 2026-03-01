use std::sync::Arc;
use uuid::Uuid;

use crate::{
    db::Database,
    domain::{Job, JobStatus, Video},
    repositories::{Repository, VideoRepositoryError},
};

type VideoWithJobsRow = (
    Uuid,
    String,
    String,
    chrono::DateTime<chrono::Utc>,
    Option<Uuid>,
    Option<String>,
    Option<String>,
    Option<Uuid>,
    Option<String>,
    Option<chrono::DateTime<chrono::Utc>>,
    Option<chrono::DateTime<chrono::Utc>>,
);

const INSERT_VIDEO_QUERY: &str =
    "INSERT INTO videos (id, resource_id, file_path, created_at) VALUES ($1, $2, $3, $4)";

const FIND_VIDEO_WITH_JOBS_QUERY: &str = r#"
    SELECT
        v.id, v.resource_id, v.file_path, v.created_at,
        j.id, j.output_bucket_path, j.status, j.video_id, j.error, j.created_at, j.updated_at
    FROM videos v
    LEFT JOIN jobs j ON v.id = j.video_id
    WHERE v.id = $1
"#;

pub struct VideoRepository<DB>
where
    DB: sqlx::Database,
{
    pub db: Database<DB>,
}

impl<DB> VideoRepository<DB>
where
    DB: sqlx::Database,
{
    pub fn new(db: Database<DB>) -> Self {
        Self { db }
    }

    fn map_job_from_row(row: VideoWithJobsRow, video: &Arc<Video>) -> Option<Arc<Job>> {
        let (_, _, _, _, job_id, output_path, status, video_id, error, created_at, updated_at) =
            row;

        let job_id = job_id?;
        let output_path = output_path?;
        let status_str = status?;
        let video_id = video_id?;
        let created_at = created_at?;
        let updated_at = updated_at?;

        Some(Arc::new(Job {
            id: job_id,
            output_bucket_path: output_path,
            status: status_str
                .parse::<JobStatus>()
                .unwrap_or(JobStatus::Pending),
            video: Arc::clone(video),
            video_id,
            error,
            created_at,
            updated_at,
        }))
    }
}

crate::db::impl_with_db! {
    impl<DB> Repository<Video> for VideoRepository<DB> {
        type Error = VideoRepositoryError;

        async fn insert(&self, item: &Video) -> Result<Video, Self::Error> {
            sqlx::query(INSERT_VIDEO_QUERY)
                .bind(item.id)
                .bind(&item.resource_id)
                .bind(&item.file_path)
                .bind(item.created_at)
                .execute(&self.db.conn)
                .await
                .map_err(|e| VideoRepositoryError::Database(e.to_string()))?;

            Ok(item.clone())
        }

        async fn find(&self, id: &Uuid) -> Result<Video, Self::Error> {
            let rows = sqlx::query_as::<_, VideoWithJobsRow>(FIND_VIDEO_WITH_JOBS_QUERY)
                .bind(id)
                .fetch_all(&self.db.conn)
                .await
                .map_err(|e| VideoRepositoryError::Database(e.to_string()))?;

            if rows.is_empty() {
                return Err(VideoRepositoryError::NotFound);
            }

            let video_data = &rows[0];
            let video_arc = Arc::new(Video {
                id: video_data.0,
                resource_id: video_data.1.clone(),
                file_path: video_data.2.clone(),
                created_at: video_data.3,
                jobs: Vec::new(),
            });

            let jobs = rows
                .into_iter()
                .filter_map(|row| Self::map_job_from_row(row, &video_arc))
                .collect();

            Ok(Video {
                id: video_arc.id,
                resource_id: video_arc.resource_id.clone(),
                file_path: video_arc.file_path.clone(),
                created_at: video_arc.created_at,
                jobs,
            })
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
        domain::{Job, Video},
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
    async fn test_video_repository_insert_and_find() {
        let db = setup_test_db().await;
        let video_repo = super::VideoRepository { db };

        let new_video =
            Video::new("resource_123".to_string(), "/path/to/video.mp4".to_string()).unwrap();

        let inserted_video = video_repo
            .insert(&new_video)
            .await
            .expect("Failed to insert video");

        assert_eq!(inserted_video.id, new_video.id);

        let found_video = video_repo
            .find(&new_video.id)
            .await
            .expect("Failed to find video");

        assert_eq!(found_video.id, new_video.id);
        assert_eq!(found_video.resource_id, new_video.resource_id);
        assert_eq!(found_video.file_path, new_video.file_path);
        assert_eq!(found_video.jobs.len(), 0);
    }

    #[tokio::test]
    async fn test_video_repository_find_with_jobs() {
        let db = setup_test_db().await;

        let video_repo = super::VideoRepository {
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

        let job_repo = crate::repositories::JobRepository { db };
        let video_arc = Arc::new(new_video.clone());

        let job1 = Job::new("/output/path1".to_string(), Arc::clone(&video_arc));
        let job2 = Job::new("/output/path2".to_string(), Arc::clone(&video_arc));

        job_repo.insert(&job1).await.expect("Failed to insert job1");
        job_repo.insert(&job2).await.expect("Failed to insert job2");

        let found_video = video_repo
            .find(&new_video.id)
            .await
            .expect("Failed to find video with jobs");

        assert_eq!(found_video.id, new_video.id);
        assert_eq!(found_video.jobs.len(), 2);

        let job_ids: Vec<_> = found_video.jobs.iter().map(|j| j.id).collect();
        assert!(job_ids.contains(&job1.id));
        assert!(job_ids.contains(&job2.id));

        for job in &found_video.jobs {
            assert_eq!(job.video.id, new_video.id);
            assert_eq!(job.video_id, new_video.id);
        }
    }
}

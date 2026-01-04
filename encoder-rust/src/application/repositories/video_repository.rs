use std::sync::Arc;
use uuid::Uuid;

use crate::{
    application::{Repository, VideoRepositoryError},
    domain::{Job, Video},
    framework::Database,
};

// Type aliases para melhor legibilidade
type VideoRow = (Uuid, String, String, chrono::DateTime<chrono::Utc>);
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

// Queries SQL como constantes
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

    /// Mapeia uma linha do LEFT JOIN para um Job (se existir)
    fn map_job_from_row(row: VideoWithJobsRow, video: &Arc<Video>) -> Option<Arc<Job>> {
        let (_, _, _, _, job_id, output_path, status, video_id, error, created_at, updated_at) =
            row;

        // Se job_id é None, significa que não há job nesta linha (LEFT JOIN sem match)
        let job_id = job_id?;
        let output_path = output_path?;
        let status = status?;
        let video_id = video_id?;
        let created_at = created_at?;
        let updated_at = updated_at?;

        Some(Arc::new(Job {
            id: job_id,
            output_bucket_path: output_path,
            status,
            video: Arc::clone(video),
            video_id,
            error,
            created_at,
            updated_at,
        }))
    }
}

// Trait bounds organizados por categoria para melhor legibilidade
impl<DB> Repository<Video> for VideoRepository<DB>
where
    DB: sqlx::Database,
    // Suporte aos tipos usados nas queries
    for<'q> Uuid: sqlx::Encode<'q, DB> + sqlx::Type<DB> + sqlx::Decode<'q, DB>,
    for<'q> String: sqlx::Encode<'q, DB> + sqlx::Type<DB> + sqlx::Decode<'q, DB>,
    for<'q> chrono::DateTime<chrono::Utc>:
        sqlx::Encode<'q, DB> + sqlx::Type<DB> + sqlx::Decode<'q, DB>,
    // Suporte a referências nas queries
    for<'q> &'q Uuid: sqlx::Encode<'q, DB>,
    for<'q> &'q String: sqlx::Encode<'q, DB>,
    for<'q> &'q chrono::DateTime<chrono::Utc>: sqlx::Encode<'q, DB>,
    // Suporte aos argumentos e executor do sqlx
    for<'q> <DB as sqlx::Database>::Arguments<'q>: sqlx::IntoArguments<'q, DB>,
    for<'c> &'c mut <DB as sqlx::Database>::Connection: sqlx::Executor<'c, Database = DB>,
    // Suporte a indexação de colunas
    usize: sqlx::ColumnIndex<DB::Row>,
{
    type Error = VideoRepositoryError;

    /// Insere um novo vídeo no banco de dados
    async fn insert(&self, item: &Video) -> Result<Video, Self::Error> {
        sqlx::query(INSERT_VIDEO_QUERY)
            .bind(item.id)
            .bind(&item.resource_id)
            .bind(&item.file_path)
            .bind(item.created_at)
            .execute(&self.db.conn)
            .await
            .map_err(|e| VideoRepositoryError(e.to_string()))?;

        Ok(item.clone())
    }

    /// Busca um vídeo por ID, incluindo todos os jobs associados via LEFT JOIN
    async fn find(&self, id: &Uuid) -> Result<Video, Self::Error> {
        // Busca vídeo com jobs em uma única query usando LEFT JOIN
        let rows = sqlx::query_as::<_, VideoWithJobsRow>(FIND_VIDEO_WITH_JOBS_QUERY)
            .bind(id)
            .fetch_all(&self.db.conn)
            .await
            .map_err(|e| VideoRepositoryError(e.to_string()))?;

        if rows.is_empty() {
            return Err(VideoRepositoryError("Video not found".to_string()));
        }

        // Extrai dados do vídeo da primeira linha
        let video_data = &rows[0];
        let video_arc = Arc::new(Video {
            id: video_data.0,
            resource_id: video_data.1.clone(),
            file_path: video_data.2.clone(),
            created_at: video_data.3,
            jobs: Vec::new(),
        });

        // Mapeia jobs encontrados (filtra linhas sem job associado)
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

#[cfg(test)]
mod tests {
    use sqlx::Sqlite;
    use std::{env, sync::Arc};

    use crate::{
        application::Repository,
        domain::{Job, Video},
        framework::Database,
    };

    async fn setup_test_db() -> Database<Sqlite> {
        let database_url =
            env::var("DATABASE_URL_TEST").unwrap_or_else(|_| "sqlite::memory:".to_string());

        Database::<Sqlite>::new(database_url, Some(true))
            .await
            .expect("Failed to create test database connection")
    }

    #[tokio::test]
    async fn test_video_repository_insert_and_find() {
        let db = setup_test_db().await;
        let video_repo = super::VideoRepository { db };

        let new_video = Video::new("resource_123".to_string(), "/path/to/video.mp4".to_string());

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
        assert_eq!(found_video.jobs.len(), 0); // Sem jobs associados
    }

    #[tokio::test]
    async fn test_video_repository_find_with_jobs() {
        let db = setup_test_db().await;

        // Criar e inserir vídeo
        let video_repo = super::VideoRepository {
            db: Database {
                conn: db.conn.clone(),
            },
        };
        let new_video = Video::new(
            "resource_456".to_string(),
            "/path/to/video2.mp4".to_string(),
        );
        video_repo
            .insert(&new_video)
            .await
            .expect("Failed to insert video");

        // Criar e inserir jobs
        let job_repo = super::super::JobRepository { db };
        let video_arc = Arc::new(new_video.clone());

        let job1 = Job::new(
            "/output/path1".to_string(),
            "pending".to_string(),
            Arc::clone(&video_arc),
        );

        let job2 = Job::new(
            "/output/path2".to_string(),
            "completed".to_string(),
            Arc::clone(&video_arc),
        );

        job_repo.insert(&job1).await.expect("Failed to insert job1");

        job_repo.insert(&job2).await.expect("Failed to insert job2");

        // Buscar vídeo com jobs
        let found_video = video_repo
            .find(&new_video.id)
            .await
            .expect("Failed to find video with jobs");

        assert_eq!(found_video.id, new_video.id);
        assert_eq!(found_video.jobs.len(), 2); // Deve ter 2 jobs

        // Verificar que os jobs foram carregados corretamente
        let job_ids: Vec<_> = found_video.jobs.iter().map(|j| j.id).collect();
        assert!(job_ids.contains(&job1.id));
        assert!(job_ids.contains(&job2.id));

        // Verificar que os jobs têm referência ao vídeo correto
        for job in &found_video.jobs {
            assert_eq!(job.video.id, new_video.id);
            assert_eq!(job.video_id, new_video.id);
        }
    }
}

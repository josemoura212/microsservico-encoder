use uuid::Uuid;

use crate::{
    application::{Repository, VideoRepositoryError},
    domain::Video,
    framework::Database,
};

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

    async fn insert(&self, item: &Video) -> Result<Video, Self::Error> {
        sqlx::query(
            "INSERT INTO videos (id, resource_id, file_path, created_at) VALUES ($1, $2, $3, $4)",
        )
        .bind(&item.id)
        .bind(&item.resource_id)
        .bind(&item.file_path)
        .bind(&item.created_at)
        .execute(&self.db.conn)
        .await
        .map_err(|e| VideoRepositoryError(e.to_string()))?;

        Ok(item.clone())
    }

    async fn find(&self, id: &Uuid) -> Result<Video, Self::Error> {
        let row = sqlx::query_as::<_, (Uuid, String, String, chrono::DateTime<chrono::Utc>)>(
            "SELECT id, resource_id, file_path, created_at FROM videos WHERE id = $1",
        )
        .bind(id)
        .fetch_one(&self.db.conn)
        .await
        .map_err(|e| VideoRepositoryError(e.to_string()))?;

        Ok(Video {
            id: row.0,
            resource_id: row.1,
            file_path: row.2,
            created_at: row.3,
            jobs: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use sqlx::Sqlite;
    use std::env;

    use crate::{application::Repository, domain::Video, framework::Database};

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
    }
}

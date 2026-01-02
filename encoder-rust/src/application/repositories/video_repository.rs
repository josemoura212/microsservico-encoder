use sqlx::Postgres;
use uuid::Uuid;

use crate::{
    application::repositories::{Repository, VideoRepositoryError},
    domain::Video,
    framework::database::db::Database,
};

pub struct VideoRepository {
    pub db: Database<Postgres>,
}

impl VideoRepository {
    pub fn new(db: Database<Postgres>) -> Self {
        Self { db }
    }
}

impl Repository<Video> for VideoRepository {
    type Error = VideoRepositoryError;

    async fn insert(&self, item: &Video) -> Result<Video, Self::Error> {
        sqlx::query!(
            "INSERT INTO videos (id, resource_id, file_path, created_at) VALUES ($1, $2, $3, $4)",
            item.id,
            item.resource_id,
            item.file_path,
            item.created_at
        )
        .execute(&self.db.conn)
        .await
        .map_err(|e| VideoRepositoryError(e.to_string()))?;

        Ok(item.clone())
    }

    async fn find(&self, id: &Uuid) -> Result<Video, Self::Error> {
        let row = sqlx::query!(
            "SELECT id, resource_id, file_path, created_at FROM videos WHERE id = $1",
            id
        )
        .fetch_one(&self.db.conn)
        .await
        .map_err(|e| VideoRepositoryError(e.to_string()))?;

        Ok(Video {
            id: row.id,
            resource_id: row.resource_id,
            file_path: row.file_path,
            created_at: row.created_at,
            jobs: Vec::new(),
        })
    }
}

use std::env;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::{application::VideoRepository, domain::Video};
use google_cloud_storage::client::{Client, ClientConfig};
use google_cloud_storage::http::objects::download::Range;
use google_cloud_storage::http::objects::get::GetObjectRequest;

pub struct VideoService<DB>
where
    DB: sqlx::Database,
{
    pub video_repository: VideoRepository<DB>,
    pub video: Video,
}

impl<DB> VideoService<DB>
where
    DB: sqlx::Database,
{
    pub fn new(video_repository: VideoRepository<DB>, video: Video) -> Self {
        VideoService {
            video_repository,
            video,
        }
    }

    pub async fn download(&self, bucket_name: &str) -> anyhow::Result<()> {
        let config = ClientConfig::default().with_auth().await?;
        let client = Client::new(config);

        let data = client
            .download_object(
                &GetObjectRequest {
                    bucket: bucket_name.to_string(),
                    object: self.video.file_path.clone(),
                    ..Default::default()
                },
                &Range::default(),
            )
            .await?;

        let local_storage_path =
            env::var("localStoragePath").unwrap_or_else(|_| "/tmp".to_string());

        let file_path = PathBuf::from(local_storage_path).join(format!("{}.mp4", self.video.id));

        let mut file = File::create(&file_path).await?;
        file.write_all(&data).await?;
        file.flush().await?;

        tracing::info!("Video {} has been stored at {:?}", self.video.id, file_path);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{application::VideoRepository, domain::Video, framework::Database};
    use sqlx::Sqlite;
    use std::env;

    async fn setup_test_db() -> Database<Sqlite> {
        let database_url =
            env::var("DATABASE_URL_TEST").unwrap_or_else(|_| "sqlite::memory:".to_string());

        Database::<Sqlite>::new(database_url, Some(true))
            .await
            .expect("Failed to create test database connection")
    }

    #[tokio::test]
    #[ignore]
    async fn test_download_video_from_gcs() {
        // Setup
        let db = setup_test_db().await;
        let video_repository = VideoRepository::new(db);

        // Criar um vídeo com o arquivo do bucket (caminho completo incluindo pasta videos)
        let video = Video::new(
            "3fa3291e-5daf-4386-9a67-69d19e1690c5".to_string(),
            "videos/3fa3291e-5daf-4386-9a67-69d19e1690c5/videos/3fa3291e-5daf-4386-9a67-69d19e1690c5-b8c187dd77c950e9b117bcc19e35a9005e45001593f7f4260040cee47d77faa0.mp4".to_string(),
        );

        let video_service = VideoService::new(video_repository, video.clone());

        // Nome do bucket
        let bucket_name = "micro-admin-typescript-josemoura212";

        // Criar pasta tmp na raiz do projeto
        let tmp_path = "./tmp";
        tokio::fs::create_dir_all(tmp_path)
            .await
            .expect("Failed to create tmp directory");

        // Configurar o caminho de armazenamento local para testes
        unsafe {
            env::set_var("localStoragePath", tmp_path);
        }

        // Executar o download
        let result = video_service.download(bucket_name).await;

        // Verificações
        assert!(result.is_ok(), "Download should succeed: {:?}", result);

        // Verificar se o arquivo foi criado
        let expected_file_path = format!("./tmp/{}.mp4", video.id);
        let file_exists = tokio::fs::metadata(&expected_file_path).await.is_ok();
        assert!(
            file_exists,
            "Downloaded file should exist at {}",
            expected_file_path
        );

        // Verificar se o arquivo tem conteúdo
        let file_metadata = tokio::fs::metadata(&expected_file_path)
            .await
            .expect("Failed to get file metadata");
        assert!(
            file_metadata.len() > 0,
            "Downloaded file should have content"
        );
    }

    #[tokio::test]
    async fn test_video_service_creation() {
        let db = setup_test_db().await;
        let video_repository = VideoRepository::new(db);
        let video = Video::new("test_resource".to_string(), "test_file.mp4".to_string());

        let video_service = VideoService::new(video_repository, video.clone());

        assert_eq!(video_service.video.resource_id, "test_resource");
        assert_eq!(video_service.video.file_path, "test_file.mp4");
    }

    #[tokio::test]
    #[ignore]
    async fn test_download_with_invalid_bucket() {
        let db = setup_test_db().await;
        let video_repository = VideoRepository::new(db);
        let video = Video::new(
            "test_resource".to_string(),
            "nonexistent_file.mp4".to_string(),
        );

        let video_service = VideoService::new(video_repository, video);

        let result = video_service.download("invalid-bucket-name").await;

        assert!(result.is_err(), "Download should fail with invalid bucket");
    }
}

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

    async fn fragment(&self) -> anyhow::Result<()> {
        let local_storage_path =
            env::var("localStoragePath").unwrap_or_else(|_| "/tmp".to_string());

        tokio::fs::create_dir_all(format!("{}/{}", local_storage_path, self.video.id))
            .await
            .expect("Failed to create tmp directory");

        let source = format!("{}/{}.mp4", local_storage_path, self.video.id);
        let destination = format!("{}/{}.frag", local_storage_path, self.video.id);

        let output = tokio::process::Command::new("mp4fragment")
            .args(&[source, destination])
            .output()
            .await?;

        Self::print_output(&output);

        Ok(())
    }

    async fn encode(&self) -> anyhow::Result<()> {
        let mut cmd_args = vec![];

        let local_storage_path =
            env::var("localStoragePath").unwrap_or_else(|_| "/tmp".to_string());

        cmd_args.push(format!("{}/{}.frag", local_storage_path, self.video.id));
        cmd_args.push("--use-segment-timeline".to_string());
        cmd_args.push("-o".to_string());
        cmd_args.push(format!("{}/{}", local_storage_path, self.video.id));
        cmd_args.push("-f".to_string());
        cmd_args.push("--exec-dir".to_string());
        cmd_args.push("/opt/bento4/bin/".to_string());

        let output = tokio::process::Command::new("mp4dash")
            .args(&cmd_args)
            .output()
            .await?;

        Self::print_output(&output);

        Ok(())
    }

    async fn finish(&self) -> anyhow::Result<()> {
        let local_storage_path =
            env::var("localStoragePath").unwrap_or_else(|_| "/tmp".to_string());

        tokio::fs::remove_file(format!("{}/{}.mp4", local_storage_path, self.video.id))
            .await
            .expect("Failed to remove mp4 file");

        tokio::fs::remove_file(format!("{}/{}.frag", local_storage_path, self.video.id))
            .await
            .expect("Failed to remove fragment file");

        tokio::fs::remove_dir_all(format!("{}/{}", local_storage_path, self.video.id))
            .await
            .expect("Failed to remove video directory");

        tracing::info!("Cleaned up files for video {}", self.video.id);

        Ok(())
    }

    fn print_output(output: &std::process::Output) {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.is_empty() {
            tracing::info!("=====> Output: {}", stdout);
        }
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
    async fn test_video_service_download() {
        // Setup
        let db = setup_test_db().await;
        let video_repository = VideoRepository::new(db);

        let video = Video::new(
            "3fa3291e-5daf-4386-9a67-69d19e1690c5".to_string(),
            "videos/3fa3291e-5daf-4386-9a67-69d19e1690c5/videos/3fa3291e-5daf-4386-9a67-69d19e1690c5-b8c187dd77c950e9b117bcc19e35a9005e45001593f7f4260040cee47d77faa0.mp4".to_string(),
        );

        let video_service = VideoService::new(video_repository, video.clone());

        let tmp_path = "./tmp";
        tokio::fs::create_dir_all(tmp_path)
            .await
            .expect("Failed to create tmp directory");

        unsafe {
            env::set_var("localStoragePath", tmp_path);
        }

        let result = video_service
            .download("micro-admin-typescript-josemoura212")
            .await;

        assert!(result.is_ok());

        let result = video_service.fragment().await;
        assert!(result.is_ok());

        let result = video_service.encode().await;
        assert!(result.is_ok());

        let result = video_service.finish().await;
        assert!(result.is_ok());
    }
}

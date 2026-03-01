use std::path::PathBuf;
use std::time::Duration;

use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::config::Config;
use crate::domain::Video;
use crate::repositories::VideoRepository;
use google_cloud_storage::client::{Client, ClientConfig};
use google_cloud_storage::http::objects::download::Range;
use google_cloud_storage::http::objects::get::GetObjectRequest;

const CMD_TIMEOUT: Duration = Duration::from_secs(600);

#[derive(Debug, thiserror::Error)]
pub enum VideoServiceError {
    #[error("fragment failed (exit {code}): {stderr}")]
    FragmentFailed { code: i32, stderr: String },
    #[error("encode failed (exit {code}): {stderr}")]
    EncodeFailed { code: i32, stderr: String },
    #[error("process timeout")]
    Timeout,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub struct VideoService<DB>
where
    DB: sqlx::Database,
{
    pub video_repository: VideoRepository<DB>,
    pub video: Video,
    pub config: Config,
}

impl<DB> VideoService<DB>
where
    DB: sqlx::Database,
{
    pub fn new(video_repository: VideoRepository<DB>, video: Video, config: Config) -> Self {
        VideoService {
            video_repository,
            video,
            config,
        }
    }

    pub async fn download(&self, bucket_name: &str) -> Result<(), VideoServiceError> {
        let gcs_config = ClientConfig::default()
            .with_auth()
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        let client = Client::new(gcs_config);

        let data = client
            .download_object(
                &GetObjectRequest {
                    bucket: bucket_name.to_string(),
                    object: self.video.file_path.clone(),
                    ..Default::default()
                },
                &Range::default(),
            )
            .await
            .map_err(|e| anyhow::anyhow!(e))?;

        let file_path =
            PathBuf::from(&self.config.local_storage_path).join(format!("{}.mp4", self.video.id));

        let mut file = File::create(&file_path).await?;
        file.write_all(&data).await?;
        file.flush().await?;

        tracing::info!("Video {} has been stored at {:?}", self.video.id, file_path);

        Ok(())
    }

    pub async fn fragment(&self) -> Result<(), VideoServiceError> {
        let local_path = &self.config.local_storage_path;

        tokio::fs::create_dir_all(format!("{}/{}", local_path, self.video.id)).await?;

        let source = format!("{}/{}.mp4", local_path, self.video.id);
        let destination = format!("{}/{}.frag", local_path, self.video.id);

        let output = tokio::time::timeout(
            CMD_TIMEOUT,
            tokio::process::Command::new("mp4fragment")
                .args([&source, &destination])
                .output(),
        )
        .await
        .map_err(|_| VideoServiceError::Timeout)?
        .map_err(VideoServiceError::Io)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(VideoServiceError::FragmentFailed {
                code: output.status.code().unwrap_or(-1),
                stderr,
            });
        }

        Self::print_output(&output);

        Ok(())
    }

    pub async fn encode(&self) -> Result<(), VideoServiceError> {
        let local_path = &self.config.local_storage_path;

        let cmd_args = vec![
            format!("{}/{}.frag", local_path, self.video.id),
            "--use-segment-timeline".to_string(),
            "-o".to_string(),
            format!("{}/{}", local_path, self.video.id),
            "-f".to_string(),
            "--exec-dir".to_string(),
            "/opt/bento4/bin/".to_string(),
        ];

        let output = tokio::time::timeout(
            CMD_TIMEOUT,
            tokio::process::Command::new("mp4dash")
                .args(&cmd_args)
                .output(),
        )
        .await
        .map_err(|_| VideoServiceError::Timeout)?
        .map_err(VideoServiceError::Io)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(VideoServiceError::EncodeFailed {
                code: output.status.code().unwrap_or(-1),
                stderr,
            });
        }

        Self::print_output(&output);

        Ok(())
    }

    pub async fn finish(&self) -> Result<(), VideoServiceError> {
        let local_path = &self.config.local_storage_path;

        tokio::fs::remove_file(format!("{}/{}.mp4", local_path, self.video.id)).await?;
        tokio::fs::remove_file(format!("{}/{}.frag", local_path, self.video.id)).await?;
        tokio::fs::remove_dir_all(format!("{}/{}", local_path, self.video.id)).await?;

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
    use crate::{
        config::{BUCKET_NAME, ENV_DATABASE_URL_TEST},
        db::Database,
        domain::Video,
    };
    use sqlx::Sqlite;
    use std::env;

    fn test_config(local_storage_path: &str) -> Config {
        Config {
            database_url: String::new(),
            local_storage_path: local_storage_path.to_string(),
            input_bucket_name: "test-bucket".to_string(),
            output_bucket_name: "test-output-bucket".to_string(),
            concurrency: 4,
            concurrency_workers: 2,
            auto_migrate: false,
        }
    }

    async fn setup_test_db() -> Database<Sqlite> {
        let database_url =
            env::var(ENV_DATABASE_URL_TEST).unwrap_or_else(|_| "sqlite::memory:".to_string());

        Database::<Sqlite>::new(database_url, Some(true))
            .await
            .expect("Failed to create test database connection")
    }

    #[tokio::test]
    #[ignore]
    async fn test_video_service_download() {
        let db = setup_test_db().await;
        let video_repository = VideoRepository::new(db);

        let video = Video::new(
            "3fa3291e-5daf-4386-9a67-69d19e1690c5".to_string(),
            "videos/3fa3291e-5daf-4386-9a67-69d19e1690c5/videos/3fa3291e-5daf-4386-9a67-69d19e1690c5-b8c187dd77c950e9b117bcc19e35a9005e45001593f7f4260040cee47d77faa0.mp4".to_string(),
        ).unwrap();

        let tmp_path = "./tmp";
        tokio::fs::create_dir_all(tmp_path)
            .await
            .expect("Failed to create tmp directory");

        let config = test_config(tmp_path);
        let video_service = VideoService::new(video_repository, video, config);

        let result = video_service.download(BUCKET_NAME).await;
        assert!(result.is_ok());

        let result = video_service.fragment().await;
        assert!(result.is_ok());

        let result = video_service.encode().await;
        assert!(result.is_ok());

        let result = video_service.finish().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn test_video_service_upload() {
        let db = setup_test_db().await;
        let video_repository = VideoRepository::new(db);

        let video = Video::new(
            "3fa3291e-5daf-4386-9a67-69d19e1690c5".to_string(),
            "videos/3fa3291e-5daf-4386-9a67-69d19e1690c5/videos/3fa3291e-5daf-4386-9a67-69d19e1690c5-b8c187dd77c950e9b117bcc19e35a9005e45001593f7f4260040cee47d77faa0.mp4".to_string(),
        ).unwrap();

        let tmp_path = "./tmp";
        tokio::fs::create_dir_all(tmp_path)
            .await
            .expect("Failed to create tmp directory");

        let config = test_config(tmp_path);
        let video_service = VideoService::new(video_repository, video.clone(), config);

        let result = video_service.download(BUCKET_NAME).await;
        assert!(result.is_ok());

        let result = video_service.fragment().await;
        assert!(result.is_ok());

        let result = video_service.encode().await;
        assert!(result.is_ok());

        let mut video_upload = crate::services::VideoUpload::new(
            format!("{}/{}", tmp_path, video.id),
            BUCKET_NAME.to_string(),
        );

        let (done_tx, mut done_rx) = tokio::sync::mpsc::channel::<String>(1);

        tokio::spawn(async move {
            video_upload
                .process_upload(50, done_tx)
                .await
                .expect("process_upload failed");
        });

        let result = done_rx.recv().await.expect("channel closed");
        assert_eq!(result, "upload completed");
    }
}

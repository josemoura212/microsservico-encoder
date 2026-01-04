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

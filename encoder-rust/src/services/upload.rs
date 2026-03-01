use std::path::Path;
use std::sync::Arc;

use google_cloud_storage::client::{Client, ClientConfig};
use google_cloud_storage::http::objects::upload::{Media, UploadObjectRequest, UploadType};
use tokio::fs;
use tokio::sync::{Semaphore, mpsc};
use tokio_util::sync::CancellationToken;

pub struct VideoUpload {
    pub paths: Vec<String>,
    pub video_path: String,
    pub output_bucket: String,
    pub errors: Vec<String>,
}

impl VideoUpload {
    pub fn new(video_path: String, output_bucket: String) -> Self {
        VideoUpload {
            paths: Vec::new(),
            video_path,
            output_bucket,
            errors: Vec::new(),
        }
    }

    pub async fn load_paths(&mut self) -> anyhow::Result<()> {
        self.paths = Self::walk_dir(Path::new(&self.video_path)).await?;
        Ok(())
    }

    pub async fn process_upload(
        &mut self,
        concurrency: usize,
        done_tx: mpsc::Sender<String>,
    ) -> anyhow::Result<()> {
        self.load_paths().await?;

        let client = Arc::new(get_client_upload().await?);
        let semaphore = Arc::new(Semaphore::new(concurrency));
        let output_bucket = self.output_bucket.clone();
        let cancel_token = CancellationToken::new();

        let mut handles = Vec::new();

        for path in &self.paths {
            let permit = Arc::clone(&semaphore).acquire_owned().await?;
            let client = Arc::clone(&client);
            let path = path.clone();
            let bucket = output_bucket.clone();
            let video_path = self.video_path.clone();
            let done_tx = done_tx.clone();
            let token = cancel_token.clone();

            handles.push(tokio::spawn(async move {
                let _permit = permit;

                if token.is_cancelled() {
                    return;
                }

                let relative = path
                    .strip_prefix(&format!("{}/", video_path))
                    .unwrap_or(&path);

                let result = async {
                    let data = fs::read(&path).await?;
                    let upload_type = UploadType::Simple(Media::new(relative.to_string()));
                    client
                        .upload_object(
                            &UploadObjectRequest {
                                bucket,
                                ..Default::default()
                            },
                            data,
                            &upload_type,
                        )
                        .await?;
                    anyhow::Ok(())
                }
                .await;

                if let Err(e) = result {
                    tracing::error!("error during upload: {}. Error: {}", path, e);
                    token.cancel();
                    let _ = done_tx.send(e.to_string()).await;
                }
            }));
        }

        for handle in handles {
            let _ = handle.await;
        }

        if !cancel_token.is_cancelled() {
            let _ = done_tx.send("upload completed".to_string()).await;
        }

        Ok(())
    }

    async fn walk_dir(dir: &Path) -> anyhow::Result<Vec<String>> {
        let mut paths = Vec::new();
        let mut entries = fs::read_dir(dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                let mut sub = Box::pin(Self::walk_dir(&path)).await?;
                paths.append(&mut sub);
            } else {
                paths.push(path.to_string_lossy().to_string());
            }
        }

        Ok(paths)
    }
}

pub async fn get_client_upload() -> anyhow::Result<Client> {
    let config = ClientConfig::default().with_auth().await?;
    Ok(Client::new(config))
}

use crate::config::Config;
use crate::domain::Job;
use crate::repositories::{JobRepository, Repository};
use crate::services::{VideoService, VideoUpload};

#[derive(Debug, thiserror::Error)]
pub enum JobServiceError {
    #[error("job failed: {0}")]
    Failed(String),
}

pub struct JobService<DB>
where
    DB: sqlx::Database,
{
    pub job: Job,
    pub job_repository: JobRepository<DB>,
    pub video_service: VideoService<DB>,
    pub config: Config,
}

crate::db::impl_with_db! {
    impl<DB> JobService<DB> {
    pub async fn start(&mut self) -> Result<(), JobServiceError> {
        if let Err(e) = self.run_pipeline().await {
            return Err(self.fail_job(e.to_string()).await);
        }
        Ok(())
    }

    async fn run_pipeline(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.change_job_status("DOWNLOADING").await?;
        self.video_service
            .download(&self.config.input_bucket_name)
            .await?;

        self.change_job_status("FRAGMENTING").await?;
        self.video_service.fragment().await?;

        self.change_job_status("ENCODING").await?;
        self.video_service.encode().await?;

        self.perform_upload().await?;

        self.change_job_status("FINISHING").await?;
        self.video_service.finish().await?;

        self.change_job_status("COMPLETED").await?;

        Ok(())
    }

    async fn perform_upload(
        &mut self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.change_job_status("UPLOADING").await?;

        let video_path = format!(
            "{}/{}",
            self.config.local_storage_path, self.video_service.video.id
        );
        let concurrency = self.config.concurrency;
        let output_bucket = self.config.output_bucket_name.clone();

        let mut video_upload = VideoUpload::new(video_path, output_bucket);

        let (done_tx, mut done_rx) = tokio::sync::mpsc::channel::<String>(1);

        let spawn_tx = done_tx.clone();
        tokio::spawn(async move {
            if let Err(e) = video_upload.process_upload(concurrency, done_tx).await {
                let _ = spawn_tx.send(e.to_string()).await;
            }
        });

        let upload_result = done_rx
            .recv()
            .await
            .unwrap_or_else(|| "upload channel closed".to_string());

        if upload_result != "upload completed" {
            return Err(upload_result.into());
        }

        Ok(())
    }

    async fn change_job_status(
        &mut self,
        status: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        tracing::info!(job_id = %self.job.id, status, "job status changed");

        let mut updated = self.job.clone();
        updated.status = status
            .to_lowercase()
            .parse()
            .unwrap_or(crate::domain::JobStatus::Processing);
        updated.updated_at = chrono::Utc::now();

        self.job = self.job_repository.update(&updated).await?;

        Ok(())
    }

    async fn fail_job(&mut self, error: String) -> JobServiceError {
        tracing::error!(job_id = %self.job.id, error = %error, "job failed");

        let failed = self.job.fail(error.clone());

        if let Ok(updated) = self.job_repository.update(&failed).await {
            self.job = updated;
        }

        JobServiceError::Failed(error)
    }
    }
}

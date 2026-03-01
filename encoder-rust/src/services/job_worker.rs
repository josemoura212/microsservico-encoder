use std::sync::Arc;

use lapin::message::Delivery;
use serde::Deserialize;

use crate::config::Config;
use crate::db::Database;
use crate::domain::{Job, Video};
use crate::repositories::{JobRepository, Repository, VideoRepository};
use crate::services::{JobService, VideoService};

pub struct JobWorkerResult {
    pub job: Option<Job>,
    pub delivery: Delivery,
    pub error: Option<String>,
}

#[derive(Deserialize)]
struct VideoMessage {
    resource_id: String,
    file_path: String,
}

pub struct JobWorker<DB: sqlx::Database> {
    db: Database<DB>,
    config: Config,
}

impl<DB: sqlx::Database> Clone for JobWorker<DB> {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            config: self.config.clone(),
        }
    }
}

crate::db::impl_with_db! {
    impl<DB> JobWorker<DB> {
    pub fn new(db: Database<DB>, config: Config) -> Self {
        Self { db, config }
    }

    pub async fn process(&self, delivery: Delivery) -> JobWorkerResult {
        match self.try_process(&delivery).await {
            Ok(job) => JobWorkerResult {
                job: Some(job),
                delivery,
                error: None,
            },
            Err(e) => JobWorkerResult {
                job: None,
                delivery,
                error: Some(e),
            },
        }
    }

    async fn try_process(&self, delivery: &Delivery) -> Result<Job, String> {
        let msg: VideoMessage =
            serde_json::from_slice(&delivery.data).map_err(|e| format!("invalid JSON: {e}"))?;

        let video =
            Video::new(msg.resource_id, msg.file_path).map_err(|e| e.to_string())?;

        let video_repo = VideoRepository::new(self.db.clone());
        video_repo
            .insert(&video)
            .await
            .map_err(|e| format!("failed to insert video: {e}"))?;

        let video_arc = Arc::new(video.clone());
        let job = Job::new(self.config.output_bucket_name.clone(), video_arc);

        let job_repo = JobRepository::new(self.db.clone());
        job_repo
            .insert(&job)
            .await
            .map_err(|e| format!("failed to insert job: {e}"))?;

        let video_service = VideoService::new(
            VideoRepository::new(self.db.clone()),
            video,
            self.config.clone(),
        );

        let mut job_service = JobService {
            job,
            job_repository: JobRepository::new(self.db.clone()),
            video_service,
            config: self.config.clone(),
        };

        job_service.start().await.map_err(|e| e.to_string())?;

        Ok(job_service.job)
    }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_json_returns_error() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let db =
                crate::db::Database::<sqlx::Sqlite>::new("sqlite::memory:".to_string(), Some(true))
                    .await
                    .unwrap();

            let config = Config {
                database_url: String::new(),
                local_storage_path: "/tmp".to_string(),
                input_bucket_name: "test".to_string(),
                output_bucket_name: "test-out".to_string(),
                concurrency: 1,
            };

            let worker = JobWorker::new(db, config);

            let delivery = make_fake_delivery(b"not json");
            let result = worker.process(delivery).await;

            assert!(result.error.is_some());
            assert!(result.job.is_none());
            assert!(result.error.unwrap().contains("invalid JSON"));
        });
    }

    #[test]
    fn empty_resource_id_returns_error() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let db =
                crate::db::Database::<sqlx::Sqlite>::new("sqlite::memory:".to_string(), Some(true))
                    .await
                    .unwrap();

            let config = Config {
                database_url: String::new(),
                local_storage_path: "/tmp".to_string(),
                input_bucket_name: "test".to_string(),
                output_bucket_name: "test-out".to_string(),
                concurrency: 1,
            };

            let worker = JobWorker::new(db, config);

            let payload = br#"{"resource_id":"","file_path":"video.mp4"}"#;
            let delivery = make_fake_delivery(payload);
            let result = worker.process(delivery).await;

            assert!(result.error.is_some());
            assert!(result.error.unwrap().contains("resource_id"));
        });
    }

    fn make_fake_delivery(data: &[u8]) -> Delivery {
        use lapin::protocol::basic::AMQPProperties;
        use lapin::types::ShortString;

        Delivery {
            delivery_tag: 1,
            exchange: ShortString::from(""),
            routing_key: ShortString::from("videos"),
            redelivered: false,
            properties: AMQPProperties::default(),
            data: data.to_vec(),
            acker: lapin::acker::Acker::default(),
        }
    }
}

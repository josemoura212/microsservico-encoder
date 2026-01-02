use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::Video;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Job {
    #[serde(rename = "job_id")]
    pub id: Uuid,
    pub output_bucket_path: String,
    pub status: String,
    pub video: Arc<Video>,
    #[serde(skip)]
    pub video_id: Uuid,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Job {
    pub fn new(output_bucket_path: String, status: String, video: Arc<Video>) -> Job {
        let video_id = video.id;
        Job {
            id: Uuid::new_v4(),
            output_bucket_path,
            status,
            video,
            video_id,
            error: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

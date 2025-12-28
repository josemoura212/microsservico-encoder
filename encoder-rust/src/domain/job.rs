use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::Video;

pub struct Job<'a> {
    pub id: String,
    pub output_bucket_path: String,
    pub status: String,
    pub video: &'a Video,
    pub video_id: Option<String>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Job<'_> {
    pub fn new<'a>(output_bucket_path: String, status: String, video: &'a Video) -> Job<'a> {
        Job {
            id: Uuid::new_v4().to_string(),
            output_bucket_path,
            status,
            video,
            video_id: None,
            error: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

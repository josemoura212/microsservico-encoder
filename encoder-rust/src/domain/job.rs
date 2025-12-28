use chrono::{DateTime, Utc};

use crate::domain::Video;

pub struct Job<'a> {
    pub id: String,
    pub output_bucket_path: String,
    pub status: String,
    pub video: &'a Video,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

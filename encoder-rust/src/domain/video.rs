use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::Job;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Video {
    #[serde(rename = "encoded_video_folder")]
    pub id: Uuid,
    pub resource_id: String,
    pub file_path: String,
    #[serde(skip)]
    pub created_at: DateTime<Utc>,
    #[serde(skip)]
    pub jobs: Vec<Arc<Job>>,
}

impl Video {
    pub fn new(resource_id: String, file_path: String) -> Self {
        Video {
            id: Uuid::new_v4(),
            resource_id,
            file_path,
            created_at: Utc::now(),
            jobs: Vec::new(),
        }
    }
}

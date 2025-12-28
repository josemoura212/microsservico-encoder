use chrono::{DateTime, Utc};
use uuid::Uuid;

pub struct Video {
    pub id: String,
    pub resource_id: String,
    pub file_path: String,
    pub created_at: DateTime<Utc>,
}

impl Video {
    pub fn new(resource_id: String, file_path: String) -> Self {
        Video {
            id: Uuid::new_v4().to_string(),
            resource_id,
            file_path,
            created_at: Utc::now(),
        }
    }
}

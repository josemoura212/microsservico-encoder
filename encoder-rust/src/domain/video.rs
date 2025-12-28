use chrono::{DateTime, Utc};

pub struct Video {
    pub id: String,
    pub resource_id: String,
    pub file_path: String,
    pub created_at: DateTime<Utc>,
}

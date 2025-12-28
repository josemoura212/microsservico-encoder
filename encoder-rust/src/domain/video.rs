use chrono::{DateTime, Utc};

pub struct Video {
    pub id: String,
    pub resource_id: String,
    pub file_path: String,
    pub created_at: DateTime<Utc>,
}

impl Video {
    pub fn new(
        id: String,
        resource_id: String,
        file_path: String,
        created_at: DateTime<Utc>,
    ) -> Self {
        Video {
            id,
            resource_id,
            file_path,
            created_at,
        }
    }
}

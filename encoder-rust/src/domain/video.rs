use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{DomainError, Job};

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
    pub fn new(resource_id: String, file_path: String) -> Result<Self, DomainError> {
        if resource_id.is_empty() {
            return Err(DomainError::Validation(
                "resource_id cannot be empty".to_string(),
            ));
        }
        if file_path.is_empty() {
            return Err(DomainError::Validation(
                "file_path cannot be empty".to_string(),
            ));
        }

        Ok(Video {
            id: Uuid::new_v4(),
            resource_id,
            file_path,
            created_at: Utc::now(),
            jobs: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_new_valid() {
        let video = Video::new("resource_123".to_string(), "/path/to/video.mp4".to_string());
        assert!(video.is_ok());

        let video = video.unwrap();
        assert_eq!(video.resource_id, "resource_123");
        assert_eq!(video.file_path, "/path/to/video.mp4");
        assert!(video.jobs.is_empty());
    }

    #[test]
    fn test_video_new_empty_resource_id() {
        let result = Video::new(String::new(), "/path/to/video.mp4".to_string());
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(err.to_string().contains("resource_id cannot be empty"));
    }

    #[test]
    fn test_video_new_empty_file_path() {
        let result = Video::new("resource_123".to_string(), String::new());
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(err.to_string().contains("file_path cannot be empty"));
    }
}

use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::Video;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl JobStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Processing => "processing",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }
}

impl std::str::FromStr for JobStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(Self::Pending),
            "processing" => Ok(Self::Processing),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            other => Err(format!("unknown job status: {other}")),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Job {
    #[serde(rename = "job_id")]
    pub id: Uuid,
    pub output_bucket_path: String,
    pub status: JobStatus,
    pub video: Arc<Video>,
    #[serde(skip)]
    pub video_id: Uuid,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Job {
    pub fn new(output_bucket_path: String, video: Arc<Video>) -> Self {
        let video_id = video.id;
        Job {
            id: Uuid::new_v4(),
            output_bucket_path,
            status: JobStatus::Pending,
            video,
            video_id,
            error: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn start_processing(&self) -> Self {
        let mut updated = self.clone();
        updated.status = JobStatus::Processing;
        updated.updated_at = Utc::now();
        updated
    }

    pub fn complete(&self) -> Self {
        let mut updated = self.clone();
        updated.status = JobStatus::Completed;
        updated.updated_at = Utc::now();
        updated
    }

    pub fn fail(&self, error: String) -> Self {
        let mut updated = self.clone();
        updated.status = JobStatus::Failed;
        updated.error = Some(error);
        updated.updated_at = Utc::now();
        updated
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Video;

    fn make_video() -> Arc<Video> {
        Arc::new(Video {
            id: Uuid::new_v4(),
            resource_id: "res_1".to_string(),
            file_path: "/path/video.mp4".to_string(),
            created_at: Utc::now(),
            jobs: Vec::new(),
        })
    }

    #[test]
    fn test_job_new_defaults_to_pending() {
        let video = make_video();
        let job = Job::new("/output".to_string(), video.clone());

        assert_eq!(job.status, JobStatus::Pending);
        assert!(job.error.is_none());
        assert_eq!(job.video_id, video.id);
    }

    #[test]
    fn test_job_start_processing() {
        let job = Job::new("/output".to_string(), make_video());
        let processing = job.start_processing();

        assert_eq!(processing.status, JobStatus::Processing);
        assert_eq!(processing.id, job.id);
    }

    #[test]
    fn test_job_complete() {
        let job = Job::new("/output".to_string(), make_video());
        let completed = job.start_processing().complete();

        assert_eq!(completed.status, JobStatus::Completed);
        assert!(completed.error.is_none());
    }

    #[test]
    fn test_job_fail() {
        let job = Job::new("/output".to_string(), make_video());
        let failed = job.start_processing().fail("something broke".to_string());

        assert_eq!(failed.status, JobStatus::Failed);
        assert_eq!(failed.error.as_deref(), Some("something broke"));
    }

    #[test]
    fn test_job_status_from_str() {
        assert_eq!("pending".parse::<JobStatus>().unwrap(), JobStatus::Pending);
        assert_eq!(
            "processing".parse::<JobStatus>().unwrap(),
            JobStatus::Processing
        );
        assert_eq!(
            "completed".parse::<JobStatus>().unwrap(),
            JobStatus::Completed
        );
        assert_eq!("failed".parse::<JobStatus>().unwrap(), JobStatus::Failed);
        assert!("unknown".parse::<JobStatus>().is_err());
    }

    #[test]
    fn test_job_status_display() {
        assert_eq!(JobStatus::Pending.to_string(), "pending");
        assert_eq!(JobStatus::Processing.to_string(), "processing");
        assert_eq!(JobStatus::Completed.to_string(), "completed");
        assert_eq!(JobStatus::Failed.to_string(), "failed");
    }
}

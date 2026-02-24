use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::job::JobStatus;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProgressUpdate {
    pub invocation_id: Uuid,
    pub bytes_transferred: u64,
    pub percentage: f64,
    pub transfer_rate: String,
    pub elapsed: String,
    pub files_transferred: u64,
    pub files_remaining: u64,
    pub files_total: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogLine {
    pub invocation_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub line: String,
    pub is_stderr: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobStatusEvent {
    pub job_id: Uuid,
    pub invocation_id: Uuid,
    pub status: JobStatus,
    pub exit_code: Option<i32>,
    pub error_message: Option<String>,
}

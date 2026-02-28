use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RunStatistic {
    pub id: Uuid,
    pub job_id: Uuid,
    pub invocation_id: Uuid,
    pub recorded_at: DateTime<Utc>,
    pub files_transferred: u64,
    pub bytes_transferred: u64,
    pub duration_secs: f64,
    pub speedup: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AggregatedStats {
    pub total_jobs_run: u64,
    pub total_files_transferred: u64,
    pub total_bytes_transferred: u64,
    pub total_duration_secs: f64,
    pub total_time_saved_secs: f64,
}

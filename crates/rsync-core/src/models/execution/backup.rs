use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct TransferStats {
    pub bytes_transferred: u64,
    pub files_transferred: u64,
    pub total_files: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExecutionOutput {
    pub command_executed: String,
    pub exit_code: Option<i32>,
    pub snapshot_path: Option<String>,
    pub log_file_path: Option<String>,
}

impl Default for ExecutionOutput {
    fn default() -> Self {
        Self {
            command_executed: String::new(),
            exit_code: None,
            snapshot_path: None,
            log_file_path: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BackupInvocation {
    pub id: Uuid,
    pub job_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub status: InvocationStatus,
    pub trigger: InvocationTrigger,
    pub transfer_stats: TransferStats,
    pub execution_output: ExecutionOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InvocationStatus {
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InvocationTrigger {
    Manual,
    Scheduled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SnapshotRecord {
    pub id: Uuid,
    pub job_id: Uuid,
    pub invocation_id: Uuid,
    pub snapshot_path: String,
    pub link_dest_path: Option<String>,
    pub created_at: DateTime<Utc>,
    pub size_bytes: u64,
    pub file_count: u64,
    pub is_latest: bool,
}

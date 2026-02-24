use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogEntry {
    pub id: Uuid,
    pub invocation_id: Uuid,
    pub job_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
}

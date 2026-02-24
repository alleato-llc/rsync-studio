use serde::Serialize;
use thiserror::Error;

use crate::file_system::FsError;
use crate::rsync_client::RsyncError;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Rsync error: {0}")]
    RsyncError(#[from] RsyncError),

    #[error("File system error: {0}")]
    FileSystemError(#[from] FsError),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Scheduler error: {0}")]
    SchedulerError(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

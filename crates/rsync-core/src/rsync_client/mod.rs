pub mod process_rsync_client;

use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct RsyncResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub command: String,
}

#[derive(Debug, Error, Clone, PartialEq)]
pub enum RsyncError {
    #[error("Process error: {message} (exit code: {exit_code:?})")]
    ProcessError {
        message: String,
        exit_code: Option<i32>,
    },

    #[error("rsync not found on system")]
    RsyncNotFound,

    #[error("SSH error: {0}")]
    SshError(String),

    #[error("I/O error: {0}")]
    IoError(String),

    #[error("Operation cancelled")]
    Cancelled,
}

pub trait RsyncClient {
    fn execute(&self, args: &[String]) -> Result<RsyncResult, RsyncError>;

    fn dry_run(&self, args: &[String]) -> Result<RsyncResult, RsyncError>;

    fn version(&self) -> Result<String, RsyncError>;
}

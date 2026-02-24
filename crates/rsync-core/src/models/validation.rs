use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PreflightResult {
    pub job_id: Uuid,
    pub checks: Vec<ValidationCheck>,
    pub overall_pass: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValidationCheck {
    pub check_type: CheckType,
    pub passed: bool,
    pub message: String,
    pub severity: CheckSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CheckType {
    SourceExists,
    DestinationWritable,
    DiskSpace,
    SshConnectivity,
    RsyncInstalled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CheckSeverity {
    Error,
    Warning,
}

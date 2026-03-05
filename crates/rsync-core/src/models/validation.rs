use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export_to = "validation/")]
pub struct PreflightResult {
    pub job_id: Uuid,
    pub checks: Vec<ValidationCheck>,
    pub overall_pass: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export_to = "validation/")]
pub struct ValidationCheck {
    pub check_type: CheckType,
    pub passed: bool,
    pub message: String,
    pub severity: CheckSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export_to = "validation/")]
pub enum CheckType {
    SourceExists,
    DestinationWritable,
    DiskSpace,
    SshConnectivity,
    RsyncInstalled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export_to = "validation/")]
pub enum CheckSeverity {
    Error,
    Warning,
}

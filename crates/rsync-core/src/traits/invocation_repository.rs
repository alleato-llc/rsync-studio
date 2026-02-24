use uuid::Uuid;

use crate::error::AppError;
use crate::models::backup::BackupInvocation;

pub trait InvocationRepository: Send + Sync {
    fn create_invocation(&self, inv: &BackupInvocation) -> Result<(), AppError>;
    fn get_invocation(&self, id: &Uuid) -> Result<BackupInvocation, AppError>;
    fn list_invocations_for_job(&self, job_id: &Uuid) -> Result<Vec<BackupInvocation>, AppError>;
    fn update_invocation(&self, inv: &BackupInvocation) -> Result<(), AppError>;
}

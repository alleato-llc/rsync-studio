use uuid::Uuid;

use crate::error::AppError;
use crate::models::backup::SnapshotRecord;

pub trait SnapshotRepository: Send + Sync {
    fn create_snapshot(&self, snapshot: &SnapshotRecord) -> Result<(), AppError>;
    fn get_latest_snapshot_for_job(&self, job_id: &Uuid) -> Result<Option<SnapshotRecord>, AppError>;
    fn list_snapshots_for_job(&self, job_id: &Uuid) -> Result<Vec<SnapshotRecord>, AppError>;
    fn delete_snapshot(&self, id: &Uuid) -> Result<(), AppError>;
}

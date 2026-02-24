use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::backup::{BackupInvocation, SnapshotRecord};
use crate::models::job::{BackupMode, JobDefinition};
use crate::services::retention;
use crate::traits::invocation_repository::InvocationRepository;
use crate::traits::job_repository::JobRepository;
use crate::traits::snapshot_repository::SnapshotRepository;

pub struct JobService {
    jobs: Arc<dyn JobRepository>,
    invocations: Arc<dyn InvocationRepository>,
    snapshots: Arc<dyn SnapshotRepository>,
}

impl JobService {
    pub fn new(
        jobs: Arc<dyn JobRepository>,
        invocations: Arc<dyn InvocationRepository>,
        snapshots: Arc<dyn SnapshotRepository>,
    ) -> Self {
        Self {
            jobs,
            invocations,
            snapshots,
        }
    }

    pub fn create_job(&self, mut job: JobDefinition) -> Result<JobDefinition, AppError> {
        if job.name.trim().is_empty() {
            return Err(AppError::ValidationError(
                "Job name must not be empty".to_string(),
            ));
        }
        job.id = Uuid::new_v4();
        let now = Utc::now();
        job.created_at = now;
        job.updated_at = now;
        self.jobs.create_job(&job)?;
        Ok(job)
    }

    pub fn update_job(&self, mut job: JobDefinition) -> Result<JobDefinition, AppError> {
        if job.name.trim().is_empty() {
            return Err(AppError::ValidationError(
                "Job name must not be empty".to_string(),
            ));
        }
        // Verify job exists
        self.jobs.get_job(&job.id)?;
        job.updated_at = Utc::now();
        self.jobs.update_job(&job)?;
        Ok(job)
    }

    pub fn delete_job(&self, id: &Uuid) -> Result<(), AppError> {
        // Verify job exists
        self.jobs.get_job(id)?;
        self.jobs.delete_job(id)
    }

    pub fn get_job(&self, id: &Uuid) -> Result<JobDefinition, AppError> {
        self.jobs.get_job(id)
    }

    pub fn list_jobs(&self) -> Result<Vec<JobDefinition>, AppError> {
        self.jobs.list_jobs()
    }

    pub fn record_invocation(&self, inv: &BackupInvocation) -> Result<(), AppError> {
        self.invocations.create_invocation(inv)
    }

    pub fn complete_invocation(&self, inv: &BackupInvocation) -> Result<(), AppError> {
        self.invocations.update_invocation(inv)
    }

    pub fn get_invocation(&self, id: &Uuid) -> Result<BackupInvocation, AppError> {
        self.invocations.get_invocation(id)
    }

    pub fn delete_invocation(&self, id: &Uuid) -> Result<(), AppError> {
        self.invocations.delete_invocation(id)
    }

    pub fn delete_invocations_for_job(&self, job_id: &Uuid) -> Result<(), AppError> {
        self.invocations.delete_invocations_for_job(job_id)
    }

    pub fn list_all_invocations(&self) -> Result<Vec<BackupInvocation>, AppError> {
        self.invocations.list_all_invocations()
    }

    pub fn get_job_history(
        &self,
        job_id: &Uuid,
        limit: usize,
    ) -> Result<Vec<BackupInvocation>, AppError> {
        let mut invocations = self.invocations.list_invocations_for_job(job_id)?;
        invocations.sort_by(|a, b| b.started_at.cmp(&a.started_at));
        invocations.truncate(limit);
        Ok(invocations)
    }

    pub fn record_snapshot(&self, snapshot: &SnapshotRecord) -> Result<(), AppError> {
        self.snapshots.create_snapshot(snapshot)
    }

    pub fn get_latest_snapshot(
        &self,
        job_id: &Uuid,
    ) -> Result<Option<SnapshotRecord>, AppError> {
        self.snapshots.get_latest_snapshot_for_job(job_id)
    }

    pub fn list_snapshots(
        &self,
        job_id: &Uuid,
    ) -> Result<Vec<SnapshotRecord>, AppError> {
        self.snapshots.list_snapshots_for_job(job_id)
    }

    pub fn delete_snapshot(&self, id: &Uuid) -> Result<(), AppError> {
        self.snapshots.delete_snapshot(id)
    }

    /// Apply the retention policy for a snapshot-mode job.
    ///
    /// Returns the list of snapshot paths that were pruned from the database.
    /// The caller is responsible for deleting the actual directories on disk.
    pub fn apply_retention_policy(
        &self,
        job_id: &Uuid,
    ) -> Result<Vec<String>, AppError> {
        let job = self.jobs.get_job(job_id)?;
        let policy = match &job.backup_mode {
            BackupMode::Snapshot { retention_policy } => retention_policy,
            _ => return Ok(Vec::new()), // Not a snapshot job, nothing to do
        };

        let snapshots = self.snapshots.list_snapshots_for_job(job_id)?;
        let to_delete = retention::compute_snapshots_to_delete(&snapshots, policy);

        let mut pruned_paths = Vec::new();
        for snap_id in &to_delete {
            if let Some(snap) = snapshots.iter().find(|s| s.id == *snap_id) {
                pruned_paths.push(snap.snapshot_path.clone());
            }
            self.snapshots.delete_snapshot(snap_id)?;
        }

        Ok(pruned_paths)
    }
}

use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::implementations::database::Database;
use crate::implementations::sqlite_invocation_repository::SqliteInvocationRepository;
use crate::implementations::sqlite_job_repository::SqliteJobRepository;
use crate::implementations::sqlite_snapshot_repository::SqliteSnapshotRepository;
use crate::models::backup::{
    BackupInvocation, InvocationStatus, InvocationTrigger, SnapshotRecord,
};
use crate::models::job::{BackupMode, JobDefinition, RsyncOptions, StorageLocation};
use crate::services::job_service::JobService;

fn setup() -> JobService {
    let db = Database::in_memory().unwrap();
    let conn = db.conn();
    JobService::new(
        Arc::new(SqliteJobRepository::new(conn.clone())),
        Arc::new(SqliteInvocationRepository::new(conn.clone())),
        Arc::new(SqliteSnapshotRepository::new(conn)),
    )
}

fn make_job_definition(name: &str) -> JobDefinition {
    let now = Utc::now();
    JobDefinition {
        id: Uuid::new_v4(),
        name: name.to_string(),
        description: Some("Test job".to_string()),
        source: StorageLocation::Local {
            path: "/src/".to_string(),
        },
        destination: StorageLocation::Local {
            path: "/dst/".to_string(),
        },
        backup_mode: BackupMode::Mirror,
        options: RsyncOptions::default(),
        ssh_config: None,
        schedule: None,
        enabled: true,
        created_at: now,
        updated_at: now,
    }
}

fn make_invocation(job_id: Uuid) -> BackupInvocation {
    BackupInvocation {
        id: Uuid::new_v4(),
        job_id,
        started_at: Utc::now(),
        finished_at: None,
        status: InvocationStatus::Running,
        bytes_transferred: 0,
        files_transferred: 0,
        total_files: 50,
        snapshot_path: None,
        command_executed: "rsync -a /src/ /dst/".to_string(),
        exit_code: None,
        trigger: InvocationTrigger::Manual,
        log_file_path: None,
    }
}

#[test]
fn test_create_and_retrieve_job() {
    let svc = setup();
    let job_def = make_job_definition("My Backup");
    let created = svc.create_job(job_def).unwrap();

    let retrieved = svc.get_job(&created.id).unwrap();
    assert_eq!(retrieved.name, "My Backup");
    assert_eq!(retrieved.id, created.id);
}

#[test]
fn test_create_job_empty_name_fails() {
    let svc = setup();
    let job_def = make_job_definition("");
    let result = svc.create_job(job_def);
    assert!(result.is_err());
}

#[test]
fn test_create_job_whitespace_name_fails() {
    let svc = setup();
    let job_def = make_job_definition("   ");
    let result = svc.create_job(job_def);
    assert!(result.is_err());
}

#[test]
fn test_update_job_changes_updated_at() {
    let svc = setup();
    let job_def = make_job_definition("Original");
    let created = svc.create_job(job_def).unwrap();
    let original_updated_at = created.updated_at;

    // Small delay to ensure timestamp differs
    std::thread::sleep(std::time::Duration::from_millis(10));

    let mut to_update = created;
    to_update.name = "Updated".to_string();
    let updated = svc.update_job(to_update).unwrap();

    assert_eq!(updated.name, "Updated");
    assert!(updated.updated_at > original_updated_at);
}

#[test]
fn test_delete_job_cascades() {
    let svc = setup();
    let job = svc.create_job(make_job_definition("To Delete")).unwrap();

    // Add an invocation
    let inv = make_invocation(job.id);
    svc.record_invocation(&inv).unwrap();

    // Add a snapshot
    let snap = SnapshotRecord {
        id: Uuid::new_v4(),
        job_id: job.id,
        invocation_id: inv.id,
        snapshot_path: "/backups/snap1".to_string(),
        link_dest_path: None,
        created_at: Utc::now(),
        size_bytes: 1024,
        file_count: 3,
        is_latest: true,
    };
    svc.record_snapshot(&snap).unwrap();

    // Delete the job
    svc.delete_job(&job.id).unwrap();

    // Job should be gone
    assert!(svc.get_job(&job.id).is_err());
    // Cascaded invocations/snapshots should be gone too
    let history = svc.get_job_history(&job.id, 10).unwrap();
    assert!(history.is_empty());
    let latest = svc.get_latest_snapshot(&job.id).unwrap();
    assert!(latest.is_none());
}

#[test]
fn test_full_lifecycle() {
    let svc = setup();

    // Create job
    let job = svc.create_job(make_job_definition("Lifecycle Test")).unwrap();

    // Record invocation
    let mut inv = make_invocation(job.id);
    svc.record_invocation(&inv).unwrap();

    // Complete invocation
    inv.status = InvocationStatus::Succeeded;
    inv.finished_at = Some(Utc::now());
    inv.bytes_transferred = 2048;
    inv.files_transferred = 10;
    inv.exit_code = Some(0);
    svc.complete_invocation(&inv).unwrap();

    // Record snapshot
    let snap = SnapshotRecord {
        id: Uuid::new_v4(),
        job_id: job.id,
        invocation_id: inv.id,
        snapshot_path: "/backups/2024-01-01".to_string(),
        link_dest_path: None,
        created_at: Utc::now(),
        size_bytes: 2048,
        file_count: 10,
        is_latest: true,
    };
    svc.record_snapshot(&snap).unwrap();

    // Query history
    let history = svc.get_job_history(&job.id, 10).unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].status, InvocationStatus::Succeeded);

    // Latest snapshot
    let latest = svc.get_latest_snapshot(&job.id).unwrap().unwrap();
    assert_eq!(latest.snapshot_path, "/backups/2024-01-01");
}

#[test]
fn test_get_job_history_respects_limit() {
    let svc = setup();
    let job = svc.create_job(make_job_definition("History Test")).unwrap();

    for _ in 0..5 {
        let inv = make_invocation(job.id);
        svc.record_invocation(&inv).unwrap();
    }

    let history = svc.get_job_history(&job.id, 3).unwrap();
    assert_eq!(history.len(), 3);
}

#[test]
fn test_list_multiple_jobs() {
    let svc = setup();
    svc.create_job(make_job_definition("Job A")).unwrap();
    svc.create_job(make_job_definition("Job B")).unwrap();
    svc.create_job(make_job_definition("Job C")).unwrap();

    let jobs = svc.list_jobs().unwrap();
    assert_eq!(jobs.len(), 3);
}

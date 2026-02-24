use chrono::Utc;
use uuid::Uuid;

use crate::database::sqlite::Database;
use crate::models::backup::{
    BackupInvocation, InvocationStatus, InvocationTrigger, SnapshotRecord,
};
use crate::repository::invocation::InvocationRepository;
use crate::repository::job::JobRepository;
use crate::repository::snapshot::SnapshotRepository;
use crate::repository::sqlite::invocation::SqliteInvocationRepository;
use crate::repository::sqlite::job::SqliteJobRepository;
use crate::repository::sqlite::snapshot::SqliteSnapshotRepository;
use crate::tests::test_helpers::create_test_job;

fn setup() -> (
    SqliteJobRepository,
    SqliteInvocationRepository,
    SqliteSnapshotRepository,
) {
    let db = Database::in_memory().unwrap();
    let conn = db.conn();
    (
        SqliteJobRepository::new(conn.clone()),
        SqliteInvocationRepository::new(conn.clone()),
        SqliteSnapshotRepository::new(conn),
    )
}

fn make_invocation(job_id: Uuid) -> BackupInvocation {
    BackupInvocation {
        id: Uuid::new_v4(),
        job_id,
        started_at: Utc::now(),
        finished_at: Some(Utc::now()),
        status: InvocationStatus::Succeeded,
        bytes_transferred: 1024,
        files_transferred: 5,
        total_files: 5,
        snapshot_path: Some("/backups/snap1".to_string()),
        command_executed: "rsync -a /src/ /dst/".to_string(),
        exit_code: Some(0),
        trigger: InvocationTrigger::Manual,
        log_file_path: None,
    }
}

fn make_snapshot(job_id: Uuid, invocation_id: Uuid) -> SnapshotRecord {
    SnapshotRecord {
        id: Uuid::new_v4(),
        job_id,
        invocation_id,
        snapshot_path: "/backups/snap1".to_string(),
        link_dest_path: None,
        created_at: Utc::now(),
        size_bytes: 2048,
        file_count: 5,
        is_latest: true,
    }
}

#[test]
fn test_create_and_list_snapshots() {
    let (job_repo, inv_repo, snap_repo) = setup();
    let job = create_test_job();
    job_repo.create_job(&job).unwrap();
    let inv = make_invocation(job.id);
    inv_repo.create_invocation(&inv).unwrap();

    let snap = make_snapshot(job.id, inv.id);
    snap_repo.create_snapshot(&snap).unwrap();

    let list = snap_repo.list_snapshots_for_job(&job.id).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, snap.id);
    assert_eq!(list[0].snapshot_path, snap.snapshot_path);
}

#[test]
fn test_get_latest_snapshot() {
    let (job_repo, inv_repo, snap_repo) = setup();
    let job = create_test_job();
    job_repo.create_job(&job).unwrap();

    let inv1 = make_invocation(job.id);
    inv_repo.create_invocation(&inv1).unwrap();
    let mut snap1 = make_snapshot(job.id, inv1.id);
    snap1.snapshot_path = "/backups/old".to_string();
    snap1.is_latest = false;
    snap1.created_at = Utc::now() - chrono::Duration::hours(1);
    snap_repo.create_snapshot(&snap1).unwrap();

    let inv2 = make_invocation(job.id);
    inv_repo.create_invocation(&inv2).unwrap();
    let mut snap2 = make_snapshot(job.id, inv2.id);
    snap2.snapshot_path = "/backups/new".to_string();
    snap2.is_latest = true;
    snap_repo.create_snapshot(&snap2).unwrap();

    let latest = snap_repo
        .get_latest_snapshot_for_job(&job.id)
        .unwrap()
        .unwrap();
    assert_eq!(latest.snapshot_path, "/backups/new");
}

#[test]
fn test_delete_snapshot() {
    let (job_repo, inv_repo, snap_repo) = setup();
    let job = create_test_job();
    job_repo.create_job(&job).unwrap();
    let inv = make_invocation(job.id);
    inv_repo.create_invocation(&inv).unwrap();
    let snap = make_snapshot(job.id, inv.id);
    snap_repo.create_snapshot(&snap).unwrap();

    snap_repo.delete_snapshot(&snap.id).unwrap();
    let list = snap_repo.list_snapshots_for_job(&job.id).unwrap();
    assert!(list.is_empty());
}

#[test]
fn test_cascade_delete_snapshots() {
    let (job_repo, inv_repo, snap_repo) = setup();
    let job = create_test_job();
    job_repo.create_job(&job).unwrap();
    let inv = make_invocation(job.id);
    inv_repo.create_invocation(&inv).unwrap();
    let snap = make_snapshot(job.id, inv.id);
    snap_repo.create_snapshot(&snap).unwrap();

    // Delete job — snapshots should cascade
    job_repo.delete_job(&job.id).unwrap();
    let list = snap_repo.list_snapshots_for_job(&job.id).unwrap();
    assert!(list.is_empty());
}

#[test]
fn test_no_latest_snapshot() {
    let (_job_repo, _inv_repo, snap_repo) = setup();
    let result = snap_repo
        .get_latest_snapshot_for_job(&Uuid::new_v4())
        .unwrap();
    assert!(result.is_none());
}

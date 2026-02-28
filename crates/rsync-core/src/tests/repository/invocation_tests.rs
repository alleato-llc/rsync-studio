use chrono::Utc;
use uuid::Uuid;

use crate::database::sqlite::Database;
use crate::models::backup::{
    BackupInvocation, ExecutionOutput, InvocationStatus, InvocationTrigger, TransferStats,
};
use crate::repository::invocation::InvocationRepository;
use crate::repository::job::JobRepository;
use crate::repository::sqlite::invocation::SqliteInvocationRepository;
use crate::repository::sqlite::job::SqliteJobRepository;
use crate::tests::test_helpers::create_test_job;

fn setup() -> (SqliteJobRepository, SqliteInvocationRepository) {
    let db = Database::in_memory().unwrap();
    let conn = db.conn();
    (
        SqliteJobRepository::new(conn.clone()),
        SqliteInvocationRepository::new(conn),
    )
}

fn make_invocation(job_id: Uuid) -> BackupInvocation {
    BackupInvocation {
        id: Uuid::new_v4(),
        job_id,
        started_at: Utc::now(),
        finished_at: None,
        status: InvocationStatus::Running,
        trigger: InvocationTrigger::Manual,
        transfer_stats: TransferStats {
            bytes_transferred: 0,
            files_transferred: 0,
            total_files: 100,
        },
        execution_output: ExecutionOutput {
            command_executed: "rsync -a /src/ /dst/".to_string(),
            exit_code: None,
            snapshot_path: None,
            log_file_path: Some("/var/log/rsync/test.log".to_string()),
        },
    }
}

#[test]
fn test_create_and_get_invocation() {
    let (job_repo, inv_repo) = setup();
    let job = create_test_job();
    job_repo.create_job(&job).unwrap();

    let inv = make_invocation(job.id);
    inv_repo.create_invocation(&inv).unwrap();

    let retrieved = inv_repo.get_invocation(&inv.id).unwrap();
    assert_eq!(retrieved.id, inv.id);
    assert_eq!(retrieved.job_id, inv.job_id);
    assert_eq!(retrieved.status, InvocationStatus::Running);
    assert_eq!(retrieved.execution_output.command_executed, inv.execution_output.command_executed);
    assert_eq!(retrieved.execution_output.log_file_path, inv.execution_output.log_file_path);
}

#[test]
fn test_update_invocation() {
    let (job_repo, inv_repo) = setup();
    let job = create_test_job();
    job_repo.create_job(&job).unwrap();

    let mut inv = make_invocation(job.id);
    inv_repo.create_invocation(&inv).unwrap();

    inv.status = InvocationStatus::Succeeded;
    inv.finished_at = Some(Utc::now());
    inv.transfer_stats.bytes_transferred = 1024;
    inv.transfer_stats.files_transferred = 10;
    inv.execution_output.exit_code = Some(0);
    inv_repo.update_invocation(&inv).unwrap();

    let retrieved = inv_repo.get_invocation(&inv.id).unwrap();
    assert_eq!(retrieved.status, InvocationStatus::Succeeded);
    assert!(retrieved.finished_at.is_some());
    assert_eq!(retrieved.transfer_stats.bytes_transferred, 1024);
    assert_eq!(retrieved.execution_output.exit_code, Some(0));
}

#[test]
fn test_list_invocations_for_job() {
    let (job_repo, inv_repo) = setup();
    let job = create_test_job();
    job_repo.create_job(&job).unwrap();

    let inv1 = make_invocation(job.id);
    let inv2 = make_invocation(job.id);
    inv_repo.create_invocation(&inv1).unwrap();
    inv_repo.create_invocation(&inv2).unwrap();

    let list = inv_repo.list_invocations_for_job(&job.id).unwrap();
    assert_eq!(list.len(), 2);
}

#[test]
fn test_cascade_delete_invocations() {
    let (job_repo, inv_repo) = setup();
    let job = create_test_job();
    job_repo.create_job(&job).unwrap();

    let inv = make_invocation(job.id);
    inv_repo.create_invocation(&inv).unwrap();

    // Delete the job — invocations should cascade
    job_repo.delete_job(&job.id).unwrap();

    let result = inv_repo.get_invocation(&inv.id);
    assert!(result.is_err());
}

#[test]
fn test_get_nonexistent_invocation() {
    let (_job_repo, inv_repo) = setup();
    let result = inv_repo.get_invocation(&Uuid::new_v4());
    assert!(result.is_err());
}

#[test]
fn test_delete_invocation() {
    let (job_repo, inv_repo) = setup();
    let job = create_test_job();
    job_repo.create_job(&job).unwrap();

    let inv = make_invocation(job.id);
    inv_repo.create_invocation(&inv).unwrap();

    inv_repo.delete_invocation(&inv.id).unwrap();
    let result = inv_repo.get_invocation(&inv.id);
    assert!(result.is_err());
}

#[test]
fn test_delete_nonexistent_invocation() {
    let (_job_repo, inv_repo) = setup();
    let result = inv_repo.delete_invocation(&Uuid::new_v4());
    assert!(result.is_err());
}

#[test]
fn test_delete_invocations_for_job() {
    let (job_repo, inv_repo) = setup();
    let job = create_test_job();
    job_repo.create_job(&job).unwrap();

    let inv1 = make_invocation(job.id);
    let inv2 = make_invocation(job.id);
    inv_repo.create_invocation(&inv1).unwrap();
    inv_repo.create_invocation(&inv2).unwrap();

    inv_repo.delete_invocations_for_job(&job.id).unwrap();
    let list = inv_repo.list_invocations_for_job(&job.id).unwrap();
    assert!(list.is_empty());
}

#[test]
fn test_list_all_invocations() {
    let (job_repo, inv_repo) = setup();
    let job1 = create_test_job();
    let mut job2 = create_test_job();
    job2.name = "Other job".to_string();
    job_repo.create_job(&job1).unwrap();
    job_repo.create_job(&job2).unwrap();

    let inv1 = make_invocation(job1.id);
    let inv2 = make_invocation(job2.id);
    inv_repo.create_invocation(&inv1).unwrap();
    inv_repo.create_invocation(&inv2).unwrap();

    let all = inv_repo.list_all_invocations().unwrap();
    assert_eq!(all.len(), 2);
}

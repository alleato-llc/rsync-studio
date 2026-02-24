use std::sync::Arc;

use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::implementations::database::Database;
use crate::implementations::sqlite_invocation_repository::SqliteInvocationRepository;
use crate::implementations::sqlite_job_repository::SqliteJobRepository;
use crate::implementations::sqlite_statistics_repository::SqliteStatisticsRepository;
use crate::models::backup::{BackupInvocation, InvocationStatus, InvocationTrigger};
use crate::services::statistics_service::StatisticsService;
use crate::tests::test_helpers::create_test_job;
use crate::traits::invocation_repository::InvocationRepository;
use crate::traits::job_repository::JobRepository;

fn setup() -> (
    SqliteJobRepository,
    SqliteInvocationRepository,
    StatisticsService,
) {
    let db = Database::in_memory().unwrap();
    let conn = db.conn();
    let stats_repo = Arc::new(SqliteStatisticsRepository::new(conn.clone()));
    (
        SqliteJobRepository::new(conn.clone()),
        SqliteInvocationRepository::new(conn),
        StatisticsService::new(stats_repo),
    )
}

fn make_completed_invocation(job_id: Uuid, bytes: u64, files: u64) -> BackupInvocation {
    let started = Utc::now() - Duration::seconds(10);
    BackupInvocation {
        id: Uuid::new_v4(),
        job_id,
        started_at: started,
        finished_at: Some(Utc::now()),
        status: InvocationStatus::Succeeded,
        bytes_transferred: bytes,
        files_transferred: files,
        total_files: files,
        snapshot_path: None,
        command_executed: "rsync -a /src/ /dst/".to_string(),
        exit_code: Some(0),
        trigger: InvocationTrigger::Manual,
        log_file_path: None,
    }
}

#[test]
fn test_record_and_aggregate() {
    let (job_repo, inv_repo, stats_service) = setup();
    let job = create_test_job();
    job_repo.create_job(&job).unwrap();

    let inv = make_completed_invocation(job.id, 1024, 5);
    inv_repo.create_invocation(&inv).unwrap();

    stats_service.record(job.id, &inv, Some(3.0)).unwrap();

    let agg = stats_service.get_aggregated().unwrap();
    assert_eq!(agg.total_jobs_run, 1);
    assert_eq!(agg.total_files_transferred, 5);
    assert_eq!(agg.total_bytes_transferred, 1024);
    assert!(agg.total_duration_secs > 0.0);
    assert!(agg.total_time_saved_secs > 0.0);
}

#[test]
fn test_aggregate_for_job() {
    let (job_repo, inv_repo, stats_service) = setup();
    let job1 = create_test_job();
    let mut job2 = create_test_job();
    job2.name = "Other job".to_string();
    job_repo.create_job(&job1).unwrap();
    job_repo.create_job(&job2).unwrap();

    let inv1 = make_completed_invocation(job1.id, 1024, 5);
    let inv2 = make_completed_invocation(job2.id, 2048, 10);
    inv_repo.create_invocation(&inv1).unwrap();
    inv_repo.create_invocation(&inv2).unwrap();

    stats_service.record(job1.id, &inv1, Some(2.0)).unwrap();
    stats_service.record(job2.id, &inv2, Some(4.0)).unwrap();

    let agg1 = stats_service.get_aggregated_for_job(&job1.id).unwrap();
    assert_eq!(agg1.total_jobs_run, 1);
    assert_eq!(agg1.total_bytes_transferred, 1024);

    let agg_all = stats_service.get_aggregated().unwrap();
    assert_eq!(agg_all.total_jobs_run, 2);
    assert_eq!(agg_all.total_bytes_transferred, 1024 + 2048);
}

#[test]
fn test_export_json() {
    let (job_repo, inv_repo, stats_service) = setup();
    let job = create_test_job();
    job_repo.create_job(&job).unwrap();

    let inv = make_completed_invocation(job.id, 512, 3);
    inv_repo.create_invocation(&inv).unwrap();

    stats_service.record(job.id, &inv, None).unwrap();

    let json = stats_service.export().unwrap();
    assert!(json.contains("files_transferred"));
    assert!(json.contains("bytes_transferred"));
}

#[test]
fn test_reset() {
    let (job_repo, inv_repo, stats_service) = setup();
    let job = create_test_job();
    job_repo.create_job(&job).unwrap();

    let inv = make_completed_invocation(job.id, 1024, 5);
    inv_repo.create_invocation(&inv).unwrap();
    stats_service.record(job.id, &inv, Some(2.0)).unwrap();

    assert_eq!(stats_service.get_aggregated().unwrap().total_jobs_run, 1);

    stats_service.reset().unwrap();
    assert_eq!(stats_service.get_aggregated().unwrap().total_jobs_run, 0);
}

#[test]
fn test_reset_for_job() {
    let (job_repo, inv_repo, stats_service) = setup();
    let job1 = create_test_job();
    let mut job2 = create_test_job();
    job2.name = "Other job".to_string();
    job_repo.create_job(&job1).unwrap();
    job_repo.create_job(&job2).unwrap();

    let inv1 = make_completed_invocation(job1.id, 1024, 5);
    let inv2 = make_completed_invocation(job2.id, 2048, 10);
    inv_repo.create_invocation(&inv1).unwrap();
    inv_repo.create_invocation(&inv2).unwrap();

    stats_service.record(job1.id, &inv1, None).unwrap();
    stats_service.record(job2.id, &inv2, None).unwrap();

    stats_service.reset_for_job(&job1.id).unwrap();

    let agg = stats_service.get_aggregated().unwrap();
    assert_eq!(agg.total_jobs_run, 1);
    assert_eq!(agg.total_bytes_transferred, 2048);
}

#[test]
fn test_no_speedup_means_no_time_saved() {
    let (job_repo, inv_repo, stats_service) = setup();
    let job = create_test_job();
    job_repo.create_job(&job).unwrap();

    let inv = make_completed_invocation(job.id, 1024, 5);
    inv_repo.create_invocation(&inv).unwrap();

    stats_service.record(job.id, &inv, None).unwrap();

    let agg = stats_service.get_aggregated().unwrap();
    assert_eq!(agg.total_time_saved_secs, 0.0);
}

#[test]
fn test_speedup_leq_one_no_time_saved() {
    let (job_repo, inv_repo, stats_service) = setup();
    let job = create_test_job();
    job_repo.create_job(&job).unwrap();

    let inv = make_completed_invocation(job.id, 1024, 5);
    inv_repo.create_invocation(&inv).unwrap();

    stats_service.record(job.id, &inv, Some(1.0)).unwrap();

    let agg = stats_service.get_aggregated().unwrap();
    assert_eq!(agg.total_time_saved_secs, 0.0);
}

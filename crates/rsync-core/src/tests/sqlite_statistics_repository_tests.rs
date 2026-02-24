use chrono::Utc;
use uuid::Uuid;

use crate::implementations::database::Database;
use crate::implementations::sqlite_invocation_repository::SqliteInvocationRepository;
use crate::implementations::sqlite_job_repository::SqliteJobRepository;
use crate::implementations::sqlite_statistics_repository::SqliteStatisticsRepository;
use crate::models::backup::{BackupInvocation, InvocationStatus, InvocationTrigger};
use crate::models::statistics::RunStatistic;
use crate::tests::test_helpers::create_test_job;
use crate::traits::invocation_repository::InvocationRepository;
use crate::traits::job_repository::JobRepository;
use crate::traits::statistics_repository::StatisticsRepository;

fn setup() -> (
    SqliteJobRepository,
    SqliteInvocationRepository,
    SqliteStatisticsRepository,
) {
    let db = Database::in_memory().unwrap();
    let conn = db.conn();
    (
        SqliteJobRepository::new(conn.clone()),
        SqliteInvocationRepository::new(conn.clone()),
        SqliteStatisticsRepository::new(conn),
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
        files_transferred: 10,
        total_files: 100,
        snapshot_path: None,
        command_executed: "rsync -a /src/ /dst/".to_string(),
        exit_code: Some(0),
        trigger: InvocationTrigger::Manual,
        log_file_path: None,
    }
}

fn make_statistic(job_id: Uuid, invocation_id: Uuid) -> RunStatistic {
    RunStatistic {
        id: Uuid::new_v4(),
        job_id,
        invocation_id,
        recorded_at: Utc::now(),
        files_transferred: 10,
        bytes_transferred: 1024,
        duration_secs: 5.5,
        speedup: Some(2.5),
    }
}

#[test]
fn test_record_and_get_statistics() {
    let (job_repo, inv_repo, stats_repo) = setup();
    let job = create_test_job();
    job_repo.create_job(&job).unwrap();
    let inv = make_invocation(job.id);
    inv_repo.create_invocation(&inv).unwrap();

    let stat = make_statistic(job.id, inv.id);
    stats_repo.record_statistic(&stat).unwrap();

    let all = stats_repo.get_all_statistics().unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].id, stat.id);
    assert_eq!(all[0].files_transferred, 10);
    assert_eq!(all[0].bytes_transferred, 1024);
    assert!((all[0].duration_secs - 5.5).abs() < 0.01);
    assert_eq!(all[0].speedup, Some(2.5));
}

#[test]
fn test_get_statistics_for_job() {
    let (job_repo, inv_repo, stats_repo) = setup();
    let job1 = create_test_job();
    let mut job2 = create_test_job();
    job2.name = "Other job".to_string();
    job_repo.create_job(&job1).unwrap();
    job_repo.create_job(&job2).unwrap();

    let inv1 = make_invocation(job1.id);
    let inv2 = make_invocation(job2.id);
    inv_repo.create_invocation(&inv1).unwrap();
    inv_repo.create_invocation(&inv2).unwrap();

    stats_repo
        .record_statistic(&make_statistic(job1.id, inv1.id))
        .unwrap();
    stats_repo
        .record_statistic(&make_statistic(job2.id, inv2.id))
        .unwrap();

    let job1_stats = stats_repo.get_statistics_for_job(&job1.id).unwrap();
    assert_eq!(job1_stats.len(), 1);
    assert_eq!(job1_stats[0].job_id, job1.id);
}

#[test]
fn test_delete_statistics_for_job() {
    let (job_repo, inv_repo, stats_repo) = setup();
    let job = create_test_job();
    job_repo.create_job(&job).unwrap();
    let inv = make_invocation(job.id);
    inv_repo.create_invocation(&inv).unwrap();

    stats_repo
        .record_statistic(&make_statistic(job.id, inv.id))
        .unwrap();
    assert_eq!(stats_repo.get_all_statistics().unwrap().len(), 1);

    stats_repo.delete_statistics_for_job(&job.id).unwrap();
    assert_eq!(stats_repo.get_all_statistics().unwrap().len(), 0);
}

#[test]
fn test_delete_all_statistics() {
    let (job_repo, inv_repo, stats_repo) = setup();
    let job = create_test_job();
    job_repo.create_job(&job).unwrap();

    let inv1 = make_invocation(job.id);
    let inv2 = make_invocation(job.id);
    inv_repo.create_invocation(&inv1).unwrap();
    inv_repo.create_invocation(&inv2).unwrap();

    stats_repo
        .record_statistic(&make_statistic(job.id, inv1.id))
        .unwrap();
    stats_repo
        .record_statistic(&make_statistic(job.id, inv2.id))
        .unwrap();
    assert_eq!(stats_repo.get_all_statistics().unwrap().len(), 2);

    stats_repo.delete_all_statistics().unwrap();
    assert_eq!(stats_repo.get_all_statistics().unwrap().len(), 0);
}

#[test]
fn test_cascade_delete_on_job_delete() {
    let (job_repo, inv_repo, stats_repo) = setup();
    let job = create_test_job();
    job_repo.create_job(&job).unwrap();
    let inv = make_invocation(job.id);
    inv_repo.create_invocation(&inv).unwrap();

    stats_repo
        .record_statistic(&make_statistic(job.id, inv.id))
        .unwrap();
    assert_eq!(stats_repo.get_all_statistics().unwrap().len(), 1);

    job_repo.delete_job(&job.id).unwrap();
    assert_eq!(stats_repo.get_all_statistics().unwrap().len(), 0);
}

#[test]
fn test_statistic_without_speedup() {
    let (job_repo, inv_repo, stats_repo) = setup();
    let job = create_test_job();
    job_repo.create_job(&job).unwrap();
    let inv = make_invocation(job.id);
    inv_repo.create_invocation(&inv).unwrap();

    let mut stat = make_statistic(job.id, inv.id);
    stat.speedup = None;
    stats_repo.record_statistic(&stat).unwrap();

    let all = stats_repo.get_all_statistics().unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].speedup, None);
}

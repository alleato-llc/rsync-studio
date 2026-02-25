use std::sync::Arc;

use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::database::sqlite::Database;
use crate::models::backup::{BackupInvocation, InvocationStatus, InvocationTrigger};
use crate::repository::sqlite::invocation::SqliteInvocationRepository;
use crate::repository::sqlite::job::SqliteJobRepository;
use crate::repository::sqlite::settings::SqliteSettingsRepository;
use crate::repository::sqlite::snapshot::SqliteSnapshotRepository;
use crate::services::job_service::JobService;
use crate::services::retention_runner::run_history_retention;
use crate::services::settings_service::SettingsService;
fn make_invocation(job_id: Uuid, age_days: i64) -> BackupInvocation {
    BackupInvocation {
        id: Uuid::new_v4(),
        job_id,
        started_at: Utc::now() - Duration::days(age_days),
        finished_at: Some(Utc::now() - Duration::days(age_days)),
        status: InvocationStatus::Succeeded,
        bytes_transferred: 0,
        files_transferred: 0,
        total_files: 0,
        snapshot_path: None,
        command_executed: "rsync -a /src /dst".to_string(),
        exit_code: Some(0),
        trigger: InvocationTrigger::Manual,
        log_file_path: None,
    }
}

fn setup_services() -> (JobService, SettingsService, Database) {
    let db = Database::open(":memory:").expect("open in-memory db");
    let conn = db.conn();
    let jobs = Arc::new(SqliteJobRepository::new(conn.clone()));
    let invocations = Arc::new(SqliteInvocationRepository::new(conn.clone()));
    let snapshots = Arc::new(SqliteSnapshotRepository::new(conn.clone()));
    let settings_repo = Arc::new(SqliteSettingsRepository::new(conn));

    let job_service = JobService::new(jobs, invocations, snapshots);
    let settings_service = SettingsService::new(settings_repo);

    (job_service, settings_service, db)
}

#[test]
fn test_retention_runner_prunes_old_invocations() {
    let (job_service, settings_service, _db) = setup_services();

    // Create a job
    let job = crate::tests::test_helpers::create_test_job();
    let created = job_service.create_job(job).expect("create job");

    // Record an old invocation (100 days old)
    let old_inv = make_invocation(created.id, 100);
    job_service
        .record_invocation(&old_inv)
        .expect("record invocation");

    // Record a recent invocation (1 day old)
    let recent_inv = make_invocation(created.id, 1);
    job_service
        .record_invocation(&recent_inv)
        .expect("record invocation");

    // Default retention is 90 days, so the old one should be pruned
    let count = run_history_retention(&job_service, &settings_service);
    assert_eq!(count, 1);

    // Verify the old invocation was deleted
    let remaining = job_service
        .get_job_history(&created.id, 100)
        .expect("get history");
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].id, recent_inv.id);
}

#[test]
fn test_retention_runner_nothing_to_prune() {
    let (job_service, settings_service, _db) = setup_services();

    let job = crate::tests::test_helpers::create_test_job();
    let created = job_service.create_job(job).expect("create job");

    // Record a recent invocation
    let inv = make_invocation(created.id, 1);
    job_service.record_invocation(&inv).expect("record inv");

    let count = run_history_retention(&job_service, &settings_service);
    assert_eq!(count, 0);
}

#[test]
fn test_retention_runner_prunes_excess_per_job() {
    let (job_service, settings_service, _db) = setup_services();

    // Set low max_history_per_job
    settings_service
        .set_retention_settings(&crate::services::settings_service::RetentionSettings {
            max_log_age_days: 365,
            max_history_per_job: 2,
        })
        .expect("set retention");

    let job = crate::tests::test_helpers::create_test_job();
    let created = job_service.create_job(job).expect("create job");

    // Record 5 invocations
    for i in 0..5 {
        let inv = make_invocation(created.id, i);
        job_service.record_invocation(&inv).expect("record inv");
    }

    let count = run_history_retention(&job_service, &settings_service);
    assert_eq!(count, 3); // 5 - 2 = 3 pruned

    let remaining = job_service
        .get_job_history(&created.id, 100)
        .expect("get history");
    assert_eq!(remaining.len(), 2);
}

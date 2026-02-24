use crate::implementations::database::Database;
use crate::implementations::sqlite_job_repository::SqliteJobRepository;
use crate::tests::test_helpers::create_test_job;
use crate::traits::job_repository::JobRepository;
use uuid::Uuid;

fn setup() -> SqliteJobRepository {
    let db = Database::in_memory().unwrap();
    SqliteJobRepository::new(db.conn())
}

#[test]
fn test_create_and_get_job() {
    let repo = setup();
    let job = create_test_job();
    repo.create_job(&job).unwrap();

    let retrieved = repo.get_job(&job.id).unwrap();
    assert_eq!(retrieved.id, job.id);
    assert_eq!(retrieved.name, job.name);
    assert_eq!(retrieved.description, job.description);
    assert_eq!(retrieved.source, job.source);
    assert_eq!(retrieved.destination, job.destination);
    assert_eq!(retrieved.backup_mode, job.backup_mode);
    assert_eq!(retrieved.enabled, job.enabled);
}

#[test]
fn test_list_jobs() {
    let repo = setup();
    let job1 = create_test_job();
    let mut job2 = create_test_job();
    job2.name = "Another Job".to_string();

    repo.create_job(&job1).unwrap();
    repo.create_job(&job2).unwrap();

    let jobs = repo.list_jobs().unwrap();
    assert_eq!(jobs.len(), 2);
}

#[test]
fn test_update_job() {
    let repo = setup();
    let mut job = create_test_job();
    repo.create_job(&job).unwrap();

    job.name = "Updated Name".to_string();
    job.enabled = false;
    repo.update_job(&job).unwrap();

    let retrieved = repo.get_job(&job.id).unwrap();
    assert_eq!(retrieved.name, "Updated Name");
    assert!(!retrieved.enabled);
}

#[test]
fn test_delete_job() {
    let repo = setup();
    let job = create_test_job();
    repo.create_job(&job).unwrap();
    repo.delete_job(&job.id).unwrap();

    let result = repo.get_job(&job.id);
    assert!(result.is_err());
}

#[test]
fn test_get_nonexistent_job() {
    let repo = setup();
    let result = repo.get_job(&Uuid::new_v4());
    assert!(result.is_err());
}

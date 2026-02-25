use std::process::Command;

use uuid::Uuid;

use crate::services::running_jobs::RunningJobs;

fn spawn_sleep_child() -> std::process::Child {
    Command::new("sleep")
        .arg("60")
        .spawn()
        .expect("failed to spawn sleep")
}

#[test]
fn test_insert_and_is_running() {
    let rj = RunningJobs::new();
    let id = Uuid::new_v4();
    let child = spawn_sleep_child();

    assert!(!rj.is_running(&id));

    let _arc = rj.insert(id, child);
    assert!(rj.is_running(&id));

    // Clean up
    rj.cancel(&id);
}

#[test]
fn test_cancel_returns_true_for_running_job() {
    let rj = RunningJobs::new();
    let id = Uuid::new_v4();
    let child = spawn_sleep_child();
    rj.insert(id, child);

    assert!(rj.cancel(&id));
}

#[test]
fn test_cancel_returns_false_for_unknown_job() {
    let rj = RunningJobs::new();
    assert!(!rj.cancel(&Uuid::new_v4()));
}

#[test]
fn test_remove() {
    let rj = RunningJobs::new();
    let id = Uuid::new_v4();
    let child = spawn_sleep_child();
    rj.insert(id, child);

    assert!(rj.is_running(&id));
    let removed = rj.remove(&id);
    assert!(removed.is_some());
    assert!(!rj.is_running(&id));

    // Clean up: kill the child
    if let Some(arc) = removed {
        if let Ok(mut c) = arc.lock() {
            let _ = c.kill();
            let _ = c.wait();
        }
    }
}

#[test]
fn test_running_job_ids() {
    let rj = RunningJobs::new();
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();

    assert!(rj.running_job_ids().is_empty());

    let child1 = spawn_sleep_child();
    let child2 = spawn_sleep_child();
    rj.insert(id1, child1);
    rj.insert(id2, child2);

    let mut ids = rj.running_job_ids();
    ids.sort();
    let mut expected = vec![id1, id2];
    expected.sort();
    assert_eq!(ids, expected);

    // Clean up
    rj.cancel(&id1);
    rj.cancel(&id2);
}

#[test]
fn test_remove_nonexistent_returns_none() {
    let rj = RunningJobs::new();
    assert!(rj.remove(&Uuid::new_v4()).is_none());
}

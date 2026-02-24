use std::rc::Rc;

use chrono::Utc;
use uuid::Uuid;

use crate::models::job::{
    BackupMode, JobDefinition, RetentionPolicy, RsyncOptions, StorageLocation,
};
use crate::tests::test_file_system::TestFileSystem;
use crate::tests::test_rsync_client::TestRsyncClient;

pub fn create_test_job() -> JobDefinition {
    create_mirror_job("/src/", "/dst/")
}

pub fn create_mirror_job(source: &str, dest: &str) -> JobDefinition {
    let now = Utc::now();
    JobDefinition {
        id: Uuid::new_v4(),
        name: "Test Mirror Job".to_string(),
        description: Some("A test mirror backup job".to_string()),
        source: StorageLocation::Local {
            path: source.to_string(),
        },
        destination: StorageLocation::Local {
            path: dest.to_string(),
        },
        backup_mode: BackupMode::Mirror,
        options: RsyncOptions {
            archive: true,
            delete: true,
            ..RsyncOptions::default()
        },
        ssh_config: None,
        schedule: None,
        enabled: true,
        created_at: now,
        updated_at: now,
    }
}

pub fn create_versioned_job(source: &str, dest: &str, backup_dir: &str) -> JobDefinition {
    let now = Utc::now();
    JobDefinition {
        id: Uuid::new_v4(),
        name: "Test Versioned Job".to_string(),
        description: Some("A test versioned backup job".to_string()),
        source: StorageLocation::Local {
            path: source.to_string(),
        },
        destination: StorageLocation::Local {
            path: dest.to_string(),
        },
        backup_mode: BackupMode::Versioned {
            backup_dir: backup_dir.to_string(),
        },
        options: RsyncOptions::default(),
        ssh_config: None,
        schedule: None,
        enabled: true,
        created_at: now,
        updated_at: now,
    }
}

pub fn create_snapshot_job(
    source: &str,
    dest: &str,
    retention: RetentionPolicy,
) -> JobDefinition {
    let now = Utc::now();
    JobDefinition {
        id: Uuid::new_v4(),
        name: "Test Snapshot Job".to_string(),
        description: Some("A test snapshot backup job".to_string()),
        source: StorageLocation::Local {
            path: source.to_string(),
        },
        destination: StorageLocation::Local {
            path: dest.to_string(),
        },
        backup_mode: BackupMode::Snapshot {
            retention_policy: retention,
        },
        options: RsyncOptions::default(),
        ssh_config: None,
        schedule: None,
        enabled: true,
        created_at: now,
        updated_at: now,
    }
}

pub fn setup_test_env() -> (Rc<TestFileSystem>, TestRsyncClient) {
    let fs = Rc::new(
        TestFileSystem::new()
            .with_file("/src/file1.txt", "Hello")
            .with_file("/src/file2.txt", "World")
            .with_file("/src/subdir/file3.txt", "Nested content")
            .with_dir("/dst"),
    );
    let client = TestRsyncClient::new(fs.clone());
    (fs, client)
}

use std::sync::Arc;

use chrono::Utc;

use crate::database::sqlite::Database;
use crate::models::job::{BackupMode, JobDefinition, RsyncOptions, StorageLocation};
use crate::repository::sqlite::settings::SqliteSettingsRepository;
use crate::services::settings_service::{apply_dry_mode_settings, DryModeSettings, SettingsService};

fn setup() -> SettingsService {
    let db = Database::in_memory().unwrap();
    let repo = Arc::new(SqliteSettingsRepository::new(db.conn()));
    SettingsService::new(repo)
}

fn make_job() -> JobDefinition {
    let now = Utc::now();
    JobDefinition {
        id: uuid::Uuid::new_v4(),
        name: "test".to_string(),
        description: None,
        source: StorageLocation::Local {
            path: "/src".to_string(),
        },
        destination: StorageLocation::Local {
            path: "/dst".to_string(),
        },
        options: RsyncOptions::default(),
        ssh_config: None,
        schedule: None,
        backup_mode: BackupMode::Mirror,
        enabled: true,
        created_at: now,
        updated_at: now,
    }
}

#[test]
fn get_dry_mode_settings_defaults_to_false() {
    let svc = setup();
    let settings = svc.get_dry_mode_settings().unwrap();
    assert!(!settings.itemize_changes);
    assert!(!settings.checksum);
}

#[test]
fn set_and_get_dry_mode_settings() {
    let svc = setup();
    let settings = DryModeSettings {
        itemize_changes: true,
        checksum: true,
    };
    svc.set_dry_mode_settings(&settings).unwrap();
    let result = svc.get_dry_mode_settings().unwrap();
    assert_eq!(result, settings);
}

#[test]
fn apply_dry_mode_settings_adds_itemize_changes() {
    let mut job = make_job();
    let settings = DryModeSettings {
        itemize_changes: true,
        checksum: false,
    };
    apply_dry_mode_settings(&mut job, &settings);
    assert!(job.options.custom_args.contains(&"--itemize-changes".to_string()));
    assert!(!job.options.custom_args.contains(&"--checksum".to_string()));
}

#[test]
fn apply_dry_mode_settings_adds_checksum() {
    let mut job = make_job();
    let settings = DryModeSettings {
        itemize_changes: false,
        checksum: true,
    };
    apply_dry_mode_settings(&mut job, &settings);
    assert!(!job.options.custom_args.contains(&"--itemize-changes".to_string()));
    assert!(job.options.custom_args.contains(&"--checksum".to_string()));
}

#[test]
fn apply_dry_mode_settings_does_not_duplicate_args() {
    let mut job = make_job();
    job.options
        .custom_args
        .push("--itemize-changes".to_string());
    job.options.custom_args.push("--checksum".to_string());

    let settings = DryModeSettings {
        itemize_changes: true,
        checksum: true,
    };
    apply_dry_mode_settings(&mut job, &settings);

    let itemize_count = job
        .options
        .custom_args
        .iter()
        .filter(|a| *a == "--itemize-changes")
        .count();
    let checksum_count = job
        .options
        .custom_args
        .iter()
        .filter(|a| *a == "--checksum")
        .count();
    assert_eq!(itemize_count, 1);
    assert_eq!(checksum_count, 1);
}

#[test]
fn apply_dry_mode_settings_both_enabled() {
    let mut job = make_job();
    let settings = DryModeSettings {
        itemize_changes: true,
        checksum: true,
    };
    apply_dry_mode_settings(&mut job, &settings);
    assert!(job.options.custom_args.contains(&"--itemize-changes".to_string()));
    assert!(job.options.custom_args.contains(&"--checksum".to_string()));
}

#[test]
fn apply_dry_mode_settings_neither_enabled() {
    let mut job = make_job();
    let settings = DryModeSettings {
        itemize_changes: false,
        checksum: false,
    };
    apply_dry_mode_settings(&mut job, &settings);
    assert!(job.options.custom_args.is_empty());
}

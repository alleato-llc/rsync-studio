use std::sync::Arc;

use chrono::Utc;

use crate::database::sqlite::Database;
use crate::models::job::{BackupMode, JobDefinition, RsyncOptions, StorageLocation, TransferConfig};
use crate::repository::sqlite::settings::SqliteSettingsRepository;
use crate::models::settings::DryModeSettings;
use crate::services::settings_service::{apply_dry_mode_settings, SettingsService};

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
        transfer: TransferConfig {
            source: StorageLocation::Local {
                path: "/src".to_string(),
            },
            destination: StorageLocation::Local {
                path: "/dst".to_string(),
            },
            backup_mode: BackupMode::Mirror,
        },
        options: RsyncOptions::default(),
        ssh_config: None,
        schedule: None,
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
    assert!(job.options.output.itemize_changes);
    assert!(!job.options.file_handling.checksum);
}

#[test]
fn apply_dry_mode_settings_adds_checksum() {
    let mut job = make_job();
    let settings = DryModeSettings {
        itemize_changes: false,
        checksum: true,
    };
    apply_dry_mode_settings(&mut job, &settings);
    assert!(!job.options.output.itemize_changes);
    assert!(job.options.file_handling.checksum);
}

#[test]
fn apply_dry_mode_settings_idempotent() {
    let mut job = make_job();
    job.options.output.itemize_changes = true;
    job.options.file_handling.checksum = true;

    let settings = DryModeSettings {
        itemize_changes: true,
        checksum: true,
    };
    apply_dry_mode_settings(&mut job, &settings);

    // Should remain true, not duplicate anything
    assert!(job.options.output.itemize_changes);
    assert!(job.options.file_handling.checksum);
}

#[test]
fn apply_dry_mode_settings_both_enabled() {
    let mut job = make_job();
    let settings = DryModeSettings {
        itemize_changes: true,
        checksum: true,
    };
    apply_dry_mode_settings(&mut job, &settings);
    assert!(job.options.output.itemize_changes);
    assert!(job.options.file_handling.checksum);
}

#[test]
fn apply_dry_mode_settings_neither_enabled() {
    let mut job = make_job();
    let settings = DryModeSettings {
        itemize_changes: false,
        checksum: false,
    };
    apply_dry_mode_settings(&mut job, &settings);
    assert!(!job.options.output.itemize_changes);
    assert!(!job.options.file_handling.checksum);
}

#[test]
fn show_file_handling_options_defaults_false() {
    let svc = setup();
    assert!(!svc.get_show_file_handling_options().unwrap());
}

#[test]
fn set_and_get_show_file_handling_options() {
    let svc = setup();
    svc.set_show_file_handling_options(true).unwrap();
    assert!(svc.get_show_file_handling_options().unwrap());
}

#[test]
fn show_metadata_options_defaults_false() {
    let svc = setup();
    assert!(!svc.get_show_metadata_options().unwrap());
}

#[test]
fn set_and_get_show_metadata_options() {
    let svc = setup();
    svc.set_show_metadata_options(true).unwrap();
    assert!(svc.get_show_metadata_options().unwrap());
}

#[test]
fn show_output_options_defaults_false() {
    let svc = setup();
    assert!(!svc.get_show_output_options().unwrap());
}

#[test]
fn set_and_get_show_output_options() {
    let svc = setup();
    svc.set_show_output_options(true).unwrap();
    assert!(svc.get_show_output_options().unwrap());
}

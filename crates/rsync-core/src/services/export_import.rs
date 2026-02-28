use chrono::Utc;
use uuid::Uuid;

use crate::models::job::{ExportData, JobDefinition};

const EXPORT_VERSION: u32 = 1;

/// Export a list of jobs to a JSON string.
pub fn export_jobs(jobs: Vec<JobDefinition>) -> Result<String, String> {
    let data = ExportData {
        version: EXPORT_VERSION,
        exported_at: Utc::now(),
        jobs,
    };
    serde_json::to_string_pretty(&data).map_err(|e| format!("Serialization error: {}", e))
}

/// Import jobs from a JSON string. Regenerates UUIDs and timestamps so imports
/// never collide with existing jobs.
pub fn import_jobs(json: &str) -> Result<Vec<JobDefinition>, String> {
    let data: ExportData =
        serde_json::from_str(json).map_err(|e| format!("Invalid export file: {}", e))?;

    if data.version > EXPORT_VERSION {
        return Err(format!(
            "Unsupported export version {} (max supported: {})",
            data.version, EXPORT_VERSION
        ));
    }

    if data.jobs.is_empty() {
        return Err("Export file contains no jobs".to_string());
    }

    let now = Utc::now();
    let jobs = data
        .jobs
        .into_iter()
        .map(|mut job| {
            job.id = Uuid::new_v4();
            job.created_at = now;
            job.updated_at = now;
            job
        })
        .collect();

    Ok(jobs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime;
    use crate::models::job::*;

    fn sample_job(name: &str) -> JobDefinition {
        JobDefinition {
            id: Uuid::new_v4(),
            name: name.to_string(),
            description: Some("test job".to_string()),
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
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn export_produces_valid_json() {
        let jobs = vec![sample_job("Job A"), sample_job("Job B")];
        let json = export_jobs(jobs).unwrap();
        let parsed: ExportData = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.version, 1);
        assert_eq!(parsed.jobs.len(), 2);
        assert_eq!(parsed.jobs[0].name, "Job A");
        assert_eq!(parsed.jobs[1].name, "Job B");
    }

    #[test]
    fn import_regenerates_uuids() {
        let original_id = Uuid::new_v4();
        let mut job = sample_job("Test");
        job.id = original_id;
        let json = export_jobs(vec![job]).unwrap();

        let imported = import_jobs(&json).unwrap();
        assert_eq!(imported.len(), 1);
        assert_ne!(imported[0].id, original_id);
    }

    #[test]
    fn import_resets_timestamps() {
        let old_time = DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let mut job = sample_job("Test");
        job.created_at = old_time;
        job.updated_at = old_time;
        let json = export_jobs(vec![job]).unwrap();

        let imported = import_jobs(&json).unwrap();
        assert!(imported[0].created_at > old_time);
        assert!(imported[0].updated_at > old_time);
    }

    #[test]
    fn import_preserves_job_data() {
        let mut job = sample_job("My Backup");
        job.options.core_transfer.compress = true;
        job.options.file_handling.delete = true;
        job.options.advanced.exclude_patterns = vec!["*.log".to_string()];
        let json = export_jobs(vec![job]).unwrap();

        let imported = import_jobs(&json).unwrap();
        assert_eq!(imported[0].name, "My Backup");
        assert!(imported[0].options.core_transfer.compress);
        assert!(imported[0].options.file_handling.delete);
        assert_eq!(imported[0].options.advanced.exclude_patterns, vec!["*.log"]);
    }

    #[test]
    fn import_rejects_invalid_json() {
        let result = import_jobs("not json at all");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid export file"));
    }

    #[test]
    fn import_rejects_future_version() {
        let data = ExportData {
            version: 999,
            exported_at: Utc::now(),
            jobs: vec![sample_job("Test")],
        };
        let json = serde_json::to_string(&data).unwrap();
        let result = import_jobs(&json);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unsupported export version"));
    }

    #[test]
    fn import_rejects_empty_jobs() {
        let data = ExportData {
            version: 1,
            exported_at: Utc::now(),
            jobs: vec![],
        };
        let json = serde_json::to_string(&data).unwrap();
        let result = import_jobs(&json);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("no jobs"));
    }

    #[test]
    fn roundtrip_with_ssh_config() {
        let mut job = sample_job("SSH Job");
        job.transfer.destination = StorageLocation::RemoteSsh {
            user: "admin".to_string(),
            host: "server.local".to_string(),
            port: 2222,
            path: "/backup".to_string(),
            identity_file: Some("/home/user/.ssh/id_rsa".to_string()),
        };
        job.ssh_config = Some(SshConfig {
            port: 2222,
            identity_file: Some("/home/user/.ssh/id_rsa".to_string()),
            strict_host_key_checking: false,
            custom_ssh_command: None,
        });
        let json = export_jobs(vec![job.clone()]).unwrap();
        let imported = import_jobs(&json).unwrap();

        assert_eq!(imported[0].transfer.destination, job.transfer.destination);
        assert_eq!(imported[0].ssh_config, job.ssh_config);
    }

    #[test]
    fn roundtrip_with_schedule() {
        use crate::models::schedule::{ScheduleConfig, ScheduleType};

        let mut job = sample_job("Scheduled Job");
        job.schedule = Some(ScheduleConfig {
            schedule_type: ScheduleType::Cron {
                expression: "0 9 * * *".to_string(),
            },
            enabled: true,
        });
        let json = export_jobs(vec![job.clone()]).unwrap();
        let imported = import_jobs(&json).unwrap();
        assert_eq!(imported[0].schedule, job.schedule);
    }

    #[test]
    fn import_multiple_jobs_all_get_unique_ids() {
        let jobs = vec![
            sample_job("A"),
            sample_job("B"),
            sample_job("C"),
        ];
        let json = export_jobs(jobs).unwrap();
        let imported = import_jobs(&json).unwrap();

        assert_eq!(imported.len(), 3);
        let ids: Vec<Uuid> = imported.iter().map(|j| j.id).collect();
        // All unique
        assert_ne!(ids[0], ids[1]);
        assert_ne!(ids[1], ids[2]);
        assert_ne!(ids[0], ids[2]);
    }
}

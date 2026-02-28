use crate::models::settings::HistoryRetentionConfig;
use crate::services::history_retention;
use crate::services::job_service::JobService;
use crate::services::settings_service::SettingsService;

/// Run history retention, pruning old invocations and their log files.
///
/// Returns the number of invocations pruned.
pub fn run_history_retention(
    job_service: &JobService,
    settings_service: &SettingsService,
) -> usize {
    let retention = match settings_service.get_retention_settings() {
        Ok(r) => r,
        Err(e) => {
            log::error!("Retention: failed to load settings: {}", e);
            return 0;
        }
    };

    let config = HistoryRetentionConfig {
        max_age_days: retention.max_log_age_days,
        max_per_job: retention.max_history_per_job,
    };

    let all_invocations = match job_service.list_all_invocations() {
        Ok(inv) => inv,
        Err(e) => {
            log::error!("Retention: failed to list invocations: {}", e);
            return 0;
        }
    };

    let to_prune = history_retention::compute_invocations_to_prune(&all_invocations, &config);
    let count = to_prune.len();

    for (inv_id, log_path) in &to_prune {
        // Delete log file if it exists
        if let Some(path) = log_path {
            if std::path::Path::new(path).exists() {
                if let Err(e) = std::fs::remove_file(path) {
                    log::error!("Retention: failed to delete log file {}: {}", path, e);
                }
            }
        }
        // Delete invocation from DB
        if let Err(e) = job_service.delete_invocation(inv_id) {
            log::error!("Retention: failed to delete invocation {}: {}", inv_id, e);
        }
    }

    if count > 0 {
        log::info!("Retention: pruned {} invocations", count);
    }

    count
}

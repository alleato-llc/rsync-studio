use std::io::{BufRead, BufReader};
use std::sync::Arc;

use serde::Serialize;
use tauri::State;
use uuid::Uuid;

use rsync_core::rsync_client::process_rsync_client::ProcessRsyncClient;
use rsync_core::file_system::real_file_system::RealFileSystem;
use rsync_core::models::backup::{BackupInvocation, InvocationTrigger, SnapshotRecord};
use rsync_core::models::job::JobDefinition;
use rsync_core::models::statistics::AggregatedStats;
use rsync_core::models::validation::PreflightResult;
use rsync_core::services::command_explainer::{self, CommandExplanation};
use rsync_core::services::command_parser;
use rsync_core::services::export_import;
use rsync_core::services::log_scrubber::{self, ScrubApplyResult, ScrubScanResult};
use rsync_core::services::preflight;
use rsync_core::services::settings_service::RetentionSettings;

use crate::execution::TauriEventHandler;
use crate::state::AppState;

#[tauri::command]
pub fn list_jobs(state: State<'_, AppState>) -> Result<Vec<JobDefinition>, String> {
    state.job_service.list_jobs().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_job(id: String, state: State<'_, AppState>) -> Result<JobDefinition, String> {
    let uuid = id
        .parse::<Uuid>()
        .map_err(|e| format!("Invalid job ID: {e}"))?;
    state.job_service.get_job(&uuid).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_job(job: JobDefinition, state: State<'_, AppState>) -> Result<JobDefinition, String> {
    state
        .job_service
        .create_job(job)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_job(job: JobDefinition, state: State<'_, AppState>) -> Result<JobDefinition, String> {
    state
        .job_service
        .update_job(job)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_job(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let uuid = id
        .parse::<Uuid>()
        .map_err(|e| format!("Invalid job ID: {e}"))?;
    state
        .job_service
        .delete_job(&uuid)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_job_history(
    job_id: String,
    limit: usize,
    state: State<'_, AppState>,
) -> Result<Vec<BackupInvocation>, String> {
    let uuid = job_id
        .parse::<Uuid>()
        .map_err(|e| format!("Invalid job ID: {e}"))?;
    state
        .job_service
        .get_job_history(&uuid, limit)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn execute_job(
    job_id: String,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let job_uuid = job_id
        .parse::<Uuid>()
        .map_err(|e| format!("Invalid job ID: {e}"))?;

    let job = state
        .job_service
        .get_job(&job_uuid)
        .map_err(|e| e.to_string())?;

    let handler = Arc::new(TauriEventHandler::new(app));
    state
        .job_executor
        .execute(&job, InvocationTrigger::Manual, handler)
        .map(|id| id.to_string())
}

#[tauri::command]
pub fn execute_job_dry_run(
    job_id: String,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let job_uuid = job_id
        .parse::<Uuid>()
        .map_err(|e| format!("Invalid job ID: {e}"))?;

    let mut job = state
        .job_service
        .get_job(&job_uuid)
        .map_err(|e| e.to_string())?;

    job.options.dry_run = true;

    let handler = Arc::new(TauriEventHandler::new(app));
    state
        .job_executor
        .execute(&job, InvocationTrigger::Manual, handler)
        .map(|id| id.to_string())
}

#[tauri::command]
pub fn cancel_job(job_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let uuid = job_id
        .parse::<Uuid>()
        .map_err(|e| format!("Invalid job ID: {e}"))?;
    if state.job_executor.cancel(&uuid) {
        Ok(())
    } else {
        Err("Job is not running".to_string())
    }
}

#[tauri::command]
pub fn get_running_jobs(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    Ok(state
        .job_executor
        .running_job_ids()
        .iter()
        .map(|id| id.to_string())
        .collect())
}

#[tauri::command]
pub fn list_snapshots(
    job_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<SnapshotRecord>, String> {
    let uuid = job_id
        .parse::<Uuid>()
        .map_err(|e| format!("Invalid job ID: {e}"))?;
    state
        .job_service
        .list_snapshots(&uuid)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_snapshot(
    snapshot_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let uuid = snapshot_id
        .parse::<Uuid>()
        .map_err(|e| format!("Invalid snapshot ID: {e}"))?;
    state
        .job_service
        .delete_snapshot(&uuid)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn explain_command(command: String) -> Result<CommandExplanation, String> {
    let parsed = command_parser::parse_rsync_command(&command)?;
    Ok(command_explainer::explain_command(&parsed))
}

#[tauri::command]
pub fn parse_command_to_job(command: String) -> Result<JobDefinition, String> {
    let parsed = command_parser::parse_rsync_command(&command)?;
    command_parser::to_job_definition(&parsed)
}

#[tauri::command]
pub fn export_jobs(state: State<'_, AppState>) -> Result<String, String> {
    let jobs = state.job_service.list_jobs().map_err(|e| e.to_string())?;
    export_import::export_jobs(jobs)
}

#[tauri::command]
pub fn import_jobs(json: String, state: State<'_, AppState>) -> Result<usize, String> {
    let jobs = export_import::import_jobs(&json)?;
    let count = jobs.len();
    for job in jobs {
        state
            .job_service
            .create_job(job)
            .map_err(|e| e.to_string())?;
    }
    Ok(count)
}

#[tauri::command]
pub fn run_preflight(job_id: String, state: State<'_, AppState>) -> Result<PreflightResult, String> {
    let uuid = job_id
        .parse::<Uuid>()
        .map_err(|e| format!("Invalid job ID: {e}"))?;
    let job = state.job_service.get_job(&uuid).map_err(|e| e.to_string())?;

    let fs = RealFileSystem::new();
    let rsync = ProcessRsyncClient::new();
    Ok(preflight::run_preflight(&job, &fs, &rsync))
}

#[tauri::command]
pub fn get_statistics(state: State<'_, AppState>) -> Result<AggregatedStats, String> {
    state
        .statistics_service
        .get_aggregated()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_statistics_for_job(
    job_id: String,
    state: State<'_, AppState>,
) -> Result<AggregatedStats, String> {
    let uuid = job_id
        .parse::<Uuid>()
        .map_err(|e| format!("Invalid job ID: {e}"))?;
    state
        .statistics_service
        .get_aggregated_for_job(&uuid)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn export_statistics(state: State<'_, AppState>) -> Result<String, String> {
    state
        .statistics_service
        .export()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reset_statistics(state: State<'_, AppState>) -> Result<(), String> {
    state
        .statistics_service
        .reset()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reset_statistics_for_job(
    job_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let uuid = job_id
        .parse::<Uuid>()
        .map_err(|e| format!("Invalid job ID: {e}"))?;
    state
        .statistics_service
        .reset_for_job(&uuid)
        .map_err(|e| e.to_string())
}

// --- Settings commands ---

#[tauri::command]
pub fn get_setting(key: String, state: State<'_, AppState>) -> Result<Option<String>, String> {
    state
        .settings_service
        .get_setting(&key)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_setting(key: String, value: String, state: State<'_, AppState>) -> Result<(), String> {
    state
        .settings_service
        .set_setting(&key, &value)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_log_directory(state: State<'_, AppState>) -> Result<String, String> {
    state
        .settings_service
        .get_log_directory()
        .map(|opt| opt.unwrap_or_else(|| state.job_executor.default_log_dir().to_string()))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_log_directory(path: String, state: State<'_, AppState>) -> Result<(), String> {
    state
        .settings_service
        .set_log_directory(&path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_retention_settings(state: State<'_, AppState>) -> Result<RetentionSettings, String> {
    state
        .settings_service
        .get_retention_settings()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_retention_settings(
    settings: RetentionSettings,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .settings_service
        .set_retention_settings(&settings)
        .map_err(|e| e.to_string())
}

// --- Trailing slash setting ---

#[tauri::command]
pub fn get_auto_trailing_slash(state: State<'_, AppState>) -> Result<bool, String> {
    state
        .settings_service
        .get_auto_trailing_slash()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_auto_trailing_slash(enabled: bool, state: State<'_, AppState>) -> Result<(), String> {
    state
        .settings_service
        .set_auto_trailing_slash(enabled)
        .map_err(|e| e.to_string())
}

// --- Delete history commands ---

#[tauri::command]
pub fn delete_invocation(
    invocation_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let uuid = invocation_id
        .parse::<Uuid>()
        .map_err(|e| format!("Invalid invocation ID: {e}"))?;

    // Fetch invocation to get log file path before deleting
    let inv = state
        .job_service
        .get_invocation(&uuid)
        .map_err(|e| e.to_string())?;

    // Delete log file if it exists
    if let Some(ref path) = inv.log_file_path {
        if std::path::Path::new(path).exists() {
            let _ = std::fs::remove_file(path);
        }
    }

    state
        .job_service
        .delete_invocation(&uuid)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_invocations_for_job(
    job_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let uuid = job_id
        .parse::<Uuid>()
        .map_err(|e| format!("Invalid job ID: {e}"))?;

    // Fetch all invocations for the job to delete their log files
    let invocations = state
        .job_service
        .get_job_history(&uuid, usize::MAX)
        .map_err(|e| e.to_string())?;

    for inv in &invocations {
        if let Some(ref path) = inv.log_file_path {
            if std::path::Path::new(path).exists() {
                let _ = std::fs::remove_file(path);
            }
        }
    }

    state
        .job_service
        .delete_invocations_for_job(&uuid)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn count_invocations(state: State<'_, AppState>) -> Result<usize, String> {
    state
        .job_service
        .list_all_invocations()
        .map(|inv| inv.len())
        .map_err(|e| e.to_string())
}

// --- Log scrubber commands ---

#[tauri::command]
pub fn scrub_scan_logs(
    pattern: String,
    state: State<'_, AppState>,
) -> Result<Vec<ScrubScanResult>, String> {
    let log_dir = state
        .settings_service
        .get_log_directory()
        .map(|opt| opt.unwrap_or_else(|| state.job_executor.default_log_dir().to_string()))
        .map_err(|e| e.to_string())?;

    log_scrubber::scrub_scan(&log_dir, &pattern).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn scrub_apply_logs(
    pattern: String,
    file_paths: Vec<String>,
) -> Result<Vec<ScrubApplyResult>, String> {
    log_scrubber::scrub_apply(&pattern, &file_paths).map_err(|e| e.to_string())
}

// --- Log file commands ---

#[tauri::command]
pub fn read_log_file(path: String) -> Result<String, String> {
    if !std::path::Path::new(&path).exists() {
        return Err("Log file not found. It may have been deleted or moved.".to_string());
    }
    std::fs::read_to_string(&path).map_err(|e| format!("Failed to read log file: {e}"))
}

#[derive(Debug, Clone, Serialize)]
pub struct LogFileLine {
    pub text: String,
    pub is_stderr: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct LogFileChunk {
    pub lines: Vec<LogFileLine>,
    pub total_lines: usize,
}

#[tauri::command]
pub fn read_log_file_lines(path: String, offset: usize, limit: usize) -> Result<LogFileChunk, String> {
    let file_path = std::path::Path::new(&path);
    if !file_path.exists() {
        return Err("Log file not found. It may have been deleted or moved.".to_string());
    }

    let file = std::fs::File::open(file_path)
        .map_err(|e| format!("Failed to open log file: {e}"))?;
    let reader = BufReader::new(file);

    let all_lines: Vec<String> = reader
        .lines()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to read log file: {e}"))?;

    let total_lines = all_lines.len();

    let lines: Vec<LogFileLine> = all_lines
        .into_iter()
        .skip(offset)
        .take(limit)
        .map(|raw| {
            let is_stderr = raw.contains("[STDERR]");
            let text = raw
                .splitn(2, "] ")
                .nth(1)
                .unwrap_or(&raw)
                .to_string();
            LogFileLine { text, is_stderr }
        })
        .collect();

    Ok(LogFileChunk { lines, total_lines })
}

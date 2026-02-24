use std::sync::Arc;

use tauri::State;
use uuid::Uuid;

use rsync_core::implementations::process_rsync_client::ProcessRsyncClient;
use rsync_core::implementations::real_file_system::RealFileSystem;
use rsync_core::models::backup::{BackupInvocation, InvocationTrigger, SnapshotRecord};
use rsync_core::models::job::JobDefinition;
use rsync_core::models::statistics::AggregatedStats;
use rsync_core::models::validation::PreflightResult;
use rsync_core::services::command_explainer::{self, CommandExplanation};
use rsync_core::services::command_parser;
use rsync_core::services::export_import;
use rsync_core::services::preflight;

use crate::execution::run_job_internal;
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

    run_job_internal(
        &job,
        InvocationTrigger::Manual,
        &state.running_jobs,
        Arc::clone(&state.job_service),
        Arc::clone(&state.statistics_service),
        app,
    )
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

    run_job_internal(
        &job,
        InvocationTrigger::Manual,
        &state.running_jobs,
        Arc::clone(&state.job_service),
        Arc::clone(&state.statistics_service),
        app,
    )
}

#[tauri::command]
pub fn cancel_job(job_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let uuid = job_id
        .parse::<Uuid>()
        .map_err(|e| format!("Invalid job ID: {e}"))?;
    if state.running_jobs.cancel(&uuid) {
        Ok(())
    } else {
        Err("Job is not running".to_string())
    }
}

#[tauri::command]
pub fn get_running_jobs(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    Ok(state
        .running_jobs
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

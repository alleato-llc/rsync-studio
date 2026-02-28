use std::io::Write;
use std::sync::Arc;

use chrono::Utc;
use regex::Regex;
use uuid::Uuid;

use crate::models::backup::{
    BackupInvocation, ExecutionOutput, InvocationStatus, InvocationTrigger, SnapshotRecord,
    TransferStats,
};
use crate::models::job::{BackupMode, JobDefinition, JobStatus, StorageLocation};
use crate::models::progress::{JobStatusEvent, LogLine};
use crate::services::command_builder::build_rsync_args;
use crate::services::execution_handler::ExecutionEventHandler;
use crate::models::execution::event::ExecutionEvent;
use crate::services::job_runner::run_job;
use crate::services::job_service::JobService;
use crate::services::progress_parser::parse_summary_line;
use crate::services::snapshot_retention;
use crate::services::running_jobs::RunningJobs;
use crate::services::settings_service::SettingsService;
use crate::services::statistics_service::StatisticsService;

/// For snapshot-mode jobs, compute the destination subdir and link-dest path.
struct SnapshotContext {
    /// The full snapshot destination path (e.g., /backups/2025-06-15_140000)
    snapshot_path: String,
    /// The destination StorageLocation overridden with the snapshot subdir
    effective_destination: StorageLocation,
    /// Path to the previous snapshot for --link-dest, if any
    link_dest: Option<String>,
}

fn prepare_snapshot_context(
    job: &JobDefinition,
    job_service: &JobService,
) -> Result<Option<SnapshotContext>, String> {
    match &job.transfer.backup_mode {
        BackupMode::Snapshot { .. } => {}
        _ => return Ok(None),
    }

    let now = Utc::now();
    let dir_name = snapshot_retention::snapshot_dir_name(now);

    // Build snapshot path under the job's destination
    let base_path = match &job.transfer.destination {
        StorageLocation::Local { path } => path.clone(),
        StorageLocation::RemoteSsh { path, .. } => path.clone(),
        StorageLocation::RemoteRsync { path, .. } => path.clone(),
    };

    let base = base_path.trim_end_matches('/');
    let snapshot_path = format!("{}/{}", base, dir_name);

    // Override destination with snapshot subdir
    let effective_destination = match &job.transfer.destination {
        StorageLocation::Local { .. } => StorageLocation::Local {
            path: format!("{}/", snapshot_path),
        },
        StorageLocation::RemoteSsh {
            user,
            host,
            port,
            identity_file,
            ..
        } => StorageLocation::RemoteSsh {
            user: user.clone(),
            host: host.clone(),
            port: *port,
            path: format!("{}/", snapshot_path),
            identity_file: identity_file.clone(),
        },
        StorageLocation::RemoteRsync {
            host, module, ..
        } => StorageLocation::RemoteRsync {
            host: host.clone(),
            module: module.clone(),
            path: format!("{}/", snapshot_path),
        },
    };

    // Get the latest snapshot for --link-dest
    let link_dest = job_service
        .get_latest_snapshot(&job.id)
        .map_err(|e| e.to_string())?
        .map(|snap| snap.snapshot_path);

    Ok(Some(SnapshotContext {
        snapshot_path,
        effective_destination,
        link_dest,
    }))
}

/// Orchestrates job execution, including snapshot context, log writing,
/// statistics recording, and event emission through a pluggable handler.
pub struct JobExecutor {
    job_service: Arc<JobService>,
    statistics_service: Arc<StatisticsService>,
    settings_service: Arc<SettingsService>,
    running_jobs: Arc<RunningJobs>,
    default_log_dir: String,
}

impl JobExecutor {
    pub fn new(
        job_service: Arc<JobService>,
        statistics_service: Arc<StatisticsService>,
        settings_service: Arc<SettingsService>,
        running_jobs: Arc<RunningJobs>,
        default_log_dir: String,
    ) -> Self {
        Self {
            job_service,
            statistics_service,
            settings_service,
            running_jobs,
            default_log_dir,
        }
    }

    pub fn job_service(&self) -> &Arc<JobService> {
        &self.job_service
    }

    pub fn statistics_service(&self) -> &Arc<StatisticsService> {
        &self.statistics_service
    }

    pub fn settings_service(&self) -> &Arc<SettingsService> {
        &self.settings_service
    }

    pub fn running_jobs(&self) -> &Arc<RunningJobs> {
        &self.running_jobs
    }

    pub fn default_log_dir(&self) -> &str {
        &self.default_log_dir
    }

    /// Execute a job with the given trigger, emitting events through the handler.
    ///
    /// Returns the invocation ID on success.
    pub fn execute(
        &self,
        job: &JobDefinition,
        trigger: InvocationTrigger,
        handler: Arc<dyn ExecutionEventHandler>,
    ) -> Result<Uuid, String> {
        let job_uuid = job.id;

        // Reject if already running
        if self.running_jobs.is_running(&job_uuid) {
            return Err("Job is already running".to_string());
        }

        // Prepare snapshot context if applicable
        let snapshot_ctx = prepare_snapshot_context(job, &self.job_service)?;

        // Choose effective destination and link-dest
        let effective_dest = snapshot_ctx
            .as_ref()
            .map(|ctx| &ctx.effective_destination)
            .unwrap_or(&job.transfer.destination);
        let link_dest = snapshot_ctx
            .as_ref()
            .and_then(|ctx| ctx.link_dest.as_deref());

        // Read auto trailing slash setting
        let auto_trailing_slash = self
            .settings_service
            .get_auto_trailing_slash()
            .unwrap_or(true);

        // Build rsync args
        let args = build_rsync_args(
            &job.transfer.source,
            effective_dest,
            &job.options,
            job.ssh_config.as_ref(),
            link_dest,
            auto_trailing_slash,
        );

        let invocation_id = Uuid::new_v4();
        let command_str = format!("rsync {}", args.join(" "));
        let snapshot_path_for_record = snapshot_ctx.as_ref().map(|ctx| ctx.snapshot_path.clone());

        // Resolve log directory from settings, fallback to default
        let log_dir = self
            .settings_service
            .get_log_directory()
            .ok()
            .flatten()
            .unwrap_or_else(|| self.default_log_dir.clone());

        let dir = std::path::Path::new(&log_dir);
        if let Err(e) = std::fs::create_dir_all(dir) {
            log::error!("Failed to create log directory {}: {}", log_dir, e);
        }
        let log_file_path = format!("{}/{}.log", log_dir, invocation_id);

        // Create invocation record
        let invocation = BackupInvocation {
            id: invocation_id,
            job_id: job_uuid,
            started_at: Utc::now(),
            finished_at: None,
            status: InvocationStatus::Running,
            trigger: trigger.clone(),
            transfer_stats: TransferStats::default(),
            execution_output: ExecutionOutput {
                command_executed: command_str,
                exit_code: None,
                snapshot_path: snapshot_path_for_record.clone(),
                log_file_path: Some(log_file_path.clone()),
            },
        };

        self.job_service
            .record_invocation(&invocation)
            .map_err(|e| e.to_string())?;

        // Emit Running status
        handler.on_status_change(JobStatusEvent {
            job_id: job_uuid,
            invocation_id,
            status: JobStatus::Running,
            exit_code: None,
            error_message: None,
        });

        // Spawn rsync process
        let (child, rx) = run_job("rsync", &args, invocation_id).map_err(|e| e.to_string())?;

        // Store in running jobs
        let _child_arc = self.running_jobs.insert(job_uuid, child);

        // Capture snapshot info for the background thread
        let is_snapshot_mode = snapshot_ctx.is_some();
        let is_dry_run = job.options.core_transfer.dry_run;
        let link_dest_for_record = snapshot_ctx
            .as_ref()
            .and_then(|ctx| ctx.link_dest.clone());
        let invocation_started_at = invocation.started_at;

        // Clone Arcs for the background thread
        let running_jobs = Arc::clone(&self.running_jobs);
        let job_service = Arc::clone(&self.job_service);
        let statistics_service = Arc::clone(&self.statistics_service);

        let log_path_for_thread = log_file_path.clone();
        std::thread::spawn(move || {
            let mut last_bytes: u64 = 0;
            let mut last_files: u64 = 0;
            let mut last_total: u64 = 0;
            let mut last_speedup: Option<f64> = None;
            let mut summary_sent_bytes: Option<u64> = None;

            // Open log file for writing
            let mut log_writer = std::fs::File::create(&log_path_for_thread)
                .map(std::io::BufWriter::new)
                .ok();

            let speedup_re = Regex::new(r"speedup is ([\d.]+)").ok();

            while let Ok(event) = rx.recv() {
                match event {
                    ExecutionEvent::StdoutLine(line) => {
                        // Parse transfer summary ("sent X bytes  received Y bytes")
                        if let Some(summary) = parse_summary_line(&line) {
                            summary_sent_bytes = Some(summary.sent_bytes);
                        }

                        // Parse speedup from rsync summary line
                        if let Some(ref re) = speedup_re {
                            if let Some(caps) = re.captures(&line) {
                                if let Some(m) = caps.get(1) {
                                    if let Ok(val) = m.as_str().parse::<f64>() {
                                        last_speedup = Some(val);
                                    }
                                }
                            }
                        }

                        // Write to log file
                        if let Some(ref mut writer) = log_writer {
                            let _ = writeln!(
                                writer,
                                "[{}] {}",
                                Utc::now().format("%Y-%m-%d %H:%M:%S"),
                                line
                            );
                        }

                        handler.on_log_line(LogLine {
                            invocation_id,
                            timestamp: Utc::now(),
                            line,
                            is_stderr: false,
                        });
                    }
                    ExecutionEvent::StderrLine(line) => {
                        // Write to log file
                        if let Some(ref mut writer) = log_writer {
                            let _ = writeln!(
                                writer,
                                "[{}] STDERR: {}",
                                Utc::now().format("%Y-%m-%d %H:%M:%S"),
                                line
                            );
                        }

                        handler.on_log_line(LogLine {
                            invocation_id,
                            timestamp: Utc::now(),
                            line,
                            is_stderr: true,
                        });
                    }
                    ExecutionEvent::Progress(progress) => {
                        last_bytes = progress.bytes_transferred;
                        last_files = progress.files_transferred;
                        last_total = progress.files_total;
                        handler.on_progress(&progress);
                    }
                    ExecutionEvent::ItemizedChange(change) => {
                        handler.on_itemized_change(invocation_id, &change);
                    }
                    ExecutionEvent::Finished { .. } => {
                        // Handled below after loop
                    }
                }
            }

            // Flush and drop the log writer before completion
            if let Some(mut writer) = log_writer.take() {
                let _ = writer.flush();
            }

            // Receiver disconnected — reader threads are done.
            // Remove from running jobs and wait for exit code.
            let exit_code: Option<i32> = {
                if let Some(child_arc) = running_jobs.remove(&job_uuid) {
                    if let Ok(mut child) = child_arc.lock() {
                        child
                            .wait()
                            .ok()
                            .and_then(|s: std::process::ExitStatus| s.code())
                    } else {
                        None
                    }
                } else {
                    None
                }
            };

            // On Unix, killed processes return None from .code()
            let was_cancelled = exit_code.is_none();

            let (status, job_status) = if was_cancelled && exit_code != Some(0) {
                (InvocationStatus::Cancelled, JobStatus::Cancelled)
            } else if exit_code == Some(0) {
                (InvocationStatus::Succeeded, JobStatus::Completed)
            } else {
                (InvocationStatus::Failed, JobStatus::Failed)
            };

            // Update invocation record
            // Use total sent bytes from rsync summary when available (accurate total),
            // falling back to the last per-file progress value.
            let final_bytes = summary_sent_bytes.unwrap_or(last_bytes);
            let completed_invocation = BackupInvocation {
                id: invocation_id,
                job_id: job_uuid,
                started_at: invocation_started_at,
                finished_at: Some(Utc::now()),
                status: status.clone(),
                trigger,
                transfer_stats: TransferStats {
                    bytes_transferred: final_bytes,
                    files_transferred: last_files,
                    total_files: last_total,
                },
                execution_output: ExecutionOutput {
                    command_executed: String::new(),
                    exit_code,
                    snapshot_path: snapshot_path_for_record.clone(),
                    log_file_path: Some(log_path_for_thread),
                },
            };

            let _ = job_service.complete_invocation(&completed_invocation);

            // Record run statistics for successful non-dry-run completions
            if status == InvocationStatus::Succeeded && !is_dry_run {
                if let Err(e) =
                    statistics_service.record(job_uuid, &completed_invocation, last_speedup)
                {
                    log::error!("Failed to record run statistics: {}", e);
                }
            }

            // On success for snapshot-mode jobs: record snapshot and apply retention
            // Skip snapshot recording for dry-run executions
            if status == InvocationStatus::Succeeded && is_snapshot_mode && !is_dry_run {
                if let Some(ref snap_path) = snapshot_path_for_record {
                    let snapshot = SnapshotRecord {
                        id: Uuid::new_v4(),
                        job_id: job_uuid,
                        invocation_id,
                        snapshot_path: snap_path.clone(),
                        link_dest_path: link_dest_for_record,
                        created_at: Utc::now(),
                        size_bytes: last_bytes,
                        file_count: last_files,
                        is_latest: true,
                    };

                    if let Err(e) = job_service.record_snapshot(&snapshot) {
                        log::error!("Failed to record snapshot: {}", e);
                    }

                    // Apply retention policy — prune old snapshots from DB
                    match job_service.apply_retention_policy(&job_uuid) {
                        Ok(pruned_paths) => {
                            for path in pruned_paths {
                                log::info!("Retention: pruned snapshot {}", path);
                                // Attempt to remove the directory on disk
                                if let Err(e) = std::fs::remove_dir_all(&path) {
                                    log::error!(
                                        "Failed to remove pruned snapshot dir {}: {}",
                                        path,
                                        e
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to apply retention policy: {}", e);
                        }
                    }
                }
            }

            handler.on_status_change(JobStatusEvent {
                job_id: job_uuid,
                invocation_id,
                status: job_status,
                exit_code,
                error_message: if status == InvocationStatus::Failed {
                    Some(format!(
                        "rsync exited with code {}",
                        exit_code.unwrap_or(-1)
                    ))
                } else {
                    None
                },
            });
        });

        Ok(invocation_id)
    }

    /// Cancel a running job. Returns true if the job was found and killed.
    pub fn cancel(&self, job_id: &Uuid) -> bool {
        self.running_jobs.cancel(job_id)
    }

    /// Check if a job is currently running.
    pub fn is_running(&self, job_id: &Uuid) -> bool {
        self.running_jobs.is_running(job_id)
    }

    /// Get the IDs of all currently running jobs.
    pub fn running_job_ids(&self) -> Vec<Uuid> {
        self.running_jobs.running_job_ids()
    }
}

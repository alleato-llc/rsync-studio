use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;

use crate::models::backup::InvocationTrigger;
use crate::services::execution_handler::ExecutionEventHandler;
use crate::services::job_executor::JobExecutor;
use crate::services::job_service::JobService;
use crate::services::retention_runner;
use crate::services::scheduler;
use crate::services::settings_service::SettingsService;

pub struct SchedulerConfig {
    /// How often the scheduler checks for due jobs (in seconds).
    pub check_interval_secs: u64,
    /// How many scheduler cycles between retention checks.
    pub retention_check_every_n_cycles: u64,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            check_interval_secs: 300,           // 5 minutes
            retention_check_every_n_cycles: 12, // ~1 hour at 5min intervals
        }
    }
}

/// Handle returned from starting a scheduler, allowing it to be stopped.
pub struct SchedulerHandle {
    stop_sender: std::sync::mpsc::Sender<()>,
}

impl SchedulerHandle {
    /// Signal the scheduler to stop. Returns false if the scheduler already stopped.
    pub fn stop(&self) -> bool {
        self.stop_sender.send(()).is_ok()
    }
}

/// Trait for pluggable scheduler backends.
///
/// The in-process scheduler runs a loop in a background thread.
/// Future implementations could use systemd timers or crontab entries.
pub trait SchedulerBackend: Send + Sync {
    fn start(&self) -> SchedulerHandle;
}

/// In-process scheduler that runs a background thread checking for due jobs.
pub struct InProcessScheduler {
    config: SchedulerConfig,
    job_executor: Arc<JobExecutor>,
    job_service: Arc<JobService>,
    settings_service: Arc<SettingsService>,
    handler_factory: Arc<dyn Fn() -> Arc<dyn ExecutionEventHandler> + Send + Sync>,
    /// Optional callback emitted when a job is scheduled (e.g., Tauri event).
    on_job_scheduled: Option<Arc<dyn Fn(&uuid::Uuid) + Send + Sync>>,
}

impl InProcessScheduler {
    pub fn new(
        config: SchedulerConfig,
        job_executor: Arc<JobExecutor>,
        job_service: Arc<JobService>,
        settings_service: Arc<SettingsService>,
        handler_factory: Arc<dyn Fn() -> Arc<dyn ExecutionEventHandler> + Send + Sync>,
    ) -> Self {
        Self {
            config,
            job_executor,
            job_service,
            settings_service,
            handler_factory,
            on_job_scheduled: None,
        }
    }

    pub fn with_on_job_scheduled(
        mut self,
        callback: Arc<dyn Fn(&uuid::Uuid) + Send + Sync>,
    ) -> Self {
        self.on_job_scheduled = Some(callback);
        self
    }
}

impl SchedulerBackend for InProcessScheduler {
    fn start(&self) -> SchedulerHandle {
        let (stop_tx, stop_rx) = std::sync::mpsc::channel();

        let config_interval = self.config.check_interval_secs;
        let config_retention_n = self.config.retention_check_every_n_cycles;
        let job_executor = Arc::clone(&self.job_executor);
        let job_service = Arc::clone(&self.job_service);
        let settings_service = Arc::clone(&self.settings_service);
        let handler_factory = Arc::clone(&self.handler_factory);
        let on_job_scheduled = self.on_job_scheduled.clone();

        std::thread::spawn(move || {
            let mut cycle_count: u64 = 0;
            loop {
                // Sleep with interruptibility via stop channel
                match stop_rx.recv_timeout(Duration::from_secs(config_interval)) {
                    Ok(()) => break,                          // Stop signal received
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {} // Normal tick
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                }

                cycle_count += 1;

                // Periodically run history retention
                if cycle_count % config_retention_n == 0 {
                    retention_runner::run_history_retention(&job_service, &settings_service);
                }

                let jobs = match job_service.list_jobs() {
                    Ok(j) => j,
                    Err(e) => {
                        log::error!("Scheduler: failed to list jobs: {}", e);
                        continue;
                    }
                };

                let now = Utc::now();

                for job in &jobs {
                    // Skip disabled jobs or jobs without a schedule
                    if !job.enabled {
                        continue;
                    }
                    let schedule = match &job.schedule {
                        Some(s) if s.enabled => s,
                        _ => continue,
                    };

                    // Skip jobs that are currently running
                    if job_executor.is_running(&job.id) {
                        continue;
                    }

                    // Determine the last run time from history
                    let last_run = job_service
                        .get_job_history(&job.id, 1)
                        .ok()
                        .and_then(|h| h.first().map(|inv| inv.started_at));

                    if scheduler::is_job_due(schedule, last_run, now) {
                        log::info!(
                            "Scheduler: job '{}' ({}) is due, executing",
                            job.name,
                            job.id
                        );

                        if let Some(ref callback) = on_job_scheduled {
                            callback(&job.id);
                        }

                        let handler = handler_factory();
                        if let Err(e) =
                            job_executor.execute(job, InvocationTrigger::Scheduled, handler)
                        {
                            log::error!(
                                "Scheduler: failed to execute job '{}' ({}): {}",
                                job.name,
                                job.id,
                                e
                            );
                        }
                    }
                }
            }
        });

        SchedulerHandle {
            stop_sender: stop_tx,
        }
    }
}

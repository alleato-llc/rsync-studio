use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::backup::BackupInvocation;
use crate::models::statistics::{AggregatedStats, RunStatistic};
use crate::traits::statistics_repository::StatisticsRepository;

pub struct StatisticsService {
    stats: Arc<dyn StatisticsRepository>,
}

impl StatisticsService {
    pub fn new(stats: Arc<dyn StatisticsRepository>) -> Self {
        Self { stats }
    }

    /// Record a run statistic after a successful job completion.
    pub fn record(
        &self,
        job_id: Uuid,
        inv: &BackupInvocation,
        speedup: Option<f64>,
    ) -> Result<(), AppError> {
        let duration_secs = match inv.finished_at {
            Some(finished) => (finished - inv.started_at).num_milliseconds() as f64 / 1000.0,
            None => 0.0,
        };

        let stat = RunStatistic {
            id: Uuid::new_v4(),
            job_id,
            invocation_id: inv.id,
            recorded_at: Utc::now(),
            files_transferred: inv.files_transferred,
            bytes_transferred: inv.bytes_transferred,
            duration_secs,
            speedup,
        };

        self.stats.record_statistic(&stat)
    }

    pub fn get_aggregated(&self) -> Result<AggregatedStats, AppError> {
        let all = self.stats.get_all_statistics()?;
        Ok(aggregate(&all))
    }

    pub fn get_aggregated_for_job(&self, job_id: &Uuid) -> Result<AggregatedStats, AppError> {
        let stats = self.stats.get_statistics_for_job(job_id)?;
        Ok(aggregate(&stats))
    }

    pub fn export(&self) -> Result<String, AppError> {
        let all = self.stats.get_all_statistics()?;
        serde_json::to_string_pretty(&all)
            .map_err(|e| AppError::SerializationError(e.to_string()))
    }

    pub fn reset(&self) -> Result<(), AppError> {
        self.stats.delete_all_statistics()
    }

    pub fn reset_for_job(&self, job_id: &Uuid) -> Result<(), AppError> {
        self.stats.delete_statistics_for_job(job_id)
    }
}

fn aggregate(stats: &[RunStatistic]) -> AggregatedStats {
    let total_jobs_run = stats.len() as u64;
    let total_files_transferred: u64 = stats.iter().map(|s| s.files_transferred).sum();
    let total_bytes_transferred: u64 = stats.iter().map(|s| s.bytes_transferred).sum();
    let total_duration_secs: f64 = stats.iter().map(|s| s.duration_secs).sum();

    // Time saved: for each run with a speedup > 1, the time it would have taken
    // without rsync's delta-transfer is duration * speedup. So time saved = duration * (speedup - 1).
    let total_time_saved_secs: f64 = stats
        .iter()
        .filter_map(|s| {
            s.speedup.and_then(|sp| {
                if sp > 1.0 {
                    Some(s.duration_secs * (sp - 1.0))
                } else {
                    None
                }
            })
        })
        .sum();

    AggregatedStats {
        total_jobs_run,
        total_files_transferred,
        total_bytes_transferred,
        total_duration_secs,
        total_time_saved_secs,
    }
}

use std::sync::Arc;

use rsync_core::implementations::database::Database;
use rsync_core::services::job_service::JobService;
use rsync_core::services::statistics_service::StatisticsService;

use crate::execution::RunningJobs;

pub struct AppState {
    pub _database: Database,
    pub job_service: Arc<JobService>,
    pub statistics_service: Arc<StatisticsService>,
    pub running_jobs: RunningJobs,
}

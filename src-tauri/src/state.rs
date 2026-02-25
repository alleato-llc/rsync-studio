use std::sync::Arc;

use rsync_core::database::sqlite::Database;
use rsync_core::services::job_executor::JobExecutor;
use rsync_core::services::job_service::JobService;
use rsync_core::services::settings_service::SettingsService;
use rsync_core::services::statistics_service::StatisticsService;

pub struct AppState {
    pub _database: Database,
    pub job_service: Arc<JobService>,
    pub statistics_service: Arc<StatisticsService>,
    pub settings_service: Arc<SettingsService>,
    pub job_executor: Arc<JobExecutor>,
}

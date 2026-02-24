use uuid::Uuid;

use crate::error::AppError;
use crate::models::statistics::RunStatistic;

pub trait StatisticsRepository: Send + Sync {
    fn record_statistic(&self, stat: &RunStatistic) -> Result<(), AppError>;
    fn get_statistics_for_job(&self, job_id: &Uuid) -> Result<Vec<RunStatistic>, AppError>;
    fn get_all_statistics(&self) -> Result<Vec<RunStatistic>, AppError>;
    fn delete_statistics_for_job(&self, job_id: &Uuid) -> Result<(), AppError>;
    fn delete_all_statistics(&self) -> Result<(), AppError>;
}

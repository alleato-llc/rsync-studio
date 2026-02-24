use uuid::Uuid;

use crate::error::AppError;
use crate::models::job::JobDefinition;

pub trait JobRepository: Send + Sync {
    fn create_job(&self, job: &JobDefinition) -> Result<(), AppError>;
    fn get_job(&self, id: &Uuid) -> Result<JobDefinition, AppError>;
    fn list_jobs(&self) -> Result<Vec<JobDefinition>, AppError>;
    fn update_job(&self, job: &JobDefinition) -> Result<(), AppError>;
    fn delete_job(&self, id: &Uuid) -> Result<(), AppError>;
}

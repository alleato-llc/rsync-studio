use crate::error::AppError;

pub trait SettingsRepository: Send + Sync {
    fn get_setting(&self, key: &str) -> Result<Option<String>, AppError>;
    fn set_setting(&self, key: &str, value: &str) -> Result<(), AppError>;
    fn delete_setting(&self, key: &str) -> Result<(), AppError>;
}

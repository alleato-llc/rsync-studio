use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::models::job::JobDefinition;
use crate::repository::settings::SettingsRepository;

const KEY_LOG_DIRECTORY: &str = "log_directory";
const KEY_MAX_LOG_AGE_DAYS: &str = "max_log_age_days";
const KEY_MAX_HISTORY_PER_JOB: &str = "max_history_per_job";
const KEY_AUTO_TRAILING_SLASH: &str = "auto_trailing_slash";
const KEY_DRY_MODE_ITEMIZE_CHANGES: &str = "dry_mode_itemize_changes";
const KEY_DRY_MODE_CHECKSUM: &str = "dry_mode_checksum";

const DEFAULT_AUTO_TRAILING_SLASH: bool = true;

const DEFAULT_MAX_LOG_AGE_DAYS: u32 = 90;
const DEFAULT_MAX_HISTORY_PER_JOB: usize = 15;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RetentionSettings {
    pub max_log_age_days: u32,
    pub max_history_per_job: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DryModeSettings {
    pub itemize_changes: bool,
    pub checksum: bool,
}

pub struct SettingsService {
    settings: Arc<dyn SettingsRepository>,
}

impl SettingsService {
    pub fn new(settings: Arc<dyn SettingsRepository>) -> Self {
        Self { settings }
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>, AppError> {
        self.settings.get_setting(key)
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), AppError> {
        self.settings.set_setting(key, value)
    }

    pub fn delete_setting(&self, key: &str) -> Result<(), AppError> {
        self.settings.delete_setting(key)
    }

    pub fn get_log_directory(&self) -> Result<Option<String>, AppError> {
        self.settings.get_setting(KEY_LOG_DIRECTORY)
    }

    pub fn set_log_directory(&self, path: &str) -> Result<(), AppError> {
        self.settings.set_setting(KEY_LOG_DIRECTORY, path)
    }

    pub fn get_retention_settings(&self) -> Result<RetentionSettings, AppError> {
        let max_age = self
            .settings
            .get_setting(KEY_MAX_LOG_AGE_DAYS)?
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(DEFAULT_MAX_LOG_AGE_DAYS);

        let max_per_job = self
            .settings
            .get_setting(KEY_MAX_HISTORY_PER_JOB)?
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(DEFAULT_MAX_HISTORY_PER_JOB);

        Ok(RetentionSettings {
            max_log_age_days: max_age,
            max_history_per_job: max_per_job,
        })
    }

    pub fn get_auto_trailing_slash(&self) -> Result<bool, AppError> {
        Ok(self
            .settings
            .get_setting(KEY_AUTO_TRAILING_SLASH)?
            .map(|v| v == "true")
            .unwrap_or(DEFAULT_AUTO_TRAILING_SLASH))
    }

    pub fn set_auto_trailing_slash(&self, enabled: bool) -> Result<(), AppError> {
        self.settings.set_setting(
            KEY_AUTO_TRAILING_SLASH,
            if enabled { "true" } else { "false" },
        )
    }

    pub fn set_retention_settings(&self, settings: &RetentionSettings) -> Result<(), AppError> {
        self.settings
            .set_setting(KEY_MAX_LOG_AGE_DAYS, &settings.max_log_age_days.to_string())?;
        self.settings
            .set_setting(KEY_MAX_HISTORY_PER_JOB, &settings.max_history_per_job.to_string())?;
        Ok(())
    }

    pub fn get_dry_mode_settings(&self) -> Result<DryModeSettings, AppError> {
        let itemize_changes = self
            .settings
            .get_setting(KEY_DRY_MODE_ITEMIZE_CHANGES)?
            .map(|v| v == "true")
            .unwrap_or(false);

        let checksum = self
            .settings
            .get_setting(KEY_DRY_MODE_CHECKSUM)?
            .map(|v| v == "true")
            .unwrap_or(false);

        Ok(DryModeSettings {
            itemize_changes,
            checksum,
        })
    }

    pub fn set_dry_mode_settings(&self, settings: &DryModeSettings) -> Result<(), AppError> {
        self.settings.set_setting(
            KEY_DRY_MODE_ITEMIZE_CHANGES,
            if settings.itemize_changes { "true" } else { "false" },
        )?;
        self.settings.set_setting(
            KEY_DRY_MODE_CHECKSUM,
            if settings.checksum { "true" } else { "false" },
        )?;
        Ok(())
    }
}

/// Apply dry-mode settings to a job definition by injecting the appropriate
/// custom args (--itemize-changes, --checksum) if enabled and not already present.
pub fn apply_dry_mode_settings(job: &mut JobDefinition, settings: &DryModeSettings) {
    if settings.itemize_changes
        && !job
            .options
            .custom_args
            .contains(&"--itemize-changes".to_string())
    {
        job.options
            .custom_args
            .push("--itemize-changes".to_string());
    }
    if settings.checksum
        && !job
            .options
            .custom_args
            .contains(&"--checksum".to_string())
    {
        job.options.custom_args.push("--checksum".to_string());
    }
}

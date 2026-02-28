use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, PartialEq)]
pub struct HistoryRetentionConfig {
    pub max_age_days: u32,
    pub max_per_job: usize,
}

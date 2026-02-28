use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScheduleConfig {
    pub schedule_type: ScheduleType,
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum ScheduleType {
    Cron { expression: String },
    Interval { minutes: u64 },
}

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

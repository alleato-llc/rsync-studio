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

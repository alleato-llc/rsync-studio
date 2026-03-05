use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export_to = "scrubber/")]
pub struct ScrubScanResult {
    pub file_path: String,
    pub match_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export_to = "scrubber/")]
pub struct ScrubApplyResult {
    pub file_path: String,
    pub replacements: usize,
}

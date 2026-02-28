use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScrubScanResult {
    pub file_path: String,
    pub match_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScrubApplyResult {
    pub file_path: String,
    pub replacements: usize,
}

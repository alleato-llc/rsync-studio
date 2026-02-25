use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::AppError;

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

/// Scan all `.log` files in `log_dir` for occurrences of `pattern`.
/// Returns a list of files that contain the pattern, with match counts.
pub fn scrub_scan(log_dir: &str, pattern: &str) -> Result<Vec<ScrubScanResult>, AppError> {
    if pattern.is_empty() {
        return Err(AppError::ValidationError(
            "Search pattern must not be empty".to_string(),
        ));
    }

    let dir = Path::new(log_dir);
    let entries = fs::read_dir(dir)?;

    let mut results = Vec::new();

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) != Some("log") {
            continue;
        }

        let file = fs::File::open(&path)?;
        let reader = BufReader::new(file);
        let mut match_count = 0;

        for line in reader.lines() {
            let line = line?;
            match_count += line.matches(pattern).count();
        }

        if match_count > 0 {
            results.push(ScrubScanResult {
                file_path: path.to_string_lossy().to_string(),
                match_count,
            });
        }
    }

    results.sort_by(|a, b| a.file_path.cmp(&b.file_path));
    Ok(results)
}

/// Replace all occurrences of `pattern` with asterisks in the given files.
/// Files that no longer exist are silently skipped.
pub fn scrub_apply(pattern: &str, file_paths: &[String]) -> Result<Vec<ScrubApplyResult>, AppError> {
    if pattern.is_empty() {
        return Err(AppError::ValidationError(
            "Search pattern must not be empty".to_string(),
        ));
    }

    let replacement = "*".repeat(pattern.len());
    let mut results = Vec::new();

    for file_path in file_paths {
        let path = Path::new(file_path);
        if !path.exists() {
            continue;
        }

        let content = fs::read_to_string(path)?;
        let replacements = content.matches(pattern).count();

        if replacements > 0 {
            let scrubbed = content.replace(pattern, &replacement);
            fs::write(path, scrubbed)?;
        }

        results.push(ScrubApplyResult {
            file_path: file_path.clone(),
            replacements,
        });
    }

    Ok(results)
}

use std::collections::HashMap;

use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::models::backup::{BackupInvocation, InvocationStatus};

pub struct HistoryRetentionConfig {
    pub max_age_days: u32,
    pub max_per_job: usize,
}

/// Compute which invocations should be pruned based on the retention config.
///
/// Returns a list of `(invocation_id, Option<log_file_path>)` tuples to delete.
/// Skips invocations with status `Running`.
pub fn compute_invocations_to_prune(
    invocations: &[BackupInvocation],
    config: &HistoryRetentionConfig,
) -> Vec<(Uuid, Option<String>)> {
    let cutoff = Utc::now() - Duration::days(config.max_age_days as i64);
    let mut to_prune = Vec::new();
    let mut marked: std::collections::HashSet<Uuid> = std::collections::HashSet::new();

    // First pass: mark invocations older than cutoff
    for inv in invocations {
        if inv.status == InvocationStatus::Running {
            continue;
        }
        if inv.started_at < cutoff {
            marked.insert(inv.id);
            to_prune.push((inv.id, inv.log_file_path.clone()));
        }
    }

    // Second pass: group by job_id and mark excess beyond max_per_job
    let mut by_job: HashMap<Uuid, Vec<&BackupInvocation>> = HashMap::new();
    for inv in invocations {
        if inv.status == InvocationStatus::Running {
            continue;
        }
        by_job.entry(inv.job_id).or_default().push(inv);
    }

    for (_job_id, mut job_invocations) in by_job {
        // Sort newest first
        job_invocations.sort_by(|a, b| b.started_at.cmp(&a.started_at));

        if job_invocations.len() > config.max_per_job {
            for inv in &job_invocations[config.max_per_job..] {
                if !marked.contains(&inv.id) {
                    marked.insert(inv.id);
                    to_prune.push((inv.id, inv.log_file_path.clone()));
                }
            }
        }
    }

    to_prune
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};
    use uuid::Uuid;

    use crate::models::backup::{BackupInvocation, InvocationStatus, InvocationTrigger};

    fn make_invocation(
        job_id: Uuid,
        age_days: i64,
        status: InvocationStatus,
    ) -> BackupInvocation {
        BackupInvocation {
            id: Uuid::new_v4(),
            job_id,
            started_at: Utc::now() - Duration::days(age_days),
            finished_at: Some(Utc::now() - Duration::days(age_days)),
            status,
            bytes_transferred: 0,
            files_transferred: 0,
            total_files: 0,
            snapshot_path: None,
            command_executed: String::new(),
            exit_code: Some(0),
            trigger: InvocationTrigger::Manual,
            log_file_path: Some(format!("/logs/{}.log", Uuid::new_v4())),
        }
    }

    #[test]
    fn test_prune_old_invocations() {
        let job_id = Uuid::new_v4();
        let invocations = vec![
            make_invocation(job_id, 10, InvocationStatus::Succeeded),
            make_invocation(job_id, 100, InvocationStatus::Succeeded),
        ];
        let config = HistoryRetentionConfig {
            max_age_days: 90,
            max_per_job: 100,
        };
        let pruned = compute_invocations_to_prune(&invocations, &config);
        assert_eq!(pruned.len(), 1);
        assert_eq!(pruned[0].0, invocations[1].id);
    }

    #[test]
    fn test_prune_excess_per_job() {
        let job_id = Uuid::new_v4();
        let invocations: Vec<_> = (0..5)
            .map(|i| make_invocation(job_id, i, InvocationStatus::Succeeded))
            .collect();
        let config = HistoryRetentionConfig {
            max_age_days: 365,
            max_per_job: 3,
        };
        let pruned = compute_invocations_to_prune(&invocations, &config);
        assert_eq!(pruned.len(), 2);
    }

    #[test]
    fn test_skip_running_invocations() {
        let job_id = Uuid::new_v4();
        let invocations = vec![
            make_invocation(job_id, 100, InvocationStatus::Running),
        ];
        let config = HistoryRetentionConfig {
            max_age_days: 1,
            max_per_job: 0,
        };
        let pruned = compute_invocations_to_prune(&invocations, &config);
        assert!(pruned.is_empty());
    }

    #[test]
    fn test_nothing_to_prune() {
        let job_id = Uuid::new_v4();
        let invocations = vec![
            make_invocation(job_id, 1, InvocationStatus::Succeeded),
        ];
        let config = HistoryRetentionConfig {
            max_age_days: 90,
            max_per_job: 15,
        };
        let pruned = compute_invocations_to_prune(&invocations, &config);
        assert!(pruned.is_empty());
    }

    #[test]
    fn test_combined_age_and_count() {
        let job_id = Uuid::new_v4();
        let mut invocations: Vec<_> = (0..5)
            .map(|i| make_invocation(job_id, i, InvocationStatus::Succeeded))
            .collect();
        // Add an old one
        invocations.push(make_invocation(job_id, 200, InvocationStatus::Succeeded));

        let config = HistoryRetentionConfig {
            max_age_days: 90,
            max_per_job: 4,
        };
        let pruned = compute_invocations_to_prune(&invocations, &config);
        // The old one is pruned by age, plus 1 excess by count = 2 pruned
        // (6 total non-running - old one already marked = 5 remaining for count check,
        //  5 > 4 so 1 more)
        assert_eq!(pruned.len(), 2);
    }
}

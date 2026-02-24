use std::collections::HashSet;

use chrono::{DateTime, Datelike, Utc};
use uuid::Uuid;

use crate::models::backup::SnapshotRecord;
use crate::models::job::RetentionPolicy;

/// Given a list of snapshots (sorted newest-first) and a retention policy,
/// compute the set of snapshot IDs that should be deleted.
///
/// The algorithm:
/// 1. Group snapshots by calendar day, ISO week, and month
/// 2. For each period type, keep the latest snapshot per period up to the limit
/// 3. Any snapshot kept by at least one rule survives; the rest are pruned
/// 4. The most recent snapshot is always kept regardless of policy
pub fn compute_snapshots_to_delete(
    snapshots: &[SnapshotRecord],
    policy: &RetentionPolicy,
) -> Vec<Uuid> {
    if snapshots.is_empty() {
        return Vec::new();
    }

    let mut keep: HashSet<Uuid> = HashSet::new();

    // Always keep the latest snapshot
    if let Some(latest) = snapshots.first() {
        keep.insert(latest.id);
    }

    // Keep daily: latest snapshot per calendar day, up to keep_daily days
    keep_by_period(snapshots, policy.keep_daily as usize, &mut keep, |dt| {
        dt.date_naive()
    });

    // Keep weekly: latest snapshot per ISO week, up to keep_weekly weeks
    keep_by_period(snapshots, policy.keep_weekly as usize, &mut keep, |dt| {
        dt.iso_week()
    });

    // Keep monthly: latest snapshot per (year, month), up to keep_monthly months
    keep_by_period(snapshots, policy.keep_monthly as usize, &mut keep, |dt| {
        (dt.year(), dt.month())
    });

    // Everything not in the keep set gets deleted
    snapshots
        .iter()
        .filter(|s| !keep.contains(&s.id))
        .map(|s| s.id)
        .collect()
}

/// Group snapshots by a period key, then keep the latest from each distinct period
/// up to `max_periods`.
///
/// Snapshots must be sorted newest-first.
fn keep_by_period<K: Eq + std::hash::Hash>(
    snapshots: &[SnapshotRecord],
    max_periods: usize,
    keep: &mut HashSet<Uuid>,
    key_fn: impl Fn(DateTime<Utc>) -> K,
) {
    let mut seen_periods: Vec<K> = Vec::new();

    for snap in snapshots {
        let period = key_fn(snap.created_at);

        if !seen_periods.iter().any(|p| *p == period) {
            seen_periods.push(period);
            if seen_periods.len() <= max_periods {
                keep.insert(snap.id);
            }
        }
    }
}

/// Format a snapshot directory name from a timestamp.
/// Uses the format: `YYYY-MM-DD_HHMMSS`
pub fn snapshot_dir_name(timestamp: DateTime<Utc>) -> String {
    timestamp.format("%Y-%m-%d_%H%M%S").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn make_snapshot_at(id_seed: u8, created_at: DateTime<Utc>) -> SnapshotRecord {
        SnapshotRecord {
            id: Uuid::from_bytes([id_seed; 16]),
            job_id: Uuid::from_bytes([0; 16]),
            invocation_id: Uuid::from_bytes([0; 16]),
            snapshot_path: format!("/backups/{}", created_at.format("%Y-%m-%d_%H%M%S")),
            link_dest_path: None,
            created_at,
            size_bytes: 1024,
            file_count: 10,
            is_latest: false,
        }
    }

    fn dt(year: i32, month: u32, day: u32, hour: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(year, month, day, hour, 0, 0).unwrap()
    }

    #[test]
    fn empty_snapshots_returns_empty() {
        let policy = RetentionPolicy::default(); // 7 daily, 4 weekly, 6 monthly
        let to_delete = compute_snapshots_to_delete(&[], &policy);
        assert!(to_delete.is_empty());
    }

    #[test]
    fn single_snapshot_always_kept() {
        let policy = RetentionPolicy {
            keep_daily: 0,
            keep_weekly: 0,
            keep_monthly: 0,
        };
        let snaps = vec![make_snapshot_at(1, dt(2025, 6, 15, 9))];
        let to_delete = compute_snapshots_to_delete(&snaps, &policy);
        // Latest is always kept
        assert!(to_delete.is_empty());
    }

    #[test]
    fn daily_retention_keeps_one_per_day() {
        let policy = RetentionPolicy {
            keep_daily: 3,
            keep_weekly: 0,
            keep_monthly: 0,
        };

        // 5 snapshots over 3 days (newest first)
        let snaps = vec![
            make_snapshot_at(5, dt(2025, 6, 17, 14)), // Day 3 (latest, always kept)
            make_snapshot_at(4, dt(2025, 6, 17, 9)),   // Day 3 (same day, not kept)
            make_snapshot_at(3, dt(2025, 6, 16, 12)),  // Day 2 (kept: daily #2)
            make_snapshot_at(2, dt(2025, 6, 15, 18)),  // Day 1 (kept: daily #3)
            make_snapshot_at(1, dt(2025, 6, 15, 9)),   // Day 1 (same day, not kept)
        ];

        let to_delete = compute_snapshots_to_delete(&snaps, &policy);
        let delete_set: HashSet<Uuid> = to_delete.into_iter().collect();

        // Snap 5 kept (latest), snap 3 kept (daily day 2), snap 2 kept (daily day 1)
        // Snap 4 and snap 1 should be deleted
        assert!(!delete_set.contains(&Uuid::from_bytes([5; 16])));
        assert!(delete_set.contains(&Uuid::from_bytes([4; 16])));
        assert!(!delete_set.contains(&Uuid::from_bytes([3; 16])));
        assert!(!delete_set.contains(&Uuid::from_bytes([2; 16])));
        assert!(delete_set.contains(&Uuid::from_bytes([1; 16])));
    }

    #[test]
    fn weekly_retention_keeps_one_per_week() {
        let policy = RetentionPolicy {
            keep_daily: 0,
            keep_weekly: 2,
            keep_monthly: 0,
        };

        // Snapshots across 3 different ISO weeks (newest first)
        // 2025-06-16 = Monday of week 25
        // 2025-06-09 = Monday of week 24
        // 2025-06-02 = Monday of week 23
        let snaps = vec![
            make_snapshot_at(3, dt(2025, 6, 18, 9)), // Week 25 (latest, kept)
            make_snapshot_at(2, dt(2025, 6, 11, 9)), // Week 24 (kept: weekly #2)
            make_snapshot_at(1, dt(2025, 6, 4, 9)),  // Week 23 (exceeds keep_weekly=2)
        ];

        let to_delete = compute_snapshots_to_delete(&snaps, &policy);
        let delete_set: HashSet<Uuid> = to_delete.into_iter().collect();

        assert!(!delete_set.contains(&Uuid::from_bytes([3; 16]))); // latest
        assert!(!delete_set.contains(&Uuid::from_bytes([2; 16]))); // weekly #2
        assert!(delete_set.contains(&Uuid::from_bytes([1; 16])));  // pruned
    }

    #[test]
    fn monthly_retention_keeps_one_per_month() {
        let policy = RetentionPolicy {
            keep_daily: 0,
            keep_weekly: 0,
            keep_monthly: 2,
        };

        let snaps = vec![
            make_snapshot_at(3, dt(2025, 6, 15, 9)),  // June (latest, kept)
            make_snapshot_at(2, dt(2025, 5, 20, 9)),   // May (kept: monthly #2)
            make_snapshot_at(1, dt(2025, 4, 10, 9)),   // April (exceeds keep_monthly=2)
        ];

        let to_delete = compute_snapshots_to_delete(&snaps, &policy);
        let delete_set: HashSet<Uuid> = to_delete.into_iter().collect();

        assert!(!delete_set.contains(&Uuid::from_bytes([3; 16])));
        assert!(!delete_set.contains(&Uuid::from_bytes([2; 16])));
        assert!(delete_set.contains(&Uuid::from_bytes([1; 16])));
    }

    #[test]
    fn overlapping_policies_union_keeps() {
        // A snapshot kept by weekly but not daily should survive
        let policy = RetentionPolicy {
            keep_daily: 1,
            keep_weekly: 3,
            keep_monthly: 0,
        };

        let snaps = vec![
            make_snapshot_at(3, dt(2025, 6, 18, 9)),  // Week 25, Day 18 (latest)
            make_snapshot_at(2, dt(2025, 6, 11, 9)),   // Week 24, Day 11 (kept by weekly)
            make_snapshot_at(1, dt(2025, 6, 4, 9)),    // Week 23, Day 4 (kept by weekly)
        ];

        let to_delete = compute_snapshots_to_delete(&snaps, &policy);
        // keep_daily=1 would only keep snap 3's day
        // keep_weekly=3 keeps all 3 weeks
        // Union: all 3 kept
        assert!(to_delete.is_empty());
    }

    #[test]
    fn all_zero_policy_keeps_only_latest() {
        let policy = RetentionPolicy {
            keep_daily: 0,
            keep_weekly: 0,
            keep_monthly: 0,
        };

        let snaps = vec![
            make_snapshot_at(3, dt(2025, 6, 18, 9)),
            make_snapshot_at(2, dt(2025, 6, 11, 9)),
            make_snapshot_at(1, dt(2025, 6, 4, 9)),
        ];

        let to_delete = compute_snapshots_to_delete(&snaps, &policy);
        let delete_set: HashSet<Uuid> = to_delete.into_iter().collect();

        assert!(!delete_set.contains(&Uuid::from_bytes([3; 16]))); // latest always kept
        assert!(delete_set.contains(&Uuid::from_bytes([2; 16])));
        assert!(delete_set.contains(&Uuid::from_bytes([1; 16])));
    }

    #[test]
    fn generous_policy_keeps_everything() {
        let policy = RetentionPolicy {
            keep_daily: 100,
            keep_weekly: 100,
            keep_monthly: 100,
        };

        let snaps = vec![
            make_snapshot_at(3, dt(2025, 6, 18, 9)),
            make_snapshot_at(2, dt(2025, 6, 11, 9)),
            make_snapshot_at(1, dt(2025, 6, 4, 9)),
        ];

        let to_delete = compute_snapshots_to_delete(&snaps, &policy);
        assert!(to_delete.is_empty());
    }

    #[test]
    fn snapshot_dir_name_format() {
        let ts = dt(2025, 6, 15, 14);
        assert_eq!(snapshot_dir_name(ts), "2025-06-15_140000");
    }

    #[test]
    fn multiple_snapshots_same_day_keeps_newest() {
        let policy = RetentionPolicy {
            keep_daily: 2,
            keep_weekly: 0,
            keep_monthly: 0,
        };

        // 4 snapshots, 2 days, 2 per day
        let snaps = vec![
            make_snapshot_at(4, dt(2025, 6, 16, 18)),  // Day 2 pm (latest)
            make_snapshot_at(3, dt(2025, 6, 16, 9)),   // Day 2 am
            make_snapshot_at(2, dt(2025, 6, 15, 18)),  // Day 1 pm (kept: daily #2)
            make_snapshot_at(1, dt(2025, 6, 15, 9)),   // Day 1 am
        ];

        let to_delete = compute_snapshots_to_delete(&snaps, &policy);
        let delete_set: HashSet<Uuid> = to_delete.into_iter().collect();

        assert!(!delete_set.contains(&Uuid::from_bytes([4; 16]))); // latest
        assert!(delete_set.contains(&Uuid::from_bytes([3; 16])));  // same day as latest
        assert!(!delete_set.contains(&Uuid::from_bytes([2; 16]))); // daily #2
        assert!(delete_set.contains(&Uuid::from_bytes([1; 16])));  // same day as #2
    }
}

use std::str::FromStr;

use chrono::{DateTime, Duration, Utc};
use croner::Cron;

use crate::models::schedule::{ScheduleConfig, ScheduleType};

/// Determines whether a scheduled job is due for execution.
///
/// Returns `true` if the job should run now based on the schedule configuration
/// and the time it was last run.
pub fn is_job_due(
    schedule: &ScheduleConfig,
    last_run: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> bool {
    if !schedule.enabled {
        return false;
    }

    match &schedule.schedule_type {
        ScheduleType::Cron { expression } => {
            let cron = match Cron::from_str(expression) {
                Ok(c) => c,
                Err(_) => return false,
            };

            match last_run {
                Some(last) => {
                    // Find the next occurrence after the last run
                    match cron.find_next_occurrence(&last, false) {
                        Ok(next) => next <= now,
                        Err(_) => false,
                    }
                }
                // Never run before — due immediately
                None => true,
            }
        }
        ScheduleType::Interval { minutes } => match last_run {
            Some(last) => {
                let elapsed = now.signed_duration_since(last);
                elapsed >= Duration::minutes(*minutes as i64)
            }
            // Never run before — due immediately
            None => true,
        },
    }
}

/// Computes the next run time for a schedule starting from the given time.
///
/// Returns `None` if the cron expression is invalid.
pub fn next_run_time(
    schedule: &ScheduleConfig,
    from: DateTime<Utc>,
) -> Option<DateTime<Utc>> {
    if !schedule.enabled {
        return None;
    }

    match &schedule.schedule_type {
        ScheduleType::Cron { expression } => {
            let cron = Cron::from_str(expression).ok()?;
            cron.find_next_occurrence(&from, false).ok()
        }
        ScheduleType::Interval { minutes } => {
            Some(from + Duration::minutes(*minutes as i64))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn make_schedule(schedule_type: ScheduleType, enabled: bool) -> ScheduleConfig {
        ScheduleConfig {
            schedule_type,
            enabled,
        }
    }

    // --- is_job_due: Interval ---

    #[test]
    fn interval_never_run_is_due() {
        let schedule = make_schedule(ScheduleType::Interval { minutes: 30 }, true);
        let now = Utc::now();
        assert!(is_job_due(&schedule, None, now));
    }

    #[test]
    fn interval_elapsed_is_due() {
        let schedule = make_schedule(ScheduleType::Interval { minutes: 30 }, true);
        let now = Utc::now();
        let last_run = now - Duration::minutes(31);
        assert!(is_job_due(&schedule, Some(last_run), now));
    }

    #[test]
    fn interval_not_elapsed_not_due() {
        let schedule = make_schedule(ScheduleType::Interval { minutes: 30 }, true);
        let now = Utc::now();
        let last_run = now - Duration::minutes(10);
        assert!(!is_job_due(&schedule, Some(last_run), now));
    }

    #[test]
    fn interval_exact_boundary_is_due() {
        let schedule = make_schedule(ScheduleType::Interval { minutes: 60 }, true);
        let now = Utc::now();
        let last_run = now - Duration::minutes(60);
        assert!(is_job_due(&schedule, Some(last_run), now));
    }

    #[test]
    fn interval_disabled_not_due() {
        let schedule = make_schedule(ScheduleType::Interval { minutes: 1 }, false);
        assert!(!is_job_due(&schedule, None, Utc::now()));
    }

    // --- is_job_due: Cron ---

    #[test]
    fn cron_never_run_is_due() {
        let schedule = make_schedule(
            ScheduleType::Cron {
                expression: "* * * * *".to_string(), // every minute
            },
            true,
        );
        assert!(is_job_due(&schedule, None, Utc::now()));
    }

    #[test]
    fn cron_due_when_next_occurrence_passed() {
        // "0 9 * * *" = daily at 09:00 UTC
        let schedule = make_schedule(
            ScheduleType::Cron {
                expression: "0 9 * * *".to_string(),
            },
            true,
        );
        // Last run was yesterday at 09:00, now is today at 10:00
        let last_run = Utc.with_ymd_and_hms(2025, 6, 15, 9, 0, 0).unwrap();
        let now = Utc.with_ymd_and_hms(2025, 6, 16, 10, 0, 0).unwrap();
        assert!(is_job_due(&schedule, Some(last_run), now));
    }

    #[test]
    fn cron_not_due_before_next_occurrence() {
        // "0 9 * * *" = daily at 09:00 UTC
        let schedule = make_schedule(
            ScheduleType::Cron {
                expression: "0 9 * * *".to_string(),
            },
            true,
        );
        // Last run was today at 09:00, now is today at 09:30
        let last_run = Utc.with_ymd_and_hms(2025, 6, 16, 9, 0, 0).unwrap();
        let now = Utc.with_ymd_and_hms(2025, 6, 16, 9, 30, 0).unwrap();
        assert!(!is_job_due(&schedule, Some(last_run), now));
    }

    #[test]
    fn cron_disabled_not_due() {
        let schedule = make_schedule(
            ScheduleType::Cron {
                expression: "* * * * *".to_string(),
            },
            false,
        );
        assert!(!is_job_due(&schedule, None, Utc::now()));
    }

    #[test]
    fn cron_invalid_expression_not_due() {
        let schedule = make_schedule(
            ScheduleType::Cron {
                expression: "invalid cron".to_string(),
            },
            true,
        );
        assert!(!is_job_due(&schedule, None, Utc::now()));
    }

    // --- next_run_time: Interval ---

    #[test]
    fn interval_next_run_time() {
        let schedule = make_schedule(ScheduleType::Interval { minutes: 45 }, true);
        let from = Utc.with_ymd_and_hms(2025, 6, 16, 10, 0, 0).unwrap();
        let next = next_run_time(&schedule, from).unwrap();
        assert_eq!(next, Utc.with_ymd_and_hms(2025, 6, 16, 10, 45, 0).unwrap());
    }

    // --- next_run_time: Cron ---

    #[test]
    fn cron_next_run_time() {
        let schedule = make_schedule(
            ScheduleType::Cron {
                expression: "0 9 * * *".to_string(),
            },
            true,
        );
        let from = Utc.with_ymd_and_hms(2025, 6, 16, 10, 0, 0).unwrap();
        let next = next_run_time(&schedule, from).unwrap();
        // Next 09:00 is the following day
        assert_eq!(next, Utc.with_ymd_and_hms(2025, 6, 17, 9, 0, 0).unwrap());
    }

    #[test]
    fn cron_next_run_time_invalid_returns_none() {
        let schedule = make_schedule(
            ScheduleType::Cron {
                expression: "not valid".to_string(),
            },
            true,
        );
        assert!(next_run_time(&schedule, Utc::now()).is_none());
    }

    #[test]
    fn disabled_schedule_next_run_returns_none() {
        let schedule = make_schedule(ScheduleType::Interval { minutes: 10 }, false);
        assert!(next_run_time(&schedule, Utc::now()).is_none());
    }

    // --- Edge cases ---

    #[test]
    fn cron_every_minute_rapid_fire() {
        let schedule = make_schedule(
            ScheduleType::Cron {
                expression: "* * * * *".to_string(),
            },
            true,
        );
        let last_run = Utc.with_ymd_and_hms(2025, 6, 16, 10, 0, 0).unwrap();
        let now = Utc.with_ymd_and_hms(2025, 6, 16, 10, 1, 0).unwrap();
        assert!(is_job_due(&schedule, Some(last_run), now));
    }

    #[test]
    fn cron_every_minute_not_yet() {
        let schedule = make_schedule(
            ScheduleType::Cron {
                expression: "* * * * *".to_string(),
            },
            true,
        );
        let last_run = Utc.with_ymd_and_hms(2025, 6, 16, 10, 0, 0).unwrap();
        let now = Utc.with_ymd_and_hms(2025, 6, 16, 10, 0, 30).unwrap();
        assert!(!is_job_due(&schedule, Some(last_run), now));
    }

    #[test]
    fn interval_one_minute() {
        let schedule = make_schedule(ScheduleType::Interval { minutes: 1 }, true);
        let now = Utc::now();
        let last_run = now - Duration::seconds(61);
        assert!(is_job_due(&schedule, Some(last_run), now));
    }
}

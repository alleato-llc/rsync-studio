use regex::Regex;
use std::sync::LazyLock;
use uuid::Uuid;

use crate::models::progress::ProgressUpdate;

// rsync --progress output format:
//      32,768 100%   31.25kB/s    0:00:00 (xfr#1, to-chk=2/4)
static PROGRESS_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^\s*([\d,]+)\s+(\d+)%\s+([\d.]+\w+/s)\s+(\d+:\d+:\d+)(?:\s+\(xfr#(\d+),\s*to-chk=(\d+)/(\d+)\))?",
    )
    .expect("invalid progress regex")
});

pub fn parse_progress_line(line: &str, invocation_id: Uuid) -> Option<ProgressUpdate> {
    let caps = PROGRESS_RE.captures(line)?;

    let bytes_transferred: u64 = caps[1].replace(',', "").parse().ok()?;
    let percentage: f64 = caps[2].parse().ok()?;
    let transfer_rate = caps[3].to_string();
    let elapsed = caps[4].to_string();

    let files_transferred = caps
        .get(5)
        .and_then(|m| m.as_str().parse::<u64>().ok())
        .unwrap_or(0);
    let files_remaining = caps
        .get(6)
        .and_then(|m| m.as_str().parse::<u64>().ok())
        .unwrap_or(0);
    let files_total = caps
        .get(7)
        .and_then(|m| m.as_str().parse::<u64>().ok())
        .unwrap_or(0);

    Some(ProgressUpdate {
        invocation_id,
        bytes_transferred,
        percentage,
        transfer_rate,
        elapsed,
        files_transferred,
        files_remaining,
        files_total,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_id() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
    }

    #[test]
    fn test_parse_full_progress_line() {
        let line = "     32,768 100%   31.25kB/s    0:00:00 (xfr#1, to-chk=2/4)";
        let update = parse_progress_line(line, test_id()).unwrap();
        assert_eq!(update.bytes_transferred, 32768);
        assert_eq!(update.percentage, 100.0);
        assert_eq!(update.transfer_rate, "31.25kB/s");
        assert_eq!(update.elapsed, "0:00:00");
        assert_eq!(update.files_transferred, 1);
        assert_eq!(update.files_remaining, 2);
        assert_eq!(update.files_total, 4);
    }

    #[test]
    fn test_parse_without_xfr_suffix() {
        let line = "  1,048,576  50%  500.00kB/s    0:00:01";
        let update = parse_progress_line(line, test_id()).unwrap();
        assert_eq!(update.bytes_transferred, 1048576);
        assert_eq!(update.percentage, 50.0);
        assert_eq!(update.transfer_rate, "500.00kB/s");
        assert_eq!(update.elapsed, "0:00:01");
        assert_eq!(update.files_transferred, 0);
        assert_eq!(update.files_remaining, 0);
        assert_eq!(update.files_total, 0);
    }

    #[test]
    fn test_parse_large_transfer() {
        let line = "  2,147,483,648  75%    1.20GB/s    0:05:30 (xfr#150, to-chk=50/200)";
        let update = parse_progress_line(line, test_id()).unwrap();
        assert_eq!(update.bytes_transferred, 2147483648);
        assert_eq!(update.percentage, 75.0);
        assert_eq!(update.transfer_rate, "1.20GB/s");
        assert_eq!(update.elapsed, "0:05:30");
        assert_eq!(update.files_transferred, 150);
        assert_eq!(update.files_remaining, 50);
        assert_eq!(update.files_total, 200);
    }

    #[test]
    fn test_non_progress_line_returns_none() {
        assert!(parse_progress_line("sending incremental file list", test_id()).is_none());
        assert!(parse_progress_line("", test_id()).is_none());
        assert!(parse_progress_line("some random text", test_id()).is_none());
    }

    #[test]
    fn test_parse_zero_progress() {
        let line = "          0   0%    0.00kB/s    0:00:00 (xfr#0, to-chk=10/10)";
        let update = parse_progress_line(line, test_id()).unwrap();
        assert_eq!(update.bytes_transferred, 0);
        assert_eq!(update.percentage, 0.0);
        assert_eq!(update.files_transferred, 0);
        assert_eq!(update.files_remaining, 10);
        assert_eq!(update.files_total, 10);
    }

    #[test]
    fn test_parse_completed_transfer() {
        let line = "     65,536 100%   62.50kB/s    0:00:01 (xfr#3, to-chk=0/3)";
        let update = parse_progress_line(line, test_id()).unwrap();
        assert_eq!(update.percentage, 100.0);
        assert_eq!(update.files_transferred, 3);
        assert_eq!(update.files_remaining, 0);
        assert_eq!(update.files_total, 3);
    }
}

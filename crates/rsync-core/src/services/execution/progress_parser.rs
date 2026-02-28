use regex::Regex;
use std::sync::LazyLock;
use uuid::Uuid;

use crate::models::execution::progress::TransferSummary;
use crate::models::progress::ProgressUpdate;

// rsync --progress output format:
//      32,768 100%   31.25kB/s    0:00:00 (xfr#1, to-chk=2/4)
// With -h (human-readable):
//     205.18M 100%    7.46M/s    0:00:26 (xfr#1, to-chk=0/1)
// rsync 3.1+ may use ir-chk instead of to-chk (incremental recursion):
//      32,768 100%   31.25kB/s    0:00:00 (xfr#1, ir-chk=2/4)
static PROGRESS_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^\s*([\d.,]+[KMGkmg]?)\s+(\d+)%\s+([\d.]+\w+/s)\s+(\d+:\d+:\d+)(?:\s+\(xfr#(\d+),\s*(?:to|ir)-chk=(\d+)/(\d+)\))?",
    )
    .expect("invalid progress regex")
});

// rsync summary line: "sent 123,456 bytes  received 789 bytes  41,415.00 bytes/sec"
// With -h: "sent 120.56K bytes  received 789 bytes  40.45K bytes/sec"
static SUMMARY_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^sent ([\d.,]+[KMGkmg]?) bytes\s+received ([\d.,]+[KMGkmg]?) bytes")
        .expect("invalid summary regex")
});

/// Parse the rsync summary line: "sent 123,456 bytes  received 789 bytes  ..."
/// Returns sent and received byte counts.
pub fn parse_summary_line(line: &str) -> Option<TransferSummary> {
    let caps = SUMMARY_RE.captures(line)?;
    let sent = parse_human_bytes(&caps[1])?;
    let received = parse_human_bytes(&caps[2])?;
    Some(TransferSummary {
        sent_bytes: sent,
        received_bytes: received,
    })
}

/// Parse a byte value that may be human-readable (e.g. "205.18M") or a raw integer with commas.
pub fn parse_human_bytes(s: &str) -> Option<u64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    let last = s.as_bytes()[s.len() - 1];
    match last {
        b'K' | b'k' => {
            let num: f64 = s[..s.len() - 1].replace(',', "").parse().ok()?;
            Some((num * 1_000.0) as u64)
        }
        b'M' | b'm' => {
            let num: f64 = s[..s.len() - 1].replace(',', "").parse().ok()?;
            Some((num * 1_000_000.0) as u64)
        }
        b'G' | b'g' => {
            let num: f64 = s[..s.len() - 1].replace(',', "").parse().ok()?;
            Some((num * 1_000_000_000.0) as u64)
        }
        _ => {
            // Plain integer, possibly with commas
            s.replace(',', "").parse().ok()
        }
    }
}

pub fn parse_progress_line(line: &str, invocation_id: Uuid) -> Option<ProgressUpdate> {
    let caps = PROGRESS_RE.captures(line)?;

    let bytes_transferred: u64 = parse_human_bytes(&caps[1])?;
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

    #[test]
    fn test_parse_human_readable_megabytes() {
        let line = "  205.18M 100%    7.46M/s    0:00:26 (xfr#1, to-chk=0/1)";
        let update = parse_progress_line(line, test_id()).unwrap();
        assert_eq!(update.bytes_transferred, 205_180_000);
        assert_eq!(update.percentage, 100.0);
        assert_eq!(update.transfer_rate, "7.46M/s");
        assert_eq!(update.elapsed, "0:00:26");
        assert_eq!(update.files_transferred, 1);
        assert_eq!(update.files_remaining, 0);
        assert_eq!(update.files_total, 1);
    }

    #[test]
    fn test_parse_human_readable_gigabytes() {
        let line = "   1.20G  75%    1.20GB/s    0:05:30 (xfr#150, to-chk=50/200)";
        let update = parse_progress_line(line, test_id()).unwrap();
        assert_eq!(update.bytes_transferred, 1_200_000_000);
        assert_eq!(update.percentage, 75.0);
        assert_eq!(update.transfer_rate, "1.20GB/s");
        assert_eq!(update.elapsed, "0:05:30");
        assert_eq!(update.files_transferred, 150);
        assert_eq!(update.files_remaining, 50);
        assert_eq!(update.files_total, 200);
    }

    #[test]
    fn test_parse_human_readable_kilobytes() {
        let line = "  512.00K  50%   256.00kB/s    0:00:02";
        let update = parse_progress_line(line, test_id()).unwrap();
        assert_eq!(update.bytes_transferred, 512_000);
        assert_eq!(update.percentage, 50.0);
        assert_eq!(update.transfer_rate, "256.00kB/s");
    }

    #[test]
    fn test_parse_human_bytes_helper() {
        assert_eq!(parse_human_bytes("205.18M"), Some(205_180_000));
        assert_eq!(parse_human_bytes("1.20G"), Some(1_200_000_000));
        assert_eq!(parse_human_bytes("512.00K"), Some(512_000));
        assert_eq!(parse_human_bytes("32,768"), Some(32_768));
        assert_eq!(parse_human_bytes("0"), Some(0));
        assert_eq!(parse_human_bytes(""), None);
    }

    // --- ir-chk tests (rsync 3.1+ incremental recursion) ---

    #[test]
    fn test_parse_ir_chk_format() {
        let line = "     32,768 100%   31.25kB/s    0:00:00 (xfr#1, ir-chk=2/4)";
        let update = parse_progress_line(line, test_id()).unwrap();
        assert_eq!(update.bytes_transferred, 32768);
        assert_eq!(update.percentage, 100.0);
        assert_eq!(update.files_transferred, 1);
        assert_eq!(update.files_remaining, 2);
        assert_eq!(update.files_total, 4);
    }

    #[test]
    fn test_parse_ir_chk_multi_file() {
        let line = "     65,536 100%   62.50kB/s    0:00:01 (xfr#42, ir-chk=0/100)";
        let update = parse_progress_line(line, test_id()).unwrap();
        assert_eq!(update.files_transferred, 42);
        assert_eq!(update.files_remaining, 0);
        assert_eq!(update.files_total, 100);
    }

    #[test]
    fn test_parse_ir_chk_human_readable() {
        let line = "  205.18M 100%    7.46M/s    0:00:26 (xfr#5, ir-chk=0/10)";
        let update = parse_progress_line(line, test_id()).unwrap();
        assert_eq!(update.bytes_transferred, 205_180_000);
        assert_eq!(update.files_transferred, 5);
        assert_eq!(update.files_remaining, 0);
        assert_eq!(update.files_total, 10);
    }

    // --- Summary line tests ---

    #[test]
    fn test_parse_summary_line_plain() {
        let line = "sent 123,456 bytes  received 789 bytes  41,415.00 bytes/sec";
        let summary = parse_summary_line(line).unwrap();
        assert_eq!(summary.sent_bytes, 123_456);
        assert_eq!(summary.received_bytes, 789);
    }

    #[test]
    fn test_parse_summary_line_human_readable() {
        let line = "sent 120.56K bytes  received 789 bytes  40.45K bytes/sec";
        let summary = parse_summary_line(line).unwrap();
        assert_eq!(summary.sent_bytes, 120_560);
        assert_eq!(summary.received_bytes, 789);
    }

    #[test]
    fn test_parse_summary_line_large() {
        let line = "sent 1.23G bytes  received 45.67M bytes  123.45M bytes/sec";
        let summary = parse_summary_line(line).unwrap();
        assert_eq!(summary.sent_bytes, 1_230_000_000);
        assert_eq!(summary.received_bytes, 45_670_000);
    }

    #[test]
    fn test_parse_summary_line_small() {
        let line = "sent 234 bytes  received 56 bytes  96.67 bytes/sec";
        let summary = parse_summary_line(line).unwrap();
        assert_eq!(summary.sent_bytes, 234);
        assert_eq!(summary.received_bytes, 56);
    }

    #[test]
    fn test_parse_summary_line_not_summary() {
        assert!(parse_summary_line("sending incremental file list").is_none());
        assert!(parse_summary_line("total size is 987,654  speedup is 7.96").is_none());
        assert!(parse_summary_line("").is_none());
    }
}

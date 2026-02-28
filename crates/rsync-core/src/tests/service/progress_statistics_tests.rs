/// End-to-end tests verifying the pipeline from rsync output parsing
/// through to statistics recording and aggregation.

use std::sync::Arc;

use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::database::sqlite::Database;
use crate::models::backup::{
    BackupInvocation, ExecutionOutput, InvocationStatus, InvocationTrigger, TransferStats,
};
use crate::repository::sqlite::invocation::SqliteInvocationRepository;
use crate::repository::sqlite::job::SqliteJobRepository;
use crate::repository::sqlite::statistics::SqliteStatisticsRepository;
use crate::repository::invocation::InvocationRepository;
use crate::repository::job::JobRepository;
use crate::services::progress_parser::{parse_progress_line, parse_summary_line};
use crate::services::statistics_service::StatisticsService;
use crate::tests::test_helpers::create_test_job;

fn setup() -> (
    SqliteJobRepository,
    SqliteInvocationRepository,
    StatisticsService,
) {
    let db = Database::in_memory().unwrap();
    let conn = db.conn();
    let stats_repo = Arc::new(SqliteStatisticsRepository::new(conn.clone()));
    (
        SqliteJobRepository::new(conn.clone()),
        SqliteInvocationRepository::new(conn),
        StatisticsService::new(stats_repo),
    )
}

/// Simulate the job_executor event loop logic for a sequence of rsync output lines.
/// Returns (final_bytes, final_files, final_total, speedup).
fn simulate_execution(stdout_lines: &[&str]) -> (u64, u64, u64, Option<f64>) {
    let inv_id = Uuid::new_v4();
    let mut last_bytes: u64 = 0;
    let mut last_files: u64 = 0;
    let mut last_total: u64 = 0;
    let mut last_speedup: Option<f64> = None;
    let mut summary_sent_bytes: Option<u64> = None;

    let speedup_re = regex::Regex::new(r"speedup is ([\d.]+)").unwrap();

    for line in stdout_lines {
        // Parse progress
        if let Some(progress) = parse_progress_line(line, inv_id) {
            last_bytes = progress.bytes_transferred;
            last_files = progress.files_transferred;
            last_total = progress.files_total;
        }

        // Parse summary
        if let Some(summary) = parse_summary_line(line) {
            summary_sent_bytes = Some(summary.sent_bytes);
        }

        // Parse speedup
        if let Some(caps) = speedup_re.captures(line) {
            if let Some(m) = caps.get(1) {
                if let Ok(val) = m.as_str().parse::<f64>() {
                    last_speedup = Some(val);
                }
            }
        }
    }

    let final_bytes = summary_sent_bytes.unwrap_or(last_bytes);
    (final_bytes, last_files, last_total, last_speedup)
}

// --- Multi-file transfer simulation tests ---

#[test]
fn test_multi_file_transfer_bytes_from_summary() {
    // Simulate rsync transferring 3 files with per-file progress, then summary
    let lines = &[
        "sending incremental file list",
        "file1.txt",
        "     32,768 100%   31.25kB/s    0:00:00 (xfr#1, to-chk=2/3)",
        "file2.txt",
        "     65,536 100%   62.50kB/s    0:00:01 (xfr#2, to-chk=1/3)",
        "file3.txt",
        "     16,384 100%   16.00kB/s    0:00:01 (xfr#3, to-chk=0/3)",
        "",
        "sent 115,000 bytes  received 789 bytes  38,596.33 bytes/sec",
        "total size is 114,688  speedup is 0.99",
    ];

    let (bytes, files, total, speedup) = simulate_execution(lines);

    // Bytes should come from summary (115,000), NOT last file's 16,384
    assert_eq!(bytes, 115_000);
    // Files should be 3 (from xfr#3)
    assert_eq!(files, 3);
    // Total should be 3
    assert_eq!(total, 3);
    // Speedup should be parsed
    assert!((speedup.unwrap() - 0.99).abs() < 0.01);
}

#[test]
fn test_multi_file_transfer_without_summary_falls_back() {
    // If rsync is killed before summary (e.g. cancelled), we fall back to last progress bytes
    let lines = &[
        "sending incremental file list",
        "file1.txt",
        "     32,768 100%   31.25kB/s    0:00:00 (xfr#1, to-chk=2/3)",
        "file2.txt",
        "     65,536 100%   62.50kB/s    0:00:01 (xfr#2, to-chk=1/3)",
        // No summary line (killed)
    ];

    let (bytes, files, _total, _speedup) = simulate_execution(lines);

    // Falls back to last file's bytes (65,536)
    assert_eq!(bytes, 65_536);
    // Files from xfr#2
    assert_eq!(files, 2);
}

#[test]
fn test_human_readable_summary() {
    let lines = &[
        "sending incremental file list",
        "bigfile.iso",
        "   1.20G 100%    120.00M/s    0:00:10 (xfr#1, to-chk=0/1)",
        "",
        "sent 1.20G bytes  received 35 bytes  120.00M bytes/sec",
        "total size is 1.20G  speedup is 1.00",
    ];

    let (bytes, files, _total, speedup) = simulate_execution(lines);

    assert_eq!(bytes, 1_200_000_000);
    assert_eq!(files, 1);
    assert!((speedup.unwrap() - 1.0).abs() < 0.01);
}

#[test]
fn test_ir_chk_format_files_counted() {
    // rsync 3.1+ with ir-chk instead of to-chk
    let lines = &[
        "sending incremental file list",
        "doc1.pdf",
        "    512.00K 100%  256.00kB/s    0:00:02 (xfr#1, ir-chk=4/5)",
        "doc2.pdf",
        "    256.00K 100%  128.00kB/s    0:00:02 (xfr#2, ir-chk=3/5)",
        "doc3.pdf",
        "    128.00K 100%   64.00kB/s    0:00:02 (xfr#3, ir-chk=2/5)",
        "",
        "sent 900,000 bytes  received 200 bytes  300,066.67 bytes/sec",
        "total size is 896,000  speedup is 1.00",
    ];

    let (bytes, files, total, _speedup) = simulate_execution(lines);

    // Bytes from summary
    assert_eq!(bytes, 900_000);
    // Files from xfr#3 (ir-chk format)
    assert_eq!(files, 3);
    assert_eq!(total, 5);
}

#[test]
fn test_no_files_transferred() {
    // rsync run where everything is up to date
    let lines = &[
        "sending incremental file list",
        "",
        "sent 234 bytes  received 12 bytes  164.00 bytes/sec",
        "total size is 987,654  speedup is 4014.86",
    ];

    let (bytes, files, _total, speedup) = simulate_execution(lines);

    // Bytes from summary (small protocol overhead)
    assert_eq!(bytes, 234);
    // No files transferred
    assert_eq!(files, 0);
    // High speedup (nothing transferred)
    assert!(speedup.unwrap() > 4000.0);
}

#[test]
fn test_intermediate_progress_lines_ignored_for_file_count() {
    // rsync shows intermediate progress (0%, 50%, 100%) for large files
    let lines = &[
        "sending incremental file list",
        "largefile.bin",
        "  524,288  50%  256.00kB/s    0:00:01",       // intermediate (no xfr#)
        "  1,048,576 100%  512.00kB/s    0:00:02 (xfr#1, to-chk=0/1)",  // completion
        "",
        "sent 1,050,000 bytes  received 35 bytes  350,011.67 bytes/sec",
        "total size is 1,048,576  speedup is 1.00",
    ];

    let (bytes, files, total, _speedup) = simulate_execution(lines);

    // Bytes from summary, NOT the last progress line
    assert_eq!(bytes, 1_050_000);
    // File count = 1 (from the completion line with xfr#1)
    assert_eq!(files, 1);
    assert_eq!(total, 1);
}

// --- Full pipeline test: parsing → invocation → statistics ---

#[test]
fn test_full_pipeline_summary_bytes_recorded_in_statistics() {
    let (job_repo, inv_repo, stats_service) = setup();
    let job = create_test_job();
    job_repo.create_job(&job).unwrap();

    // Simulate a multi-file transfer
    let lines = &[
        "file1.txt",
        "     32,768 100%   31.25kB/s    0:00:00 (xfr#1, to-chk=2/3)",
        "file2.txt",
        "     65,536 100%   62.50kB/s    0:00:01 (xfr#2, to-chk=1/3)",
        "file3.txt",
        "     16,384 100%   16.00kB/s    0:00:01 (xfr#3, to-chk=0/3)",
        "sent 115,000 bytes  received 789 bytes  38,596.33 bytes/sec",
        "total size is 114,688  speedup is 0.99",
    ];

    let (bytes, files, total, speedup) = simulate_execution(lines);

    // Create invocation with parsed values (as job_executor would)
    let started = Utc::now() - Duration::seconds(10);
    let inv = BackupInvocation {
        id: Uuid::new_v4(),
        job_id: job.id,
        started_at: started,
        finished_at: Some(Utc::now()),
        status: InvocationStatus::Succeeded,
        trigger: InvocationTrigger::Manual,
        transfer_stats: TransferStats {
            bytes_transferred: bytes,
            files_transferred: files,
            total_files: total,
        },
        execution_output: ExecutionOutput {
            command_executed: "rsync -av /src/ /dst/".to_string(),
            exit_code: Some(0),
            snapshot_path: None,
            log_file_path: None,
        },
    };

    inv_repo.create_invocation(&inv).unwrap();
    stats_service.record(job.id, &inv, speedup).unwrap();

    let agg = stats_service.get_aggregated().unwrap();
    assert_eq!(agg.total_jobs_run, 1);
    assert_eq!(agg.total_files_transferred, 3);
    assert_eq!(agg.total_bytes_transferred, 115_000);
    assert!(agg.total_duration_secs > 0.0);
}

#[test]
fn test_full_pipeline_multiple_runs_accumulate() {
    let (job_repo, inv_repo, stats_service) = setup();
    let job = create_test_job();
    job_repo.create_job(&job).unwrap();

    // Run 1: 3 files, 115KB
    let (bytes1, files1, total1, speedup1) = simulate_execution(&[
        "     32,768 100%   31.25kB/s    0:00:00 (xfr#1, to-chk=2/3)",
        "     65,536 100%   62.50kB/s    0:00:01 (xfr#2, to-chk=1/3)",
        "     16,384 100%   16.00kB/s    0:00:01 (xfr#3, to-chk=0/3)",
        "sent 115,000 bytes  received 789 bytes  38,596.33 bytes/sec",
        "total size is 114,688  speedup is 0.99",
    ]);

    let started = Utc::now() - Duration::seconds(5);
    let inv1 = BackupInvocation {
        id: Uuid::new_v4(),
        job_id: job.id,
        started_at: started,
        finished_at: Some(Utc::now()),
        status: InvocationStatus::Succeeded,
        trigger: InvocationTrigger::Manual,
        transfer_stats: TransferStats {
            bytes_transferred: bytes1,
            files_transferred: files1,
            total_files: total1,
        },
        execution_output: ExecutionOutput {
            command_executed: "rsync -av /src/ /dst/".to_string(),
            exit_code: Some(0),
            snapshot_path: None,
            log_file_path: None,
        },
    };
    inv_repo.create_invocation(&inv1).unwrap();
    stats_service.record(job.id, &inv1, speedup1).unwrap();

    // Run 2: 1 changed file, 50KB
    let (bytes2, files2, total2, speedup2) = simulate_execution(&[
        "     50,000 100%   50.00kB/s    0:00:01 (xfr#1, to-chk=0/3)",
        "sent 50,500 bytes  received 35 bytes  16,845.00 bytes/sec",
        "total size is 114,688  speedup is 2.27",
    ]);

    let started2 = Utc::now() - Duration::seconds(3);
    let inv2 = BackupInvocation {
        id: Uuid::new_v4(),
        job_id: job.id,
        started_at: started2,
        finished_at: Some(Utc::now()),
        status: InvocationStatus::Succeeded,
        trigger: InvocationTrigger::Manual,
        transfer_stats: TransferStats {
            bytes_transferred: bytes2,
            files_transferred: files2,
            total_files: total2,
        },
        execution_output: ExecutionOutput {
            command_executed: "rsync -av /src/ /dst/".to_string(),
            exit_code: Some(0),
            snapshot_path: None,
            log_file_path: None,
        },
    };
    inv_repo.create_invocation(&inv2).unwrap();
    stats_service.record(job.id, &inv2, speedup2).unwrap();

    let agg = stats_service.get_aggregated().unwrap();
    assert_eq!(agg.total_jobs_run, 2);
    assert_eq!(agg.total_files_transferred, 3 + 1); // 4 total files
    assert_eq!(agg.total_bytes_transferred, 115_000 + 50_500); // 165,500 total bytes
    assert!(agg.total_time_saved_secs > 0.0); // Run 2 has speedup > 1
}

#[test]
fn test_ir_chk_files_counted_in_statistics() {
    let (job_repo, inv_repo, stats_service) = setup();
    let job = create_test_job();
    job_repo.create_job(&job).unwrap();

    // Simulate ir-chk format output
    let (bytes, files, total, speedup) = simulate_execution(&[
        "    512.00K 100%  256.00kB/s    0:00:02 (xfr#1, ir-chk=1/2)",
        "    256.00K 100%  128.00kB/s    0:00:02 (xfr#2, ir-chk=0/2)",
        "sent 770,000 bytes  received 100 bytes  256,700.00 bytes/sec",
        "total size is 768,000  speedup is 1.00",
    ]);

    let started = Utc::now() - Duration::seconds(4);
    let inv = BackupInvocation {
        id: Uuid::new_v4(),
        job_id: job.id,
        started_at: started,
        finished_at: Some(Utc::now()),
        status: InvocationStatus::Succeeded,
        trigger: InvocationTrigger::Manual,
        transfer_stats: TransferStats {
            bytes_transferred: bytes,
            files_transferred: files,
            total_files: total,
        },
        execution_output: ExecutionOutput {
            command_executed: "rsync -av /src/ /dst/".to_string(),
            exit_code: Some(0),
            snapshot_path: None,
            log_file_path: None,
        },
    };
    inv_repo.create_invocation(&inv).unwrap();
    stats_service.record(job.id, &inv, speedup).unwrap();

    let agg = stats_service.get_aggregated().unwrap();
    assert_eq!(agg.total_files_transferred, 2);
    assert_eq!(agg.total_bytes_transferred, 770_000);
}

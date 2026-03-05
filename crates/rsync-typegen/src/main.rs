use rsync_core::models::command::{CommandExplanation, ParsedCommand};
use rsync_core::models::execution::backup::{BackupInvocation, SnapshotRecord};
use rsync_core::models::execution::itemize::ItemizedChange;
use rsync_core::models::execution::log::LogEntry;
use rsync_core::models::execution::progress::{JobStatusEvent, LogLine, ProgressUpdate};
use rsync_core::models::execution::statistics::{AggregatedStats, RunStatistic};
use rsync_core::models::job::{ExportData, JobDefinition};
use rsync_core::models::scrubber::{ScrubApplyResult, ScrubScanResult};
use rsync_core::models::settings::{DryModeSettings, RetentionSettings};
use rsync_core::models::validation::PreflightResult;
use ts_rs::TS;

fn main() {
    // Root types — export_all() recursively exports all referenced types
    JobDefinition::export_all().expect("JobDefinition");
    ExportData::export_all().expect("ExportData");
    BackupInvocation::export_all().expect("BackupInvocation");
    SnapshotRecord::export_all().expect("SnapshotRecord");
    CommandExplanation::export_all().expect("CommandExplanation");
    ParsedCommand::export_all().expect("ParsedCommand");
    PreflightResult::export_all().expect("PreflightResult");
    ScrubScanResult::export_all().expect("ScrubScanResult");
    ScrubApplyResult::export_all().expect("ScrubApplyResult");
    RetentionSettings::export_all().expect("RetentionSettings");
    DryModeSettings::export_all().expect("DryModeSettings");
    ProgressUpdate::export_all().expect("ProgressUpdate");
    LogLine::export_all().expect("LogLine");
    JobStatusEvent::export_all().expect("JobStatusEvent");
    RunStatistic::export_all().expect("RunStatistic");
    AggregatedStats::export_all().expect("AggregatedStats");
    ItemizedChange::export_all().expect("ItemizedChange");
    LogEntry::export_all().expect("LogEntry");
    println!("TypeScript types exported successfully.");
}

import { invoke } from "@tauri-apps/api/core";
import type { JobDefinition } from "@/types/job";
import type { BackupInvocation, SnapshotRecord } from "@/types/backup";
import type { CommandExplanation } from "@/types/command";
import type { AggregatedStats } from "@/types/statistics";
import type { PreflightResult } from "@/types/validation";
import type { LogFileChunk } from "@/types/log-file";
import type { ScrubScanResult, ScrubApplyResult } from "@/types/scrubber";

export async function listJobs(): Promise<JobDefinition[]> {
  return invoke<JobDefinition[]>("list_jobs");
}

export async function getJob(id: string): Promise<JobDefinition> {
  return invoke<JobDefinition>("get_job", { id });
}

export async function createJob(job: JobDefinition): Promise<JobDefinition> {
  return invoke<JobDefinition>("create_job", { job });
}

export async function updateJob(job: JobDefinition): Promise<JobDefinition> {
  return invoke<JobDefinition>("update_job", { job });
}

export async function deleteJob(id: string): Promise<void> {
  return invoke<void>("delete_job", { id });
}

export async function getJobHistory(
  jobId: string,
  limit: number
): Promise<BackupInvocation[]> {
  return invoke<BackupInvocation[]>("get_job_history", { jobId, limit });
}

export async function executeJob(jobId: string): Promise<string> {
  return invoke<string>("execute_job", { jobId });
}

export async function executeDryRun(jobId: string): Promise<string> {
  return invoke<string>("execute_job_dry_run", { jobId });
}

export async function cancelJob(jobId: string): Promise<void> {
  return invoke<void>("cancel_job", { jobId });
}

export async function getRunningJobs(): Promise<string[]> {
  return invoke<string[]>("get_running_jobs");
}

export async function listSnapshots(jobId: string): Promise<SnapshotRecord[]> {
  return invoke<SnapshotRecord[]>("list_snapshots", { jobId });
}

export async function deleteSnapshot(snapshotId: string): Promise<void> {
  return invoke<void>("delete_snapshot", { snapshotId });
}

export async function explainCommand(
  command: string
): Promise<CommandExplanation> {
  return invoke<CommandExplanation>("explain_command", { command });
}

export async function parseCommandToJob(
  command: string
): Promise<JobDefinition> {
  return invoke<JobDefinition>("parse_command_to_job", { command });
}

export async function exportJobs(): Promise<string> {
  return invoke<string>("export_jobs");
}

export async function importJobs(json: string): Promise<number> {
  return invoke<number>("import_jobs", { json });
}

export async function runPreflight(jobId: string): Promise<PreflightResult> {
  return invoke<PreflightResult>("run_preflight", { jobId });
}

export async function getStatistics(): Promise<AggregatedStats> {
  return invoke<AggregatedStats>("get_statistics");
}

export async function getStatisticsForJob(
  jobId: string
): Promise<AggregatedStats> {
  return invoke<AggregatedStats>("get_statistics_for_job", { jobId });
}

export async function exportStatistics(): Promise<string> {
  return invoke<string>("export_statistics");
}

export async function resetStatistics(): Promise<void> {
  return invoke<void>("reset_statistics");
}

export async function resetStatisticsForJob(jobId: string): Promise<void> {
  return invoke<void>("reset_statistics_for_job", { jobId });
}

// --- Settings ---

export async function getSetting(key: string): Promise<string | null> {
  return invoke<string | null>("get_setting", { key });
}

export async function setSetting(key: string, value: string): Promise<void> {
  return invoke<void>("set_setting", { key, value });
}

export async function getLogDirectory(): Promise<string> {
  return invoke<string>("get_log_directory");
}

export async function setLogDirectory(path: string): Promise<void> {
  return invoke<void>("set_log_directory", { path });
}

export interface RetentionSettings {
  max_log_age_days: number;
  max_history_per_job: number;
}

export async function getRetentionSettings(): Promise<RetentionSettings> {
  return invoke<RetentionSettings>("get_retention_settings");
}

export async function setRetentionSettings(
  settings: RetentionSettings
): Promise<void> {
  return invoke<void>("set_retention_settings", { settings });
}

// --- Trailing slash ---

export async function getAutoTrailingSlash(): Promise<boolean> {
  return invoke<boolean>("get_auto_trailing_slash");
}

export async function setAutoTrailingSlash(enabled: boolean): Promise<void> {
  return invoke<void>("set_auto_trailing_slash", { enabled });
}

// --- Max itemized changes ---

const DEFAULT_MAX_ITEMIZED = 50_000;

export async function getMaxItemizedChanges(): Promise<number> {
  const val = await getSetting("max_itemized_changes");
  return val ? parseInt(val, 10) : DEFAULT_MAX_ITEMIZED;
}

export async function setMaxItemizedChanges(value: number): Promise<void> {
  return setSetting("max_itemized_changes", value.toString());
}

// --- Dry mode settings ---

export interface DryModeSettings {
  itemize_changes: boolean;
  checksum: boolean;
}

export async function getDryModeSettings(): Promise<DryModeSettings> {
  return invoke<DryModeSettings>("get_dry_mode_settings");
}

export async function setDryModeSettings(
  settings: DryModeSettings
): Promise<void> {
  return invoke<void>("set_dry_mode_settings", { settings });
}

// --- Delete history ---

export async function deleteInvocation(invocationId: string): Promise<void> {
  return invoke<void>("delete_invocation", { invocationId });
}

export async function deleteInvocationsForJob(jobId: string): Promise<void> {
  return invoke<void>("delete_invocations_for_job", { jobId });
}

export async function countInvocations(): Promise<number> {
  return invoke<number>("count_invocations");
}

// --- Log files ---

export async function readLogFile(path: string): Promise<string> {
  return invoke<string>("read_log_file", { path });
}

export async function readLogFileLines(
  path: string,
  offset: number,
  limit: number
): Promise<LogFileChunk> {
  return invoke<LogFileChunk>("read_log_file_lines", { path, offset, limit });
}

// --- Log scrubber ---

export async function scrubScanLogs(
  pattern: string
): Promise<ScrubScanResult[]> {
  return invoke<ScrubScanResult[]>("scrub_scan_logs", { pattern });
}

export async function scrubApplyLogs(
  pattern: string,
  filePaths: string[]
): Promise<ScrubApplyResult[]> {
  return invoke<ScrubApplyResult[]>("scrub_apply_logs", { pattern, filePaths });
}

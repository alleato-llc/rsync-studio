import { invoke } from "@tauri-apps/api/core";
import type { JobDefinition } from "@/types/job";
import type { BackupInvocation, SnapshotRecord } from "@/types/backup";
import type { CommandExplanation } from "@/types/command";
import type { AggregatedStats } from "@/types/statistics";
import type { PreflightResult } from "@/types/validation";

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

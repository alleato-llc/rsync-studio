export interface TransferStats {
  bytes_transferred: number;
  files_transferred: number;
  total_files: number;
}

export interface ExecutionOutput {
  command_executed: string;
  exit_code: number | null;
  snapshot_path: string | null;
  log_file_path: string | null;
}

export interface BackupInvocation {
  id: string;
  job_id: string;
  started_at: string;
  finished_at: string | null;
  status: InvocationStatus;
  trigger: InvocationTrigger;
  transfer_stats: TransferStats;
  execution_output: ExecutionOutput;
}

export type InvocationStatus = "Running" | "Succeeded" | "Failed" | "Cancelled";

export type InvocationTrigger = "Manual" | "Scheduled";

export interface SnapshotRecord {
  id: string;
  job_id: string;
  invocation_id: string;
  snapshot_path: string;
  link_dest_path: string | null;
  created_at: string;
  size_bytes: number;
  file_count: number;
  is_latest: boolean;
}

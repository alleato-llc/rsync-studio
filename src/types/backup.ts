export interface BackupInvocation {
  id: string;
  job_id: string;
  started_at: string;
  finished_at: string | null;
  status: InvocationStatus;
  bytes_transferred: number;
  files_transferred: number;
  total_files: number;
  snapshot_path: string | null;
  command_executed: string;
  exit_code: number | null;
  trigger: InvocationTrigger;
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

import type { JobStatus } from "../job";

export interface ProgressUpdate {
  invocation_id: string;
  bytes_transferred: number;
  percentage: number;
  transfer_rate: string;
  elapsed: string;
  files_transferred: number;
  files_remaining: number;
  files_total: number;
}

export interface LogLine {
  invocation_id: string;
  timestamp: string;
  line: string;
  is_stderr: boolean;
}

export interface JobStatusEvent {
  job_id: string;
  invocation_id: string;
  status: JobStatus;
  exit_code: number | null;
  error_message: string | null;
}

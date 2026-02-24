export type StorageLocation =
  | { type: "Local"; path: string }
  | {
      type: "RemoteSsh";
      user: string;
      host: string;
      port: number;
      path: string;
      identity_file: string | null;
    }
  | { type: "RemoteRsync"; host: string; module: string; path: string };

export type BackupMode =
  | { type: "Mirror" }
  | { type: "Versioned"; backup_dir: string }
  | { type: "Snapshot"; retention_policy: RetentionPolicy };

export interface RetentionPolicy {
  keep_daily: number;
  keep_weekly: number;
  keep_monthly: number;
}

export interface SshConfig {
  port: number;
  identity_file: string | null;
  strict_host_key_checking: boolean;
  custom_ssh_command: string | null;
}

export interface RsyncOptions {
  archive: boolean;
  compress: boolean;
  verbose: boolean;
  delete: boolean;
  dry_run: boolean;
  partial: boolean;
  progress: boolean;
  human_readable: boolean;
  exclude_patterns: string[];
  include_patterns: string[];
  bandwidth_limit: number | null;
  custom_args: string[];
}

export interface JobDefinition {
  id: string;
  name: string;
  description: string | null;
  source: StorageLocation;
  destination: StorageLocation;
  backup_mode: BackupMode;
  options: RsyncOptions;
  ssh_config: SshConfig | null;
  schedule: import("./schedule").ScheduleConfig | null;
  enabled: boolean;
  created_at: string;
  updated_at: string;
}

export type JobStatus = "Idle" | "Running" | "Completed" | "Failed" | "Cancelled";

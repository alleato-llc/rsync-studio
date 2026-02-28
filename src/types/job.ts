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

export interface CoreTransferOptions {
  archive: boolean;
  compress: boolean;
  partial: boolean;
  dry_run: boolean;
}

export interface FileHandlingOptions {
  delete: boolean;
  size_only: boolean;
  checksum: boolean;
  update: boolean;
  whole_file: boolean;
  ignore_existing: boolean;
  one_file_system: boolean;
}

export interface MetadataOptions {
  hard_links: boolean;
  acls: boolean;
  xattrs: boolean;
  numeric_ids: boolean;
}

export interface OutputOptions {
  verbose: boolean;
  progress: boolean;
  human_readable: boolean;
  stats: boolean;
  itemize_changes: boolean;
}

export interface AdvancedOptions {
  exclude_patterns: string[];
  include_patterns: string[];
  bandwidth_limit: number | null;
  custom_args: string[];
}

export interface RsyncOptions {
  core_transfer: CoreTransferOptions;
  file_handling: FileHandlingOptions;
  metadata: MetadataOptions;
  output: OutputOptions;
  advanced: AdvancedOptions;
}

export interface TransferConfig {
  source: StorageLocation;
  destination: StorageLocation;
  backup_mode: BackupMode;
}

export interface JobDefinition {
  id: string;
  name: string;
  description: string | null;
  transfer: TransferConfig;
  options: RsyncOptions;
  ssh_config: SshConfig | null;
  schedule: import("./schedule").ScheduleConfig | null;
  enabled: boolean;
  created_at: string;
  updated_at: string;
}

export type JobStatus = "Idle" | "Running" | "Completed" | "Failed" | "Cancelled";

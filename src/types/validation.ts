export interface PreflightResult {
  job_id: string;
  checks: ValidationCheck[];
  overall_pass: boolean;
}

export interface ValidationCheck {
  check_type: CheckType;
  passed: boolean;
  message: string;
  severity: CheckSeverity;
}

export type CheckType =
  | "SourceExists"
  | "DestinationWritable"
  | "DiskSpace"
  | "SshConnectivity"
  | "RsyncInstalled";

export type CheckSeverity = "Error" | "Warning";

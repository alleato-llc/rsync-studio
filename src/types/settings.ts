export interface RetentionSettings {
  max_log_age_days: number;
  max_history_per_job: number;
}

export interface DryModeSettings {
  itemize_changes: boolean;
  checksum: boolean;
}

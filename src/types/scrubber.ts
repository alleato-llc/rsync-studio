export interface ScrubScanResult {
  file_path: string;
  match_count: number;
}

export interface ScrubApplyResult {
  file_path: string;
  replacements: number;
}

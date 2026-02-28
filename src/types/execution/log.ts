export type LogLevel = "Debug" | "Info" | "Warning" | "Error";

export interface LogEntry {
  id: string;
  invocation_id: string;
  job_id: string;
  timestamp: string;
  level: LogLevel;
  message: string;
}

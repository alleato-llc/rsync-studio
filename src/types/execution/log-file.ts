export interface LogFileLine {
  text: string;
  is_stderr: boolean;
}

export interface LogFileChunk {
  lines: LogFileLine[];
  total_lines: number;
}

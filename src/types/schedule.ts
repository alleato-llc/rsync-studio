export interface ScheduleConfig {
  schedule_type: ScheduleType;
  enabled: boolean;
}

export type ScheduleType =
  | { type: "Cron"; expression: string }
  | { type: "Interval"; minutes: number };

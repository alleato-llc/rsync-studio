import type { ScheduleConfig } from "@/types/schedule";
import { Badge } from "@/components/ui/badge";

interface ScheduleBadgeProps {
  schedule: ScheduleConfig | null;
}

function formatSchedule(schedule: ScheduleConfig): string {
  if (!schedule.enabled) {
    return "Schedule (paused)";
  }

  switch (schedule.schedule_type.type) {
    case "Interval": {
      const mins = schedule.schedule_type.minutes;
      if (mins >= 1440 && mins % 1440 === 0) {
        const days = mins / 1440;
        return days === 1 ? "Every day" : `Every ${days} days`;
      }
      if (mins >= 60 && mins % 60 === 0) {
        const hours = mins / 60;
        return hours === 1 ? "Every hour" : `Every ${hours} hr`;
      }
      return `Every ${mins} min`;
    }
    case "Cron":
      return schedule.schedule_type.expression;
  }
}

export function ScheduleBadge({ schedule }: ScheduleBadgeProps) {
  if (!schedule) return null;

  const variant = schedule.enabled ? "outline" : "secondary";

  return (
    <Badge variant={variant} className="text-xs">
      {formatSchedule(schedule)}
    </Badge>
  );
}

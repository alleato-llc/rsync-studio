import type { ScheduleConfig, ScheduleType } from "@/types/schedule";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

interface ScheduleFieldProps {
  value: ScheduleConfig | null;
  onChange: (schedule: ScheduleConfig | null) => void;
}

function defaultSchedule(): ScheduleConfig {
  return {
    schedule_type: { type: "Interval", minutes: 60 },
    enabled: true,
  };
}

export function ScheduleField({ value, onChange }: ScheduleFieldProps) {
  const hasSchedule = value !== null;
  const enabled = value?.enabled ?? false;

  function handleToggle(checked: boolean) {
    if (checked) {
      onChange(defaultSchedule());
    } else {
      onChange(null);
    }
  }

  function handleEnabledToggle(checked: boolean) {
    if (!value) return;
    onChange({ ...value, enabled: checked });
  }

  function handleTypeChange(type: string) {
    if (!value) return;
    const newType: ScheduleType =
      type === "Cron"
        ? { type: "Cron", expression: "0 9 * * *" }
        : { type: "Interval", minutes: 60 };
    onChange({ ...value, schedule_type: newType });
  }

  function handleCronChange(expression: string) {
    if (!value) return;
    onChange({
      ...value,
      schedule_type: { type: "Cron", expression },
    });
  }

  function handleMinutesChange(minutes: string) {
    if (!value) return;
    const parsed = parseInt(minutes, 10);
    if (isNaN(parsed) || parsed < 1) return;
    onChange({
      ...value,
      schedule_type: { type: "Interval", minutes: parsed },
    });
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <Label>Schedule</Label>
        <div className="flex items-center gap-2">
          <Label htmlFor="schedule-toggle" className="text-sm text-muted-foreground">
            Enable schedule
          </Label>
          <Switch
            id="schedule-toggle"
            checked={hasSchedule}
            onCheckedChange={handleToggle}
          />
        </div>
      </div>

      {hasSchedule && value && (
        <div className="space-y-4 rounded-md border p-4">
          <div className="flex items-center justify-between">
            <Label className="text-sm">Active</Label>
            <Switch
              checked={enabled}
              onCheckedChange={handleEnabledToggle}
            />
          </div>

          <div className="space-y-2">
            <Label className="text-sm">Type</Label>
            <Select
              value={value.schedule_type.type}
              onValueChange={handleTypeChange}
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="Cron">Cron Expression</SelectItem>
                <SelectItem value="Interval">Interval</SelectItem>
              </SelectContent>
            </Select>
          </div>

          {value.schedule_type.type === "Cron" && (
            <div className="space-y-2">
              <Label className="text-sm">Cron Expression</Label>
              <Input
                value={value.schedule_type.expression}
                onChange={(e) => handleCronChange(e.target.value)}
                placeholder="0 9 * * *"
              />
              <p className="text-xs text-muted-foreground">
                Format: minute hour day month weekday. Examples: "0 9 * * *" (daily at 9am),
                "0 */6 * * *" (every 6 hours), "0 0 * * 0" (weekly on Sunday)
              </p>
            </div>
          )}

          {value.schedule_type.type === "Interval" && (
            <div className="space-y-2">
              <Label className="text-sm">Interval (minutes)</Label>
              <Input
                type="number"
                min={1}
                value={value.schedule_type.minutes}
                onChange={(e) => handleMinutesChange(e.target.value)}
              />
              <p className="text-xs text-muted-foreground">
                Minimum interval: 1 minute. The job will run approximately every N minutes.
              </p>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

import type { BackupMode } from "@/types/job";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

interface BackupModeFieldProps {
  value: BackupMode;
  onChange: (value: BackupMode) => void;
}

export function BackupModeField({ value, onChange }: BackupModeFieldProps) {
  function handleTypeChange(type: string) {
    switch (type) {
      case "Mirror":
        onChange({ type: "Mirror" });
        break;
      case "Versioned":
        onChange({ type: "Versioned", backup_dir: "" });
        break;
      case "Snapshot":
        onChange({
          type: "Snapshot",
          retention_policy: {
            keep_daily: 7,
            keep_weekly: 4,
            keep_monthly: 6,
          },
        });
        break;
    }
  }

  return (
    <div className="space-y-3">
      <Label>Backup Mode</Label>
      <Select value={value.type} onValueChange={handleTypeChange}>
        <SelectTrigger>
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="Mirror">Mirror</SelectItem>
          <SelectItem value="Versioned">Versioned</SelectItem>
          <SelectItem value="Snapshot">Snapshot</SelectItem>
        </SelectContent>
      </Select>

      {value.type === "Versioned" && (
        <div className="space-y-1">
          <Label className="text-xs text-muted-foreground">Backup Directory</Label>
          <Input
            value={value.backup_dir}
            onChange={(e) => onChange({ ...value, backup_dir: e.target.value })}
            placeholder="/path/to/backup-dir"
          />
        </div>
      )}

      {value.type === "Snapshot" && (
        <div className="grid grid-cols-3 gap-2">
          <div className="space-y-1">
            <Label className="text-xs text-muted-foreground">Keep Daily</Label>
            <Input
              type="number"
              min={0}
              value={value.retention_policy.keep_daily}
              onChange={(e) =>
                onChange({
                  ...value,
                  retention_policy: {
                    ...value.retention_policy,
                    keep_daily: parseInt(e.target.value) || 0,
                  },
                })
              }
            />
          </div>
          <div className="space-y-1">
            <Label className="text-xs text-muted-foreground">Keep Weekly</Label>
            <Input
              type="number"
              min={0}
              value={value.retention_policy.keep_weekly}
              onChange={(e) =>
                onChange({
                  ...value,
                  retention_policy: {
                    ...value.retention_policy,
                    keep_weekly: parseInt(e.target.value) || 0,
                  },
                })
              }
            />
          </div>
          <div className="space-y-1">
            <Label className="text-xs text-muted-foreground">Keep Monthly</Label>
            <Input
              type="number"
              min={0}
              value={value.retention_policy.keep_monthly}
              onChange={(e) =>
                onChange({
                  ...value,
                  retention_policy: {
                    ...value.retention_policy,
                    keep_monthly: parseInt(e.target.value) || 0,
                  },
                })
              }
            />
          </div>
        </div>
      )}
    </div>
  );
}

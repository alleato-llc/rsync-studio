import type { JobDefinition } from "@/types/job";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Textarea } from "@/components/ui/textarea";
import { StorageLocationField } from "./storage-location-field";

interface JobFormGeneralProps {
  job: JobDefinition;
  onNameChange: (name: string) => void;
  onDescriptionChange: (description: string | null) => void;
  onEnabledChange: (enabled: boolean) => void;
  onSourceChange: (source: JobDefinition["source"]) => void;
  onDestinationChange: (destination: JobDefinition["destination"]) => void;
  errors: Record<string, string>;
  autoTrailingSlash?: boolean;
}

export function JobFormGeneral({
  job,
  onNameChange,
  onDescriptionChange,
  onEnabledChange,
  onSourceChange,
  onDestinationChange,
  errors,
  autoTrailingSlash,
}: JobFormGeneralProps) {
  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div className="space-y-2 flex-1 mr-4">
          <Label htmlFor="job-name">Name</Label>
          <Input
            id="job-name"
            value={job.name}
            onChange={(e) => onNameChange(e.target.value)}
            placeholder="My Backup Job"
          />
          {errors.name && (
            <p className="text-sm text-destructive">{errors.name}</p>
          )}
        </div>
        <div className="flex items-center gap-2 pt-6">
          <Label htmlFor="job-enabled">Enabled</Label>
          <Switch
            id="job-enabled"
            checked={job.enabled}
            onCheckedChange={onEnabledChange}
          />
        </div>
      </div>

      <div className="space-y-2">
        <Label htmlFor="job-description">Description</Label>
        <Textarea
          id="job-description"
          value={job.description ?? ""}
          onChange={(e) => onDescriptionChange(e.target.value || null)}
          placeholder="Optional description of this backup job"
          rows={2}
        />
      </div>

      <StorageLocationField
        label="Source"
        value={job.source}
        onChange={onSourceChange}
        autoTrailingSlash={autoTrailingSlash}
      />
      {errors.source && (
        <p className="text-sm text-destructive">{errors.source}</p>
      )}

      <StorageLocationField
        label="Destination"
        value={job.destination}
        onChange={onDestinationChange}
        autoTrailingSlash={autoTrailingSlash}
      />
      {errors.destination && (
        <p className="text-sm text-destructive">{errors.destination}</p>
      )}
    </div>
  );
}

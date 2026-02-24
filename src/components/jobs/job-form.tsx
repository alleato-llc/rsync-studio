import { useReducer, useState } from "react";
import { useTrailingSlash } from "@/hooks/use-trailing-slash";
import type { JobDefinition, StorageLocation, SshConfig } from "@/types/job";
import type { ScheduleConfig } from "@/types/schedule";
import { Button } from "@/components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { ScrollArea } from "@/components/ui/scroll-area";
import { JobFormGeneral } from "./job-form-general";
import { BackupModeField } from "./backup-mode-field";
import { RsyncOptionsField } from "./rsync-options-field";
import { SshConfigField } from "./ssh-config-field";
import { ScheduleField } from "./schedule-field";
import { CommandPreview } from "./command-preview";

// --- Reducer ---

type Action =
  | { type: "SET_NAME"; name: string }
  | { type: "SET_DESCRIPTION"; description: string | null }
  | { type: "SET_ENABLED"; enabled: boolean }
  | { type: "SET_SOURCE"; source: StorageLocation }
  | { type: "SET_DESTINATION"; destination: StorageLocation }
  | { type: "SET_BACKUP_MODE"; mode: JobDefinition["backup_mode"] }
  | { type: "SET_OPTIONS"; options: JobDefinition["options"] }
  | { type: "SET_SSH_CONFIG"; ssh_config: SshConfig }
  | { type: "SET_SCHEDULE"; schedule: ScheduleConfig | null };

function needsSshConfig(source: StorageLocation, destination: StorageLocation): boolean {
  return source.type === "RemoteSsh" || destination.type === "RemoteSsh";
}

function defaultSshConfig(): SshConfig {
  return {
    port: 22,
    identity_file: null,
    strict_host_key_checking: true,
    custom_ssh_command: null,
  };
}

function jobReducer(state: JobDefinition, action: Action): JobDefinition {
  switch (action.type) {
    case "SET_NAME":
      return { ...state, name: action.name };
    case "SET_DESCRIPTION":
      return { ...state, description: action.description };
    case "SET_ENABLED":
      return { ...state, enabled: action.enabled };
    case "SET_SOURCE": {
      const ssh = needsSshConfig(action.source, state.destination)
        ? state.ssh_config ?? defaultSshConfig()
        : null;
      return { ...state, source: action.source, ssh_config: ssh };
    }
    case "SET_DESTINATION": {
      const ssh = needsSshConfig(state.source, action.destination)
        ? state.ssh_config ?? defaultSshConfig()
        : null;
      return { ...state, destination: action.destination, ssh_config: ssh };
    }
    case "SET_BACKUP_MODE":
      return { ...state, backup_mode: action.mode };
    case "SET_OPTIONS":
      return { ...state, options: action.options };
    case "SET_SSH_CONFIG":
      return { ...state, ssh_config: action.ssh_config };
    case "SET_SCHEDULE":
      return { ...state, schedule: action.schedule };
  }
}

// --- Validation ---

function validate(job: JobDefinition): Record<string, string> {
  const errors: Record<string, string> = {};
  if (!job.name.trim()) {
    errors.name = "Name is required";
  }
  if (job.source.type === "Local" && !job.source.path.trim()) {
    errors.source = "Source path is required";
  }
  if (job.destination.type === "Local" && !job.destination.path.trim()) {
    errors.destination = "Destination path is required";
  }
  return errors;
}

// --- Component ---

interface JobFormProps {
  initialJob: JobDefinition;
  onSave: (job: JobDefinition) => Promise<void> | void;
  onCancel: () => void;
  title: string;
}

export function JobForm({ initialJob, onSave, onCancel, title }: JobFormProps) {
  const [job, dispatch] = useReducer(jobReducer, initialJob);
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [saving, setSaving] = useState(false);
  const autoTrailingSlash = useTrailingSlash();

  async function handleSubmit() {
    const validationErrors = validate(job);
    setErrors(validationErrors);
    if (Object.keys(validationErrors).length > 0) return;

    setSaving(true);
    try {
      await onSave({ ...job, updated_at: new Date().toISOString() });
    } catch (err) {
      setErrors({
        save: err instanceof Error ? err.message : String(err),
      });
    } finally {
      setSaving(false);
    }
  }

  const showSshConfig = job.ssh_config !== null;

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-bold">{title}</h2>
        <div className="flex items-center gap-2">
          {errors.save && (
            <p className="text-sm text-destructive">{errors.save}</p>
          )}
          <Button variant="outline" onClick={onCancel}>
            Cancel
          </Button>
          <Button onClick={handleSubmit} disabled={saving}>
            {saving ? "Saving..." : "Save"}
          </Button>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-2">
          <ScrollArea className="h-[calc(100vh-12rem)]">
            <Tabs defaultValue="general" className="pr-4">
              <TabsList>
                <TabsTrigger value="general">General</TabsTrigger>
                <TabsTrigger value="options">Options</TabsTrigger>
              </TabsList>

              <TabsContent value="general" className="space-y-6 mt-4">
                <JobFormGeneral
                  job={job}
                  onNameChange={(name) => dispatch({ type: "SET_NAME", name })}
                  onDescriptionChange={(description) =>
                    dispatch({ type: "SET_DESCRIPTION", description })
                  }
                  onEnabledChange={(enabled) =>
                    dispatch({ type: "SET_ENABLED", enabled })
                  }
                  onSourceChange={(source) =>
                    dispatch({ type: "SET_SOURCE", source })
                  }
                  onDestinationChange={(destination) =>
                    dispatch({ type: "SET_DESTINATION", destination })
                  }
                  errors={errors}
                  autoTrailingSlash={autoTrailingSlash}
                />
              </TabsContent>

              <TabsContent value="options" className="space-y-6 mt-4">
                <BackupModeField
                  value={job.backup_mode}
                  onChange={(mode) =>
                    dispatch({ type: "SET_BACKUP_MODE", mode })
                  }
                />
                <RsyncOptionsField
                  value={job.options}
                  onChange={(options) =>
                    dispatch({ type: "SET_OPTIONS", options })
                  }
                />
                {showSshConfig && (
                  <SshConfigField
                    value={job.ssh_config!}
                    onChange={(ssh_config) =>
                      dispatch({ type: "SET_SSH_CONFIG", ssh_config })
                    }
                  />
                )}
                <ScheduleField
                  value={job.schedule}
                  onChange={(schedule) =>
                    dispatch({ type: "SET_SCHEDULE", schedule })
                  }
                />
              </TabsContent>
            </Tabs>
          </ScrollArea>
        </div>

        <div className="lg:col-span-1">
          <div className="sticky top-0">
            <CommandPreview job={job} autoTrailingSlash={autoTrailingSlash} />
          </div>
        </div>
      </div>
    </div>
  );
}

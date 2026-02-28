import { useReducer, useState, useEffect } from "react";
import { useTrailingSlash } from "@/hooks/use-trailing-slash";
import { useNasAutoDetect } from "@/hooks/use-nas-auto-detect";
import { useShowFileHandlingOptions } from "@/hooks/use-show-file-handling-options";
import { useShowMetadataOptions } from "@/hooks/use-show-metadata-options";
import { useShowOutputOptions } from "@/hooks/use-show-output-options";
import { detectFilesystemType } from "@/lib/tauri";
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
import { CommandPreview } from "../command-preview";

// --- Reducer ---

type Action =
  | { type: "SET_NAME"; name: string }
  | { type: "SET_DESCRIPTION"; description: string | null }
  | { type: "SET_ENABLED"; enabled: boolean }
  | { type: "SET_SOURCE"; source: StorageLocation }
  | { type: "SET_DESTINATION"; destination: StorageLocation }
  | { type: "SET_BACKUP_MODE"; mode: JobDefinition["transfer"]["backup_mode"] }
  | { type: "SET_OPTIONS"; options: JobDefinition["options"] }
  | { type: "SET_SSH_CONFIG"; ssh_config: SshConfig }
  | { type: "SET_SCHEDULE"; schedule: ScheduleConfig | null }
  | { type: "ENABLE_NAS_MODE" };

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
      const ssh = needsSshConfig(action.source, state.transfer.destination)
        ? state.ssh_config ?? defaultSshConfig()
        : null;
      return { ...state, transfer: { ...state.transfer, source: action.source }, ssh_config: ssh };
    }
    case "SET_DESTINATION": {
      const ssh = needsSshConfig(state.transfer.source, action.destination)
        ? state.ssh_config ?? defaultSshConfig()
        : null;
      return { ...state, transfer: { ...state.transfer, destination: action.destination }, ssh_config: ssh };
    }
    case "SET_BACKUP_MODE":
      return { ...state, transfer: { ...state.transfer, backup_mode: action.mode } };
    case "SET_OPTIONS":
      return { ...state, options: action.options };
    case "SET_SSH_CONFIG":
      return { ...state, ssh_config: action.ssh_config };
    case "SET_SCHEDULE":
      return { ...state, schedule: action.schedule };
    case "ENABLE_NAS_MODE": {
      if (!state.options.file_handling.size_only) {
        return {
          ...state,
          options: {
            ...state.options,
            file_handling: { ...state.options.file_handling, size_only: true },
          },
        };
      }
      return state;
    }
  }
}

// --- Validation ---

function validate(job: JobDefinition): Record<string, string> {
  const errors: Record<string, string> = {};
  if (!job.name.trim()) {
    errors.name = "Name is required";
  }
  if (job.transfer.source.type === "Local" && !job.transfer.source.path.trim()) {
    errors.source = "Source path is required";
  }
  if (job.transfer.destination.type === "Local" && !job.transfer.destination.path.trim()) {
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
  const nasAutoDetect = useNasAutoDetect();
  const showFileHandling = useShowFileHandlingOptions();
  const showMetadata = useShowMetadataOptions();
  const showOutput = useShowOutputOptions();
  const [networkFs, setNetworkFs] = useState<{ location: "source" | "destination"; fsType: string } | null>(null);

  const NETWORK_FS_TYPES = ["smbfs", "cifs", "nfs", "nfs4", "afpfs"];

  useEffect(() => {
    let cancelled = false;
    const detect = async () => {
      if (!nasAutoDetect) {
        if (!cancelled) setNetworkFs(null);
        return;
      }
      if (job.transfer.source.type === "Local" && job.transfer.source.path) {
        try {
          const fsType = await detectFilesystemType(job.transfer.source.path);
          if (!cancelled && fsType && NETWORK_FS_TYPES.includes(fsType)) {
            setNetworkFs({ location: "source", fsType });
            dispatch({ type: "ENABLE_NAS_MODE" });
            return;
          }
        } catch { /* ignore */ }
      }
      if (job.transfer.destination.type === "Local" && job.transfer.destination.path) {
        try {
          const fsType = await detectFilesystemType(job.transfer.destination.path);
          if (!cancelled && fsType && NETWORK_FS_TYPES.includes(fsType)) {
            setNetworkFs({ location: "destination", fsType });
            dispatch({ type: "ENABLE_NAS_MODE" });
            return;
          }
        } catch { /* ignore */ }
      }
      if (!cancelled) setNetworkFs(null);
    };
    detect();
    return () => { cancelled = true; };
  }, [job.transfer.source, job.transfer.destination, nasAutoDetect]);

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
                  value={job.transfer.backup_mode}
                  onChange={(mode) =>
                    dispatch({ type: "SET_BACKUP_MODE", mode })
                  }
                />
                <RsyncOptionsField
                  value={job.options}
                  onChange={(options) =>
                    dispatch({ type: "SET_OPTIONS", options })
                  }
                  networkFs={networkFs}
                  showFileHandling={showFileHandling}
                  showMetadata={showMetadata}
                  showOutput={showOutput}
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

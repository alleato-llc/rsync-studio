import type { JobDefinition } from "@/types/job";

export function createDefaultJob(): JobDefinition {
  const now = new Date().toISOString();
  return {
    id: crypto.randomUUID(),
    name: "",
    description: null,
    source: { type: "Local", path: "" },
    destination: { type: "Local", path: "" },
    backup_mode: { type: "Mirror" },
    options: {
      archive: true,
      compress: false,
      verbose: true,
      delete: false,
      dry_run: false,
      partial: false,
      progress: true,
      human_readable: true,
      exclude_patterns: [],
      include_patterns: [],
      bandwidth_limit: null,
      custom_args: [],
      size_only: false,
    },
    ssh_config: null,
    schedule: null,
    enabled: true,
    created_at: now,
    updated_at: now,
  };
}

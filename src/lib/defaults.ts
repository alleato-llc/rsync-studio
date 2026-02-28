import type { JobDefinition } from "@/types/job";

export function createDefaultJob(): JobDefinition {
  const now = new Date().toISOString();
  return {
    id: crypto.randomUUID(),
    name: "",
    description: null,
    transfer: {
      source: { type: "Local", path: "" },
      destination: { type: "Local", path: "" },
      backup_mode: { type: "Mirror" },
    },
    options: {
      core_transfer: {
        archive: true,
        compress: false,
        partial: false,
        dry_run: false,
      },
      file_handling: {
        delete: false,
        size_only: false,
        checksum: false,
        update: false,
        whole_file: false,
        ignore_existing: false,
        one_file_system: false,
      },
      metadata: {
        hard_links: false,
        acls: false,
        xattrs: false,
        numeric_ids: false,
      },
      output: {
        verbose: true,
        progress: true,
        human_readable: true,
        stats: false,
        itemize_changes: false,
      },
      advanced: {
        exclude_patterns: [],
        include_patterns: [],
        bandwidth_limit: null,
        custom_args: [],
      },
    },
    ssh_config: null,
    schedule: null,
    enabled: true,
    created_at: now,
    updated_at: now,
  };
}

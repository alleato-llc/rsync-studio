import type { JobStatus, StorageLocation } from "@/types/job";

export function locationSummary(loc: StorageLocation): string {
  switch (loc.type) {
    case "Local":
      return loc.path || "(no path)";
    case "RemoteSsh":
      return `${loc.user}@${loc.host}:${loc.path}`;
    case "RemoteRsync":
      return `rsync://${loc.host}/${loc.module}`;
  }
}

export function statusBadgeVariant(status: JobStatus): "default" | "secondary" | "destructive" | "outline" {
  switch (status) {
    case "Running":
      return "default";
    case "Completed":
      return "secondary";
    case "Failed":
      return "destructive";
    case "Cancelled":
      return "outline";
    default:
      return "secondary";
  }
}

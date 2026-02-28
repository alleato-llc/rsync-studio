import type { JobDefinition, JobStatus } from "@/types/job";
import type { ProgressUpdate, LogLine } from "@/types/execution/progress";
import type { ItemizedChange } from "@/types/itemize";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { ArrowLeft, Square } from "lucide-react";
import { ProgressDisplay } from "./progress-display";
import { ItemizedChangesTable } from "./itemized-changes-table";
import { VirtualLogViewer } from "@/components/logs/virtual-log-viewer";
import { buildCommandString } from "@/lib/command-preview";
import { useTrailingSlash } from "@/hooks/use-trailing-slash";

interface ExecutionViewProps {
  job: JobDefinition;
  status: JobStatus;
  isDryRun?: boolean;
  progress: ProgressUpdate | null;
  logs: LogLine[];
  itemizedChanges?: ItemizedChange[];
  isTruncated?: boolean;
  logFilePath?: string | null;
  error: string | null;
  onCancel: () => void;
  onBack: () => void;
}

function statusBadgeVariant(status: JobStatus): "default" | "secondary" | "destructive" | "outline" {
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

export function ExecutionView({
  job,
  status,
  isDryRun,
  progress,
  logs,
  itemizedChanges,
  isTruncated,
  logFilePath,
  error,
  onCancel,
  onBack,
}: ExecutionViewProps) {
  const autoTrailingSlash = useTrailingSlash();

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <Button variant="ghost" size="icon" onClick={onBack}>
            <ArrowLeft className="h-4 w-4" />
          </Button>
          <div>
            <h2 className="text-2xl font-bold">{job.name}</h2>
            {job.description && (
              <p className="text-muted-foreground text-sm">{job.description}</p>
            )}
          </div>
        </div>
        <div className="flex items-center gap-2">
          <Badge variant={statusBadgeVariant(status)}>{status}</Badge>
          {status === "Running" && (
            <Button variant="destructive" size="sm" onClick={onCancel}>
              <Square className="h-3 w-3 mr-1" />
              Cancel
            </Button>
          )}
        </div>
      </div>

      <div className="space-y-2">
        <h3 className="text-sm font-medium">Command</h3>
        <pre className="text-xs font-mono whitespace-pre-wrap bg-muted/50 rounded-md border p-3">
          {buildCommandString(job, autoTrailingSlash)}
        </pre>
      </div>

      <ProgressDisplay progress={progress} status={status} />

      {error && (
        <div className="rounded-md bg-destructive/10 p-4 text-sm text-destructive">
          {error}
        </div>
      )}

      {isDryRun && itemizedChanges && (
        <ItemizedChangesTable
          changes={itemizedChanges}
          isStreaming={status === "Running"}
          isTruncated={isTruncated}
          logFilePath={logFilePath}
        />
      )}

      <div className="space-y-2">
        <div className="flex items-center gap-2">
          <h3 className="text-sm font-medium">Output</h3>
          {logFilePath && (
            <span className="text-xs text-muted-foreground font-mono truncate max-w-md" title={logFilePath}>
              {logFilePath}
            </span>
          )}
        </div>
        <VirtualLogViewer logs={logs} height={400} autoScroll={status === "Running"} />
      </div>

      {status !== "Running" && status !== "Idle" && (
        <div className="flex justify-end">
          <Button variant="outline" onClick={onBack}>
            Back to Jobs
          </Button>
        </div>
      )}
    </div>
  );
}

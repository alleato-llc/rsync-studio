import type { ProgressUpdate } from "@/types/progress";
import type { JobStatus } from "@/types/job";
import { Progress } from "@/components/ui/progress";

interface ProgressDisplayProps {
  progress: ProgressUpdate | null;
  status?: JobStatus;
}

export function ProgressDisplay({ progress, status }: ProgressDisplayProps) {
  if (!progress) {
    const isTerminal = status === "Completed" || status === "Failed" || status === "Cancelled";
    if (isTerminal) {
      const message =
        status === "Completed"
          ? "Sync complete."
          : status === "Failed"
            ? "Job failed before progress data was received."
            : "Job was cancelled.";
      return (
        <div className="space-y-2">
          <Progress value={100} />
          <p className="text-sm text-muted-foreground">{message}</p>
        </div>
      );
    }
    return (
      <div className="space-y-2">
        <Progress value={0} />
        <p className="text-sm text-muted-foreground">Waiting for progress data...</p>
      </div>
    );
  }

  const overallPercent =
    progress.files_total > 0
      ? ((progress.files_total - progress.files_remaining) / progress.files_total) * 100
      : progress.percentage;

  return (
    <div className="space-y-2">
      <Progress value={overallPercent} />
      <div className="flex items-center justify-between text-sm text-muted-foreground">
        <span>{Math.round(overallPercent)}%</span>
        <span>{progress.transfer_rate}</span>
        <span>{progress.elapsed}</span>
      </div>
      {progress.files_total > 0 && (
        <p className="text-xs text-muted-foreground">
          {progress.files_transferred} of {progress.files_total} files transferred
          {progress.files_remaining > 0 && ` (${progress.files_remaining} remaining)`}
        </p>
      )}
    </div>
  );
}

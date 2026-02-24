import { useState, useEffect, useRef } from "react";
import type { JobDefinition, JobStatus } from "@/types/job";
import type { ProgressUpdate, LogLine } from "@/types/progress";
import type { PreflightResult } from "@/types/validation";
import * as api from "@/lib/tauri";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Pencil, Trash2, ShieldCheck, ChevronDown, ChevronRight, FlaskConical, ExternalLink, Terminal } from "lucide-react";
import { JobRunButton } from "./job-run-button";
import { ScheduleBadge } from "./schedule-badge";
import { locationSummary, statusBadgeVariant } from "./job-utils";

interface JobTableProps {
  jobs: JobDefinition[];
  onEdit: (jobId: string) => void;
  onDelete: (job: JobDefinition) => void;
  onRun: (jobId: string) => void;
  onDryRun: (jobId: string) => void;
  onCancel: (jobId: string) => void;
  onViewExecution: (jobId: string) => void;
  getStatus: (jobId: string) => JobStatus;
  getLogs: (jobId: string) => LogLine[];
  getProgress: (jobId: string) => ProgressUpdate | null;
  getError: (jobId: string) => string | null;
}

function JobTableRow({
  job,
  status,
  logs,
  progress,
  executionError,
  onEdit,
  onDelete,
  onRun,
  onDryRun,
  onCancel,
  onViewExecution,
}: {
  job: JobDefinition;
  status: JobStatus;
  logs: LogLine[];
  progress: ProgressUpdate | null;
  executionError: string | null;
  onEdit: () => void;
  onDelete: () => void;
  onRun: () => void;
  onDryRun: () => void;
  onCancel: () => void;
  onViewExecution: () => void;
}) {
  const isRunning = status === "Running";
  const [preflight, setPreflight] = useState<PreflightResult | null>(null);
  const [preflightLoading, setPreflightLoading] = useState(false);
  const [preflightOpen, setPreflightOpen] = useState(false);
  const [logsOpen, setLogsOpen] = useState(false);
  const logEndRef = useRef<HTMLDivElement>(null);
  const prevStatusRef = useRef(status);

  const hasExecution = logs.length > 0 || executionError !== null || status === "Running";

  // Auto-open logs when job starts running
  useEffect(() => {
    if (status === "Running" && prevStatusRef.current !== "Running") {
      setLogsOpen(true);
    }
    prevStatusRef.current = status;
  }, [status]);

  // Auto-scroll logs
  useEffect(() => {
    if (logsOpen) {
      logEndRef.current?.scrollIntoView({ behavior: "smooth" });
    }
  }, [logs.length, logsOpen]);

  async function handlePreflight() {
    setPreflightLoading(true);
    setPreflight(null);
    setPreflightOpen(false);
    try {
      const result = await api.runPreflight(job.id);
      setPreflight(result);
      setPreflightOpen(true);
    } catch {
      setPreflight(null);
    } finally {
      setPreflightLoading(false);
    }
  }

  return (
    <>
      <tr className="border-b hover:bg-muted/50 transition-colors">
        <td className="px-4 py-3 text-sm font-medium">
          <div className="flex items-center gap-2">
            <span>{job.name}</span>
            {!job.enabled && (
              <Badge variant="secondary" className="text-xs">
                Disabled
              </Badge>
            )}
          </div>
        </td>
        <td className="px-4 py-3 text-sm text-muted-foreground truncate max-w-[200px]">
          {locationSummary(job.source)}
        </td>
        <td className="px-4 py-3 text-sm text-muted-foreground truncate max-w-[200px]">
          {locationSummary(job.destination)}
        </td>
        <td className="px-4 py-3">
          <Badge variant="outline" className="text-xs">
            {job.backup_mode.type}
          </Badge>
        </td>
        <td className="px-4 py-3">
          <ScheduleBadge schedule={job.schedule} />
        </td>
        <td className="px-4 py-3">
          {status !== "Idle" && (
            <Badge variant={statusBadgeVariant(status)} className="text-xs">
              {status}
            </Badge>
          )}
        </td>
        <td className="px-4 py-3">
          <div className="flex items-center gap-1">
            <Button
              variant="ghost"
              size="icon"
              className="h-8 w-8"
              onClick={handlePreflight}
              disabled={isRunning || preflightLoading}
              title="Preflight check"
            >
              <ShieldCheck className="h-4 w-4" />
            </Button>
            <Button
              variant="ghost"
              size="icon"
              className="h-8 w-8 text-amber-600"
              onClick={onDryRun}
              disabled={isRunning}
              title="Dry run"
            >
              <FlaskConical className="h-4 w-4" />
            </Button>
            <JobRunButton isRunning={isRunning} disabled={preflight !== null && !preflight.overall_pass} hidden={job.options.dry_run} onRun={onRun} onCancel={onCancel} />
            {status !== "Idle" && (
              <Button
                variant="ghost"
                size="icon"
                className="h-8 w-8"
                onClick={onViewExecution}
                title="View execution output"
              >
                <Terminal className="h-4 w-4" />
              </Button>
            )}
            <Button variant="ghost" size="icon" className="h-8 w-8" onClick={onEdit} disabled={isRunning}>
              <Pencil className="h-4 w-4" />
            </Button>
            <Button
              variant="ghost"
              size="icon"
              className="h-8 w-8 text-destructive"
              onClick={onDelete}
              disabled={isRunning}
            >
              <Trash2 className="h-4 w-4" />
            </Button>
          </div>
        </td>
      </tr>
      {preflight && (
        <tr className="border-b bg-muted/30">
          <td colSpan={7} className="px-4 py-2">
            <button
              className="flex items-center gap-1 text-sm font-medium mb-1"
              onClick={() => setPreflightOpen(!preflightOpen)}
            >
              {preflightOpen ? (
                <ChevronDown className="h-3 w-3" />
              ) : (
                <ChevronRight className="h-3 w-3" />
              )}
              <span>
                {preflight.overall_pass ? "All checks passed" : "Some checks failed"}
              </span>
              <Badge
                variant={preflight.overall_pass ? "secondary" : "destructive"}
                className="text-xs ml-1"
              >
                {preflight.overall_pass ? "Pass" : "Fail"}
              </Badge>
            </button>
            {preflightOpen &&
              preflight.checks.map((check, i) => (
                <div key={i} className="flex items-start gap-2 text-xs text-muted-foreground ml-4">
                  <span className="shrink-0 mt-0.5">
                    {check.passed ? "\u2713" : "\u2717"}
                  </span>
                  <span>{check.message}</span>
                </div>
              ))}
          </td>
        </tr>
      )}
      {hasExecution && (
        <tr className="border-b bg-muted/30">
          <td colSpan={7} className="px-4 py-2">
            <button
              className="flex items-center gap-1 text-sm font-medium mb-1"
              onClick={() => setLogsOpen(!logsOpen)}
            >
              {logsOpen ? (
                <ChevronDown className="h-3 w-3" />
              ) : (
                <ChevronRight className="h-3 w-3" />
              )}
              <span>Execution Output</span>
              <Badge variant={statusBadgeVariant(status)} className="text-xs ml-1">
                {status}
              </Badge>
            </button>
            {logsOpen && (
              <div className="space-y-2">
                {progress && (
                  <div className="text-xs text-muted-foreground">
                    {Math.round(progress.percentage)}% &middot; {progress.transfer_rate} &middot; {progress.elapsed}
                  </div>
                )}
                <pre className="text-xs font-mono whitespace-pre-wrap bg-muted/50 rounded-md border p-2 max-h-[200px] overflow-y-auto">
                  {logs.slice(-50).map((log, i) => (
                    <div key={i} className={log.is_stderr ? "text-destructive" : ""}>
                      {log.line}
                    </div>
                  ))}
                  <div ref={logEndRef} />
                </pre>
                {executionError && (
                  <div className="rounded-md bg-destructive/10 p-2 text-xs text-destructive">
                    {executionError}
                  </div>
                )}
                <Button
                  variant="link"
                  size="sm"
                  className="h-auto p-0 text-xs"
                  onClick={onViewExecution}
                >
                  <ExternalLink className="h-3 w-3 mr-1" />
                  View full output
                </Button>
              </div>
            )}
          </td>
        </tr>
      )}
    </>
  );
}

export function JobTable({
  jobs,
  onEdit,
  onDelete,
  onRun,
  onDryRun,
  onCancel,
  onViewExecution,
  getStatus,
  getLogs,
  getProgress,
  getError,
}: JobTableProps) {
  return (
    <div className="overflow-x-auto rounded-md border">
      <table className="w-full">
        <thead>
          <tr className="border-b bg-muted/50">
            <th className="px-4 py-3 text-left text-sm font-medium text-muted-foreground">Name</th>
            <th className="px-4 py-3 text-left text-sm font-medium text-muted-foreground">Source</th>
            <th className="px-4 py-3 text-left text-sm font-medium text-muted-foreground">Destination</th>
            <th className="px-4 py-3 text-left text-sm font-medium text-muted-foreground">Mode</th>
            <th className="px-4 py-3 text-left text-sm font-medium text-muted-foreground">Schedule</th>
            <th className="px-4 py-3 text-left text-sm font-medium text-muted-foreground">Status</th>
            <th className="px-4 py-3 text-left text-sm font-medium text-muted-foreground">Actions</th>
          </tr>
        </thead>
        <tbody>
          {jobs.map((job) => (
            <JobTableRow
              key={job.id}
              job={job}
              status={getStatus(job.id)}
              logs={getLogs(job.id)}
              progress={getProgress(job.id)}
              executionError={getError(job.id)}
              onEdit={() => onEdit(job.id)}
              onDelete={() => onDelete(job)}
              onRun={() => onRun(job.id)}
              onDryRun={() => onDryRun(job.id)}
              onCancel={() => onCancel(job.id)}
              onViewExecution={() => onViewExecution(job.id)}
            />
          ))}
        </tbody>
      </table>
    </div>
  );
}

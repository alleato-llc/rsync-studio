import { useState } from "react";
import type { JobDefinition, JobStatus } from "@/types/job";
import type { PreflightResult } from "@/types/validation";
import * as api from "@/lib/tauri";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Pencil, Trash2, ArrowRight, ShieldCheck, FlaskConical, Terminal } from "lucide-react";
import { JobRunButton } from "./job-run-button";
import { ScheduleBadge } from "./schedule-badge";
import { locationSummary, statusBadgeVariant } from "./job-utils";

interface JobCardProps {
  job: JobDefinition;
  status: JobStatus;
  onEdit: () => void;
  onDelete: () => void;
  onRun: () => void;
  onDryRun: () => void;
  onCancel: () => void;
  onViewExecution: () => void;
}

export function JobCard({ job, status, onEdit, onDelete, onRun, onDryRun, onCancel, onViewExecution }: JobCardProps) {
  const isRunning = status === "Running";
  const [preflight, setPreflight] = useState<PreflightResult | null>(null);
  const [preflightLoading, setPreflightLoading] = useState(false);

  async function handlePreflight() {
    setPreflightLoading(true);
    setPreflight(null);
    try {
      const result = await api.runPreflight(job.id);
      setPreflight(result);
    } catch {
      setPreflight(null);
    } finally {
      setPreflightLoading(false);
    }
  }

  return (
    <Card className={isRunning ? "border-primary/50" : undefined}>
      <CardHeader className="pb-2">
        <div className="space-y-2">
          <div className="flex items-center gap-2">
            <CardTitle className="text-base truncate">{job.name}</CardTitle>
            {!job.enabled && (
              <Badge variant="secondary" className="text-xs shrink-0">
                Disabled
              </Badge>
            )}
          </div>
          {job.description && (
            <CardDescription className="truncate">
              {job.description}
            </CardDescription>
          )}
          <div className="flex flex-wrap gap-1">
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
              onClick={(e) => { e.stopPropagation(); onDryRun(); }}
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
        </div>
      </CardHeader>
      <CardContent>
        <div className="flex items-center gap-2 text-sm text-muted-foreground">
          <span className="truncate max-w-[40%]">{locationSummary(job.source)}</span>
          <ArrowRight className="h-3 w-3 shrink-0" />
          <span className="truncate max-w-[40%]">
            {locationSummary(job.destination)}
          </span>
        </div>
        <div className="mt-2 flex flex-wrap gap-2">
          <Badge variant="outline" className="text-xs">
            {job.backup_mode.type}
          </Badge>
          <ScheduleBadge schedule={job.schedule} />
          {status !== "Idle" && (
            <Badge variant={statusBadgeVariant(status)} className="text-xs">
              {status}
            </Badge>
          )}
        </div>
        {preflight && (
          <div className="mt-3 space-y-1 border-t pt-2">
            <div className="flex items-center gap-2 text-sm font-medium">
              <span>
                {preflight.overall_pass ? "All checks passed" : "Some checks failed"}
              </span>
              <Badge
                variant={preflight.overall_pass ? "secondary" : "destructive"}
                className="text-xs"
              >
                {preflight.overall_pass ? "Pass" : "Fail"}
              </Badge>
            </div>
            {preflight.checks.map((check, i) => (
              <div key={i} className="flex items-start gap-2 text-xs text-muted-foreground">
                <span className="shrink-0 mt-0.5">
                  {check.passed ? "\u2713" : "\u2717"}
                </span>
                <span>{check.message}</span>
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}

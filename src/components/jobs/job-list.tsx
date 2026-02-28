import { useState } from "react";
import type { JobDefinition, JobStatus } from "@/types/job";
import type { ProgressUpdate, LogLine } from "@/types/execution/progress";
import { Button } from "@/components/ui/button";
import { Plus, LayoutGrid, List } from "lucide-react";
import { JobCard } from "./job-card";
import { JobTable } from "./job-table";

type ViewMode = "cards" | "table";

interface JobListProps {
  jobs: JobDefinition[];
  loading: boolean;
  error: string | null;
  onCreate: () => void;
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

export function JobList({
  jobs,
  loading,
  error,
  onCreate,
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
}: JobListProps) {
  const [viewMode, setViewMode] = useState<ViewMode>("table");

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold">Jobs</h2>
          <p className="text-muted-foreground mt-1">
            Create and manage your rsync backup jobs.
          </p>
        </div>
        <div className="flex items-center gap-2">
          <div className="flex items-center border rounded-md">
            <Button
              variant={viewMode === "cards" ? "secondary" : "ghost"}
              size="icon"
              className="h-8 w-8 rounded-r-none"
              onClick={() => setViewMode("cards")}
              title="Card view"
            >
              <LayoutGrid className="h-4 w-4" />
            </Button>
            <Button
              variant={viewMode === "table" ? "secondary" : "ghost"}
              size="icon"
              className="h-8 w-8 rounded-l-none"
              onClick={() => setViewMode("table")}
              title="Table view"
            >
              <List className="h-4 w-4" />
            </Button>
          </div>
          <Button onClick={onCreate}>
            <Plus className="h-4 w-4 mr-2" />
            Create Job
          </Button>
        </div>
      </div>

      {error && (
        <div className="rounded-md bg-destructive/10 p-4 text-sm text-destructive">
          {error}
        </div>
      )}

      {loading ? (
        <div className="text-center py-12 text-muted-foreground">
          Loading jobs...
        </div>
      ) : jobs.length === 0 ? (
        <div className="text-center py-12">
          <p className="text-muted-foreground mb-4">
            No backup jobs yet. Create your first job to get started.
          </p>
          <Button onClick={onCreate}>
            <Plus className="h-4 w-4 mr-2" />
            Create Job
          </Button>
        </div>
      ) : viewMode === "cards" ? (
        <div className="grid gap-4 sm:grid-cols-1 lg:grid-cols-2">
          {jobs.map((job) => (
            <JobCard
              key={job.id}
              job={job}
              status={getStatus(job.id)}
              onEdit={() => onEdit(job.id)}
              onDelete={() => onDelete(job)}
              onRun={() => onRun(job.id)}
              onDryRun={() => onDryRun(job.id)}
              onCancel={() => onCancel(job.id)}
              onViewExecution={() => onViewExecution(job.id)}
            />
          ))}
        </div>
      ) : (
        <JobTable
          jobs={jobs}
          onEdit={onEdit}
          onDelete={onDelete}
          onRun={onRun}
          onDryRun={onDryRun}
          onCancel={onCancel}
          onViewExecution={onViewExecution}
          getStatus={getStatus}
          getLogs={getLogs}
          getProgress={getProgress}
          getError={getError}
        />
      )}
    </div>
  );
}

import { useState } from "react";
import type { JobDefinition } from "@/types/job";
import { useJobs } from "@/hooks/use-jobs";
import { useJobExecution } from "@/hooks/use-job-execution";
import { createDefaultJob } from "@/lib/defaults";
import { JobList } from "@/components/jobs/job-list";
import { JobForm } from "@/components/jobs/form/job-form";
import { DeleteJobDialog } from "@/components/jobs/delete-job-dialog";
import { ExecutionView } from "@/components/jobs/execution/execution-view";

type View =
  | { view: "list" }
  | { view: "create" }
  | { view: "edit"; jobId: string }
  | { view: "running"; jobId: string };

export function JobsPage() {
  const { jobs, loading, error, handleCreate, handleUpdate, handleDelete } =
    useJobs();
  const execution = useJobExecution();
  const [currentView, setCurrentView] = useState<View>({ view: "list" });
  const [deleteTarget, setDeleteTarget] = useState<JobDefinition | null>(null);

  async function onSaveNew(job: JobDefinition) {
    await handleCreate(job);
    setCurrentView({ view: "list" });
  }

  async function onSaveEdit(job: JobDefinition) {
    await handleUpdate(job);
    setCurrentView({ view: "list" });
  }

  async function onConfirmDelete() {
    if (deleteTarget) {
      await handleDelete(deleteTarget.id);
      setDeleteTarget(null);
    }
  }

  function handleRun(jobId: string) {
    execution.runJob(jobId);
  }

  function handleDryRun(jobId: string) {
    execution.runDryRun(jobId);
  }

  if (currentView.view === "create") {
    return (
      <JobForm
        title="Create Job"
        initialJob={createDefaultJob()}
        onSave={onSaveNew}
        onCancel={() => setCurrentView({ view: "list" })}
      />
    );
  }

  if (currentView.view === "edit") {
    const job = jobs.find((j) => j.id === currentView.jobId);
    if (!job) {
      setCurrentView({ view: "list" });
      return null;
    }
    return (
      <JobForm
        title="Edit Job"
        initialJob={job}
        onSave={onSaveEdit}
        onCancel={() => setCurrentView({ view: "list" })}
      />
    );
  }

  if (currentView.view === "running") {
    const job = jobs.find((j) => j.id === currentView.jobId);
    if (!job) {
      setCurrentView({ view: "list" });
      return null;
    }
    return (
      <ExecutionView
        job={job}
        status={execution.getStatus(job.id)}
        isDryRun={execution.getIsDryRun(job.id)}
        progress={execution.getProgress(job.id)}
        logs={execution.getLogs(job.id)}
        itemizedChanges={execution.getItemizedChanges(job.id)}
        isTruncated={execution.getIsTruncated(job.id)}
        logFilePath={execution.getLogFilePath(job.id)}
        error={execution.getError(job.id)}
        onCancel={() => execution.cancelJob(job.id)}
        onBack={() => setCurrentView({ view: "list" })}
      />
    );
  }

  return (
    <>
      <JobList
        jobs={jobs}
        loading={loading}
        error={error}
        onCreate={() => setCurrentView({ view: "create" })}
        onEdit={(jobId) => setCurrentView({ view: "edit", jobId })}
        onDelete={(job) => setDeleteTarget(job)}
        onRun={handleRun}
        onDryRun={handleDryRun}
        onCancel={(jobId) => execution.cancelJob(jobId)}
        onViewExecution={(jobId) => setCurrentView({ view: "running", jobId })}
        getStatus={(jobId) => execution.getStatus(jobId)}
        getLogs={(jobId) => execution.getLogs(jobId)}
        getProgress={(jobId) => execution.getProgress(jobId)}
        getError={(jobId) => execution.getError(jobId)}
      />
      <DeleteJobDialog
        open={deleteTarget !== null}
        jobName={deleteTarget?.name ?? ""}
        onConfirm={onConfirmDelete}
        onCancel={() => setDeleteTarget(null)}
      />
    </>
  );
}

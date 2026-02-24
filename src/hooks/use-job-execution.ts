import { useCallback, useEffect, useRef, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { ProgressUpdate, LogLine, JobStatusEvent } from "@/types/progress";
import type { JobStatus } from "@/types/job";
import { executeJob as invokeExecute, executeDryRun as invokeDryRun, cancelJob as invokeCancel, getRunningJobs } from "@/lib/tauri";

interface JobExecutionState {
  status: JobStatus;
  invocationId: string | null;
  progress: ProgressUpdate | null;
  logs: LogLine[];
  error: string | null;
}

const initialState: JobExecutionState = {
  status: "Idle",
  invocationId: null,
  progress: null,
  logs: [],
  error: null,
};

export function useJobExecution() {
  const [jobs, setJobs] = useState<Map<string, JobExecutionState>>(new Map());
  const unlistenRefs = useRef<UnlistenFn[]>([]);

  const getOrDefault = useCallback(
    (jobId: string): JobExecutionState => jobs.get(jobId) ?? { ...initialState },
    [jobs]
  );

  const updateJob = useCallback((jobId: string, update: Partial<JobExecutionState>) => {
    setJobs((prev) => {
      const next = new Map(prev);
      const current = next.get(jobId) ?? { ...initialState };
      next.set(jobId, { ...current, ...update });
      return next;
    });
  }, []);

  useEffect(() => {
    // Subscribe to Tauri events
    const setupListeners = async () => {
      const unlistenLog = await listen<LogLine>("job-log", (event) => {
        const log = event.payload;
        setJobs((prev) => {
          const next = new Map(prev);
          const jobEntries = Array.from(next.entries());
          const match = jobEntries.find(
            ([, state]) => state.invocationId === log.invocation_id
          );
          if (match) {
            const [jobId, state] = match;
            next.set(jobId, {
              ...state,
              logs: [...state.logs, log],
            });
          }
          return next;
        });
      });

      const unlistenProgress = await listen<ProgressUpdate>("job-progress", (event) => {
        const progress = event.payload;
        setJobs((prev) => {
          const next = new Map(prev);
          const jobEntries = Array.from(next.entries());
          const match = jobEntries.find(
            ([, state]) => state.invocationId === progress.invocation_id
          );
          if (match) {
            const [jobId, state] = match;
            next.set(jobId, { ...state, progress });
          }
          return next;
        });
      });

      const unlistenStatus = await listen<JobStatusEvent>("job-status", (event) => {
        const status = event.payload;
        updateJob(status.job_id, {
          status: status.status,
          invocationId: status.invocation_id,
          error: status.error_message ?? null,
        });
      });

      unlistenRefs.current = [unlistenLog, unlistenProgress, unlistenStatus];
    };

    setupListeners();

    // Load initially running jobs
    getRunningJobs().then((ids) => {
      for (const id of ids) {
        updateJob(id, { status: "Running" });
      }
    });

    return () => {
      for (const unlisten of unlistenRefs.current) {
        unlisten();
      }
    };
  }, [updateJob]);

  const runJob = useCallback(
    async (jobId: string) => {
      try {
        const invocationId = await invokeExecute(jobId);
        updateJob(jobId, {
          status: "Running",
          invocationId,
          progress: null,
          logs: [],
          error: null,
        });
      } catch (err) {
        updateJob(jobId, {
          status: "Failed",
          error: err instanceof Error ? err.message : String(err),
        });
      }
    },
    [updateJob]
  );

  const runDryRun = useCallback(
    async (jobId: string) => {
      try {
        const invocationId = await invokeDryRun(jobId);
        updateJob(jobId, {
          status: "Running",
          invocationId,
          progress: null,
          logs: [],
          error: null,
        });
      } catch (err) {
        updateJob(jobId, {
          status: "Failed",
          error: err instanceof Error ? err.message : String(err),
        });
      }
    },
    [updateJob]
  );

  const cancelJobById = useCallback(
    async (jobId: string) => {
      try {
        await invokeCancel(jobId);
      } catch (err) {
        updateJob(jobId, {
          error: err instanceof Error ? err.message : String(err),
        });
      }
    },
    [updateJob]
  );

  const isRunning = useCallback(
    (jobId: string): boolean => getOrDefault(jobId).status === "Running",
    [getOrDefault]
  );

  const getProgress = useCallback(
    (jobId: string): ProgressUpdate | null => getOrDefault(jobId).progress,
    [getOrDefault]
  );

  const getLogs = useCallback(
    (jobId: string): LogLine[] => getOrDefault(jobId).logs,
    [getOrDefault]
  );

  const getStatus = useCallback(
    (jobId: string): JobStatus => getOrDefault(jobId).status,
    [getOrDefault]
  );

  const getError = useCallback(
    (jobId: string): string | null => getOrDefault(jobId).error,
    [getOrDefault]
  );

  const getInvocationId = useCallback(
    (jobId: string): string | null => getOrDefault(jobId).invocationId,
    [getOrDefault]
  );

  return {
    runJob,
    runDryRun,
    cancelJob: cancelJobById,
    isRunning,
    getProgress,
    getLogs,
    getStatus,
    getError,
    getInvocationId,
  };
}

import { useCallback, useEffect, useRef, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { ProgressUpdate, LogLine, JobStatusEvent } from "@/types/execution/progress";
import type { ItemizedChange } from "@/types/itemize";
import type { JobStatus } from "@/types/job";
import { executeJob as invokeExecute, executeDryRun as invokeDryRun, cancelJob as invokeCancel, getRunningJobs, getMaxItemizedChanges, getLogDirectory } from "@/lib/tauri";

const MAX_LOG_LINES = 10_000;

interface JobExecutionState {
  status: JobStatus;
  invocationId: string | null;
  isDryRun: boolean;
  progress: ProgressUpdate | null;
  logs: LogLine[];
  itemizedChanges: ItemizedChange[];
  isTruncated: boolean;
  logFilePath: string | null;
  error: string | null;
}

const initialState: JobExecutionState = {
  status: "Idle",
  invocationId: null,
  isDryRun: false,
  progress: null,
  logs: [],
  itemizedChanges: [],
  isTruncated: false,
  logFilePath: null,
  error: null,
};

export function useJobExecution() {
  const [jobs, setJobs] = useState<Map<string, JobExecutionState>>(new Map());
  const unlistenRefs = useRef<UnlistenFn[]>([]);
  const maxItemizedRef = useRef(50_000);

  useEffect(() => {
    getMaxItemizedChanges().then((v) => { maxItemizedRef.current = v; }).catch(console.error);
  }, []);

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
    let cancelled = false;

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
            const newLogs = [...state.logs, log];
            next.set(jobId, {
              ...state,
              logs: newLogs.length > MAX_LOG_LINES
                ? newLogs.slice(newLogs.length - MAX_LOG_LINES)
                : newLogs,
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

      const unlistenItemize = await listen<{ invocation_id: string; change: ItemizedChange }>("job-itemized-change", (event) => {
        const { invocation_id, change } = event.payload;
        setJobs((prev) => {
          const next = new Map(prev);
          const jobEntries = Array.from(next.entries());
          const match = jobEntries.find(
            ([, state]) => state.invocationId === invocation_id
          );
          if (match) {
            const [jobId, state] = match;
            const maxItems = maxItemizedRef.current;
            if (state.itemizedChanges.length >= maxItems) {
              if (!state.isTruncated) {
                next.set(jobId, { ...state, isTruncated: true });
              }
              return next;
            }
            next.set(jobId, {
              ...state,
              itemizedChanges: [...state.itemizedChanges, change],
            });
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

      if (cancelled) {
        unlistenLog();
        unlistenProgress();
        unlistenItemize();
        unlistenStatus();
        return;
      }

      unlistenRefs.current = [unlistenLog, unlistenProgress, unlistenItemize, unlistenStatus];
    };

    setupListeners();

    // Load initially running jobs
    getRunningJobs().then((ids) => {
      if (cancelled) return;
      for (const id of ids) {
        updateJob(id, { status: "Running" });
      }
    });

    return () => {
      cancelled = true;
      for (const unlisten of unlistenRefs.current) {
        unlisten();
      }
    };
  }, [updateJob]);

  const runJob = useCallback(
    async (jobId: string) => {
      updateJob(jobId, {
        status: "Running",
        invocationId: null,
        isDryRun: false,
        progress: null,
        logs: [],
        itemizedChanges: [],
        isTruncated: false,
        logFilePath: null,
        error: null,
      });
      try {
        const invocationId = await invokeExecute(jobId);
        const logDir = await getLogDirectory();
        updateJob(jobId, { invocationId, logFilePath: `${logDir}/${invocationId}.log` });
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
      updateJob(jobId, {
        status: "Running",
        invocationId: null,
        isDryRun: true,
        progress: null,
        logs: [],
        itemizedChanges: [],
        isTruncated: false,
        logFilePath: null,
        error: null,
      });
      try {
        const invocationId = await invokeDryRun(jobId);
        const logDir = await getLogDirectory();
        updateJob(jobId, { invocationId, logFilePath: `${logDir}/${invocationId}.log` });
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

  const getItemizedChanges = useCallback(
    (jobId: string): ItemizedChange[] => getOrDefault(jobId).itemizedChanges,
    [getOrDefault]
  );

  const getIsTruncated = useCallback(
    (jobId: string): boolean => getOrDefault(jobId).isTruncated,
    [getOrDefault]
  );

  const getLogFilePath = useCallback(
    (jobId: string): string | null => getOrDefault(jobId).logFilePath,
    [getOrDefault]
  );

  const getIsDryRun = useCallback(
    (jobId: string): boolean => getOrDefault(jobId).isDryRun,
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
    getItemizedChanges,
    getIsTruncated,
    getLogFilePath,
    getIsDryRun,
  };
}

import { useState, useEffect, useCallback } from "react";
import type { JobDefinition } from "@/types/job";
import * as api from "@/lib/tauri";

interface UseJobsResult {
  jobs: JobDefinition[];
  loading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
  handleCreate: (job: JobDefinition) => Promise<JobDefinition>;
  handleUpdate: (job: JobDefinition) => Promise<JobDefinition>;
  handleDelete: (id: string) => Promise<void>;
}

export function useJobs(): UseJobsResult {
  const [jobs, setJobs] = useState<JobDefinition[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await api.listJobs();
      setJobs(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const handleCreate = useCallback(
    async (job: JobDefinition): Promise<JobDefinition> => {
      const created = await api.createJob(job);
      await refresh();
      return created;
    },
    [refresh]
  );

  const handleUpdate = useCallback(
    async (job: JobDefinition): Promise<JobDefinition> => {
      const updated = await api.updateJob(job);
      await refresh();
      return updated;
    },
    [refresh]
  );

  const handleDelete = useCallback(
    async (id: string): Promise<void> => {
      await api.deleteJob(id);
      await refresh();
    },
    [refresh]
  );

  return { jobs, loading, error, refresh, handleCreate, handleUpdate, handleDelete };
}

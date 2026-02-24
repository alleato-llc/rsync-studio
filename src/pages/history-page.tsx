import { useState, useEffect, useCallback } from "react";
import { Trash2, FileText, X } from "lucide-react";
import type { JobDefinition } from "@/types/job";
import type { BackupInvocation, SnapshotRecord } from "@/types/backup";
import * as api from "@/lib/tauri";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { ScrollArea } from "@/components/ui/scroll-area";
import { HistoricalLogViewer } from "@/components/logs/historical-log-viewer";

function statusVariant(
  status: string
): "default" | "secondary" | "destructive" | "outline" {
  switch (status) {
    case "Succeeded":
      return "secondary";
    case "Failed":
      return "destructive";
    case "Cancelled":
      return "outline";
    case "Running":
      return "default";
    default:
      return "secondary";
  }
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${units[i]}`;
}

function formatDate(iso: string): string {
  return new Date(iso).toLocaleString();
}

function formatDuration(start: string, end: string | null): string {
  if (!end) return "Running...";
  const ms = new Date(end).getTime() - new Date(start).getTime();
  const secs = Math.floor(ms / 1000);
  if (secs < 60) return `${secs}s`;
  const mins = Math.floor(secs / 60);
  const remainSecs = secs % 60;
  return `${mins}m ${remainSecs}s`;
}

export function HistoryPage() {
  const [jobs, setJobs] = useState<JobDefinition[]>([]);
  const [selectedJobId, setSelectedJobId] = useState<string | null>(null);
  const [invocations, setInvocations] = useState<BackupInvocation[]>([]);
  const [snapshots, setSnapshots] = useState<SnapshotRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [tab, setTab] = useState<"invocations" | "snapshots">("invocations");
  const [logFilePath, setLogFilePath] = useState<string | null>(null);
  const [viewingLogId, setViewingLogId] = useState<string | null>(null);

  useEffect(() => {
    api.listJobs().then((j) => {
      setJobs(j);
      if (j.length > 0 && !selectedJobId) {
        setSelectedJobId(j[0].id);
      }
      setLoading(false);
    });
  }, []);

  const loadHistory = useCallback(async (jobId: string) => {
    const [inv, snaps] = await Promise.all([
      api.getJobHistory(jobId, 50),
      api.listSnapshots(jobId),
    ]);
    setInvocations(inv);
    setSnapshots(snaps);
  }, []);

  useEffect(() => {
    if (selectedJobId) {
      loadHistory(selectedJobId);
    }
  }, [selectedJobId, loadHistory]);

  async function handleDeleteInvocation(invId: string) {
    if (!confirm("Delete this invocation? This will also remove its log file and statistics.")) {
      return;
    }
    try {
      await api.deleteInvocation(invId);
      if (selectedJobId) {
        await loadHistory(selectedJobId);
      }
      if (viewingLogId === invId) {
        setLogFilePath(null);
        setViewingLogId(null);
      }
    } catch (err) {
      console.error("Failed to delete invocation:", err);
    }
  }

  async function handleClearAllHistory() {
    if (!selectedJobId) return;
    if (
      !confirm(
        "Delete ALL invocations for this job? This will also remove log files and statistics."
      )
    ) {
      return;
    }
    try {
      await api.deleteInvocationsForJob(selectedJobId);
      setInvocations([]);
      setLogFilePath(null);
      setViewingLogId(null);
    } catch (err) {
      console.error("Failed to clear history:", err);
    }
  }

  function handleViewLog(inv: BackupInvocation) {
    if (!inv.log_file_path) return;
    setViewingLogId(inv.id);
    setLogFilePath(inv.log_file_path);
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center h-48">
        <p className="text-muted-foreground">Loading...</p>
      </div>
    );
  }

  if (jobs.length === 0) {
    return (
      <div>
        <h2 className="text-2xl font-bold">History</h2>
        <p className="text-muted-foreground mt-2">
          No jobs yet. Create a job first to see history.
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-bold">History</h2>
        <div className="flex items-center gap-2">
          {tab === "invocations" && invocations.length > 0 && (
            <Button
              variant="outline"
              size="sm"
              onClick={handleClearAllHistory}
              className="text-destructive"
            >
              <Trash2 className="h-4 w-4 mr-1" />
              Clear All
            </Button>
          )}
          <Select
            value={selectedJobId ?? undefined}
            onValueChange={setSelectedJobId}
          >
            <SelectTrigger className="w-64">
              <SelectValue placeholder="Select a job" />
            </SelectTrigger>
            <SelectContent>
              {jobs.map((job) => (
                <SelectItem key={job.id} value={job.id}>
                  {job.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      </div>

      <div className="flex gap-2">
        <Button
          variant={tab === "invocations" ? "default" : "outline"}
          size="sm"
          onClick={() => setTab("invocations")}
        >
          Invocations ({invocations.length})
        </Button>
        <Button
          variant={tab === "snapshots" ? "default" : "outline"}
          size="sm"
          onClick={() => setTab("snapshots")}
        >
          Snapshots ({snapshots.length})
        </Button>
      </div>

      {/* Log viewer */}
      {logFilePath !== null && (
        <Card>
          <CardHeader className="pb-2">
            <div className="flex items-center justify-between">
              <CardTitle className="text-sm font-medium">Log Output</CardTitle>
              <Button
                variant="ghost"
                size="icon"
                onClick={() => {
                  setLogFilePath(null);
                  setViewingLogId(null);
                }}
              >
                <X className="h-4 w-4" />
              </Button>
            </div>
          </CardHeader>
          <CardContent>
            <HistoricalLogViewer filePath={logFilePath} height={256} />
          </CardContent>
        </Card>
      )}

      <ScrollArea className="h-[calc(100vh-14rem)]">
        {tab === "invocations" && (
          <div className="space-y-3 pr-4">
            {invocations.length === 0 ? (
              <p className="text-muted-foreground text-sm">
                No invocations yet for this job.
              </p>
            ) : (
              invocations.map((inv) => (
                <Card key={inv.id}>
                  <CardHeader className="pb-2">
                    <div className="flex items-center justify-between">
                      <CardTitle className="text-sm font-medium">
                        {formatDate(inv.started_at)}
                      </CardTitle>
                      <div className="flex items-center gap-2">
                        {inv.log_file_path && (
                          <Button
                            variant="ghost"
                            size="icon"
                            className="h-7 w-7"
                            title="View log"
                            onClick={() => handleViewLog(inv)}
                          >
                            <FileText className="h-3.5 w-3.5" />
                          </Button>
                        )}
                        <Button
                          variant="ghost"
                          size="icon"
                          className="h-7 w-7 text-destructive"
                          title="Delete invocation"
                          onClick={() => handleDeleteInvocation(inv.id)}
                        >
                          <Trash2 className="h-3.5 w-3.5" />
                        </Button>
                        <Badge variant="outline" className="text-xs">
                          {inv.trigger}
                        </Badge>
                        <Badge
                          variant={statusVariant(inv.status)}
                          className="text-xs"
                        >
                          {inv.status}
                        </Badge>
                      </div>
                    </div>
                    {inv.finished_at && (
                      <CardDescription className="text-xs">
                        Duration: {formatDuration(inv.started_at, inv.finished_at)}
                        {inv.exit_code !== null && ` | Exit code: ${inv.exit_code}`}
                      </CardDescription>
                    )}
                  </CardHeader>
                  <CardContent>
                    <div className="flex gap-4 text-xs text-muted-foreground">
                      <span>
                        Files: {inv.files_transferred}
                        {inv.total_files > 0 && `/${inv.total_files}`}
                      </span>
                      <span>Transferred: {formatBytes(inv.bytes_transferred)}</span>
                      {inv.snapshot_path && (
                        <span className="truncate max-w-[200px]" title={inv.snapshot_path}>
                          Snapshot: {inv.snapshot_path}
                        </span>
                      )}
                    </div>
                    <div className="mt-1">
                      <code className="text-xs text-muted-foreground break-all">
                        {inv.command_executed}
                      </code>
                    </div>
                  </CardContent>
                </Card>
              ))
            )}
          </div>
        )}

        {tab === "snapshots" && (
          <div className="space-y-3 pr-4">
            {snapshots.length === 0 ? (
              <p className="text-muted-foreground text-sm">
                No snapshots yet for this job. Snapshots are created when running jobs with Snapshot backup mode.
              </p>
            ) : (
              snapshots.map((snap) => (
                <Card key={snap.id}>
                  <CardHeader className="pb-2">
                    <div className="flex items-center justify-between">
                      <CardTitle className="text-sm font-medium truncate" title={snap.snapshot_path}>
                        {snap.snapshot_path}
                      </CardTitle>
                      <div className="flex items-center gap-2">
                        {snap.is_latest && (
                          <Badge variant="default" className="text-xs">
                            Latest
                          </Badge>
                        )}
                      </div>
                    </div>
                    <CardDescription className="text-xs">
                      {formatDate(snap.created_at)}
                    </CardDescription>
                  </CardHeader>
                  <CardContent>
                    <div className="flex gap-4 text-xs text-muted-foreground">
                      <span>{snap.file_count} files</span>
                      <span>{formatBytes(snap.size_bytes)}</span>
                      {snap.link_dest_path && (
                        <span className="truncate max-w-[200px]" title={snap.link_dest_path}>
                          link-dest: {snap.link_dest_path}
                        </span>
                      )}
                    </div>
                  </CardContent>
                </Card>
              ))
            )}
          </div>
        )}
      </ScrollArea>
    </div>
  );
}

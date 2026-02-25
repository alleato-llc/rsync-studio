import { useRef, useState, useEffect } from "react";
import { FolderOpen, Sun, Moon, Monitor } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
import * as api from "@/lib/tauri";
import { themes, type AppearanceMode } from "@/lib/themes";
import { useTheme } from "@/hooks/use-theme";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { Switch } from "@/components/ui/switch";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";

export function SettingsPage() {
  const { theme, setTheme, appearance, setAppearance } = useTheme();
  const [status, setStatus] = useState<{
    type: "success" | "error";
    message: string;
  } | null>(null);
  const [loading, setLoading] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  // Trailing slash state
  const [autoTrailingSlash, setAutoTrailingSlashState] = useState(true);

  // NAS auto-detect state
  const [nasAutoDetect, setNasAutoDetectState] = useState(true);

  // Dry mode state
  const [dryModeItemize, setDryModeItemize] = useState(false);
  const [dryModeChecksum, setDryModeChecksum] = useState(false);
  const [maxItemized, setMaxItemized] = useState(50000);

  // Log directory state
  const [logDir, setLogDir] = useState("");
  const [logDirStatus, setLogDirStatus] = useState<{
    type: "success" | "error";
    message: string;
  } | null>(null);

  // Retention state
  const [maxAgeDays, setMaxAgeDays] = useState(90);
  const [maxPerJob, setMaxPerJob] = useState(15);
  const [invocationCount, setInvocationCount] = useState(0);
  const [retentionStatus, setRetentionStatus] = useState<{
    type: "success" | "error";
    message: string;
  } | null>(null);

  useEffect(() => {
    api.getAutoTrailingSlash().then(setAutoTrailingSlashState).catch(console.error);
    api.getNasAutoDetect().then(setNasAutoDetectState).catch(console.error);
    api
      .getDryModeSettings()
      .then((s) => {
        setDryModeItemize(s.itemize_changes);
        setDryModeChecksum(s.checksum);
      })
      .catch(console.error);
    api.getMaxItemizedChanges().then(setMaxItemized).catch(console.error);
    api.getLogDirectory().then(setLogDir).catch(console.error);
    api
      .getRetentionSettings()
      .then((s) => {
        setMaxAgeDays(s.max_log_age_days);
        setMaxPerJob(s.max_history_per_job);
      })
      .catch(console.error);
    api.countInvocations().then(setInvocationCount).catch(console.error);
  }, []);

  async function handleExport() {
    setLoading(true);
    setStatus(null);
    try {
      const json = await api.exportJobs();
      const blob = new Blob([json], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `rsync-studio-jobs-${new Date().toISOString().slice(0, 10)}.json`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
      setStatus({ type: "success", message: "Jobs exported successfully." });
    } catch (err) {
      setStatus({
        type: "error",
        message: err instanceof Error ? err.message : String(err),
      });
    } finally {
      setLoading(false);
    }
  }

  function handleImportClick() {
    fileInputRef.current?.click();
  }

  async function handleFileSelected(e: React.ChangeEvent<HTMLInputElement>) {
    const file = e.target.files?.[0];
    if (!file) return;

    setLoading(true);
    setStatus(null);
    try {
      const json = await file.text();
      const count = await api.importJobs(json);
      setStatus({
        type: "success",
        message: `Imported ${count} job${count !== 1 ? "s" : ""} successfully.`,
      });
    } catch (err) {
      setStatus({
        type: "error",
        message: err instanceof Error ? err.message : String(err),
      });
    } finally {
      setLoading(false);
      if (fileInputRef.current) {
        fileInputRef.current.value = "";
      }
    }
  }

  async function handleSaveLogDir() {
    setLogDirStatus(null);
    try {
      await api.setLogDirectory(logDir);
      setLogDirStatus({
        type: "success",
        message: "Log directory saved.",
      });
    } catch (err) {
      setLogDirStatus({
        type: "error",
        message: err instanceof Error ? err.message : String(err),
      });
    }
  }

  async function handleSaveRetention() {
    setRetentionStatus(null);
    try {
      await api.setRetentionSettings({
        max_log_age_days: maxAgeDays,
        max_history_per_job: maxPerJob,
      });
      setRetentionStatus({
        type: "success",
        message: "Retention settings saved.",
      });
    } catch (err) {
      setRetentionStatus({
        type: "error",
        message: err instanceof Error ? err.message : String(err),
      });
    }
  }

  return (
    <div className="space-y-4">
      <div>
        <h2 className="text-2xl font-bold">Settings</h2>
        <p className="text-muted-foreground mt-1">
          Configure application preferences and manage job data.
        </p>
      </div>

      {/* Theme */}
      <Card>
        <CardHeader>
          <CardTitle>Theme</CardTitle>
          <CardDescription>
            Choose a color theme for the application.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-4 gap-2">
            {themes.map((t) => (
              <button
                key={t.name}
                onClick={() => setTheme(t.name)}
                className={`flex items-center gap-2 rounded-md border p-2 text-sm transition-colors hover:bg-accent ${
                  theme === t.name
                    ? "border-primary ring-2 ring-primary ring-offset-2 ring-offset-background"
                    : "border-border"
                }`}
              >
                <span
                  className="h-4 w-4 rounded-full shrink-0"
                  style={{ backgroundColor: t.color }}
                />
                {t.label}
              </button>
            ))}
          </div>
          {/* Live preview */}
          <Card className="max-w-sm">
            <CardContent className="p-4 space-y-2">
              <div className="flex items-center gap-2">
                <Button size="sm">Primary</Button>
                <Button size="sm" variant="secondary">
                  Secondary
                </Button>
                <Badge>Badge</Badge>
              </div>
              <Input placeholder="Sample input..." className="max-w-xs" />
            </CardContent>
          </Card>
        </CardContent>
      </Card>

      {/* Appearance */}
      <Card>
        <CardHeader>
          <CardTitle>Appearance</CardTitle>
          <CardDescription>
            Choose between light, dark, or system-matched appearance.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-3 gap-2">
            {([
              { mode: "light" as AppearanceMode, label: "Light", Icon: Sun },
              { mode: "dark" as AppearanceMode, label: "Dark", Icon: Moon },
              { mode: "system" as AppearanceMode, label: "System", Icon: Monitor },
            ]).map(({ mode, label, Icon }) => (
              <button
                key={mode}
                onClick={() => setAppearance(mode)}
                className={`flex items-center justify-center gap-2 rounded-md border p-2 text-sm transition-colors hover:bg-accent ${
                  appearance === mode
                    ? "border-primary ring-2 ring-primary ring-offset-2 ring-offset-background"
                    : "border-border"
                }`}
              >
                <Icon className="h-4 w-4" />
                {label}
              </button>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Trailing Slash */}
      <Card>
        <CardHeader>
          <CardTitle>Trailing Slash</CardTitle>
          <CardDescription>
            Automatically append a trailing slash to source and destination paths
            when building rsync commands. A trailing slash tells rsync to sync
            directory contents rather than copying the directory itself.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex items-center justify-between">
            <Label htmlFor="auto-trailing-slash">Auto-append trailing slash</Label>
            <Switch
              id="auto-trailing-slash"
              checked={autoTrailingSlash}
              onCheckedChange={async (checked) => {
                setAutoTrailingSlashState(checked);
                await api.setAutoTrailingSlash(checked);
              }}
            />
          </div>
        </CardContent>
      </Card>

      {/* NAS Auto-Detection */}
      <Card>
        <CardHeader>
          <CardTitle>NAS / Network Filesystem</CardTitle>
          <CardDescription>
            Automatically detect when a job source or destination is on a
            network filesystem (SMB, NFS, AFP) and suggest compatibility flags
            that prevent unnecessary re-transfers.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex items-center justify-between">
            <div>
              <Label htmlFor="nas-auto-detect">Auto-detect network filesystems</Label>
              <p className="text-xs text-muted-foreground">
                When enabled, the job form will check paths and auto-enable
                --size-only for network mounts.
              </p>
            </div>
            <Switch
              id="nas-auto-detect"
              checked={nasAutoDetect}
              onCheckedChange={async (checked) => {
                setNasAutoDetectState(checked);
                await api.setNasAutoDetect(checked);
              }}
            />
          </div>
        </CardContent>
      </Card>

      {/* Dry Mode */}
      <Card>
        <CardHeader>
          <CardTitle>Dry Mode</CardTitle>
          <CardDescription>
            Configure options applied automatically when running dry-run
            executions via the Flask button.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <Label htmlFor="dry-mode-itemize">Itemize Changes</Label>
              <p className="text-xs text-muted-foreground">
                Add --itemize-changes to show a per-file change summary during
                dry runs.
              </p>
            </div>
            <Switch
              id="dry-mode-itemize"
              checked={dryModeItemize}
              onCheckedChange={async (checked) => {
                setDryModeItemize(checked);
                await api.setDryModeSettings({
                  itemize_changes: checked,
                  checksum: dryModeChecksum,
                });
              }}
            />
          </div>
          <div className="flex items-center justify-between">
            <div>
              <Label htmlFor="dry-mode-checksum">Enable Checksum</Label>
              <p className="text-xs text-muted-foreground">
                Add --checksum to use checksums for more accurate change detection
                during dry runs.
              </p>
            </div>
            <Switch
              id="dry-mode-checksum"
              checked={dryModeChecksum}
              onCheckedChange={async (checked) => {
                setDryModeChecksum(checked);
                await api.setDryModeSettings({
                  itemize_changes: dryModeItemize,
                  checksum: checked,
                });
              }}
            />
          </div>
          <div className="space-y-1">
            <Label htmlFor="max-itemized">Max itemized changes</Label>
            <p className="text-xs text-muted-foreground">
              Maximum number of entries shown in the itemized changes table.
              Reduce if dry runs on very large directories cause slowdowns.
            </p>
            <Input
              id="max-itemized"
              type="number"
              min={100}
              className="max-w-xs"
              value={maxItemized}
              onChange={async (e) => {
                const val = parseInt(e.target.value) || 50000;
                setMaxItemized(val);
                await api.setMaxItemizedChanges(val);
              }}
            />
          </div>
        </CardContent>
      </Card>

      {/* Log Directory */}
      <Card>
        <CardHeader>
          <CardTitle>Log Directory</CardTitle>
          <CardDescription>
            Directory where job execution logs are stored.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="flex gap-2">
            <Input
              className="flex-1"
              value={logDir}
              onChange={(e) => setLogDir(e.target.value)}
              placeholder="/path/to/logs"
            />
            <Button
              variant="outline"
              size="icon"
              title="Browse for folder"
              onClick={async () => {
                const selected = await open({
                  directory: true,
                  multiple: false,
                });
                if (selected) {
                  setLogDir(selected as string);
                }
              }}
            >
              <FolderOpen className="h-4 w-4" />
            </Button>
            <Button onClick={handleSaveLogDir}>Save</Button>
          </div>
          {logDirStatus && (
            <p
              className={`text-sm ${
                logDirStatus.type === "success"
                  ? "text-green-600 dark:text-green-400"
                  : "text-destructive"
              }`}
            >
              {logDirStatus.message}
            </p>
          )}
        </CardContent>
      </Card>

      {/* Retention */}
      <Card>
        <CardHeader>
          <CardTitle>Retention</CardTitle>
          <CardDescription>
            Automatically prune old invocations and log files.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-1">
              <Label className="text-sm">Max log age (days)</Label>
              <Input
                type="number"
                min={1}
                value={maxAgeDays}
                onChange={(e) => setMaxAgeDays(parseInt(e.target.value) || 90)}
              />
            </div>
            <div className="space-y-1">
              <Label className="text-sm">Max runs per job</Label>
              <Input
                type="number"
                min={1}
                value={maxPerJob}
                onChange={(e) => setMaxPerJob(parseInt(e.target.value) || 15)}
              />
            </div>
          </div>
          <p className="text-xs text-muted-foreground">
            Currently saved: {invocationCount} invocation
            {invocationCount !== 1 ? "s" : ""}
          </p>
          <Button onClick={handleSaveRetention}>Save</Button>
          {retentionStatus && (
            <p
              className={`text-sm ${
                retentionStatus.type === "success"
                  ? "text-green-600 dark:text-green-400"
                  : "text-destructive"
              }`}
            >
              {retentionStatus.message}
            </p>
          )}
        </CardContent>
      </Card>

      {/* Export & Import */}
      <Card>
        <CardHeader>
          <CardTitle>Export &amp; Import</CardTitle>
          <CardDescription>
            Export all jobs to a JSON file for backup or transfer. Import jobs
            from a previously exported file.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="flex gap-2">
            <Button onClick={handleExport} disabled={loading}>
              Export All Jobs
            </Button>
            <Button
              variant="outline"
              onClick={handleImportClick}
              disabled={loading}
            >
              Import Jobs
            </Button>
            <input
              ref={fileInputRef}
              type="file"
              accept=".json"
              className="hidden"
              onChange={handleFileSelected}
            />
          </div>
          {status && (
            <p
              className={`text-sm ${
                status.type === "success"
                  ? "text-green-600 dark:text-green-400"
                  : "text-destructive"
              }`}
            >
              {status.message}
            </p>
          )}
        </CardContent>
      </Card>
    </div>
  );
}

import { useRef, useState } from "react";
import * as api from "@/lib/tauri";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";

export function SettingsPage() {
  const [status, setStatus] = useState<{
    type: "success" | "error";
    message: string;
  } | null>(null);
  const [loading, setLoading] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  async function handleExport() {
    setLoading(true);
    setStatus(null);
    try {
      const json = await api.exportJobs();
      const blob = new Blob([json], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `rsync-desktop-jobs-${new Date().toISOString().slice(0, 10)}.json`;
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
      // Reset file input so the same file can be re-imported
      if (fileInputRef.current) {
        fileInputRef.current.value = "";
      }
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
            <Button variant="outline" onClick={handleImportClick} disabled={loading}>
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

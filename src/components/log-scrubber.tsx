import { useState } from "react";
import type { ScrubScanResult, ScrubApplyResult } from "@/types/scrubber";
import * as api from "@/lib/tauri";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog";

type Phase = "input" | "review" | "complete";

export function LogScrubber() {
  const [pattern, setPattern] = useState("");
  const [phase, setPhase] = useState<Phase>("input");
  const [scanResults, setScanResults] = useState<ScrubScanResult[]>([]);
  const [applyResults, setApplyResults] = useState<ScrubApplyResult[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  async function handleScan() {
    if (!pattern.trim()) return;
    setLoading(true);
    setError(null);
    try {
      const results = await api.scrubScanLogs(pattern);
      setScanResults(results);
      if (results.length === 0) {
        setError("No matches found in any log files.");
      } else {
        setPhase("review");
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }

  async function handleApply() {
    setLoading(true);
    setError(null);
    try {
      const filePaths = scanResults.map((r) => r.file_path);
      const results = await api.scrubApplyLogs(pattern, filePaths);
      setApplyResults(results);
      setPhase("complete");
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }

  function handleReset() {
    setPattern("");
    setPhase("input");
    setScanResults([]);
    setApplyResults([]);
    setError(null);
  }

  function basename(filePath: string): string {
    return filePath.split("/").pop() ?? filePath;
  }

  const totalMatches = scanResults.reduce((sum, r) => sum + r.match_count, 0);
  const totalReplacements = applyResults.reduce(
    (sum, r) => sum + r.replacements,
    0
  );

  return (
    <div className="space-y-4">
      <p className="text-muted-foreground">
        Search job execution logs for sensitive text (passwords, keys, etc.) and
        replace all occurrences with asterisks.
      </p>

      {phase === "input" && (
        <div className="space-y-2">
          <Input
            value={pattern}
            onChange={(e) => setPattern(e.target.value)}
            placeholder="Enter text to find and scrub..."
            className="font-mono text-sm"
            onKeyDown={(e) => {
              if (e.key === "Enter") handleScan();
            }}
          />
          <Button
            onClick={handleScan}
            disabled={loading || !pattern.trim()}
          >
            {loading ? "Scanning..." : "Scan Logs"}
          </Button>
        </div>
      )}

      {phase === "review" && (
        <div className="space-y-4">
          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm">Scan Results</CardTitle>
              <CardDescription>
                Found {totalMatches} occurrence{totalMatches !== 1 ? "s" : ""} in{" "}
                {scanResults.length} file{scanResults.length !== 1 ? "s" : ""}
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="rounded-md border">
                <table className="w-full text-sm">
                  <thead>
                    <tr className="border-b bg-muted/50">
                      <th className="px-3 py-2 text-left font-medium">File</th>
                      <th className="px-3 py-2 text-right font-medium">
                        Matches
                      </th>
                    </tr>
                  </thead>
                  <tbody>
                    {scanResults.map((result) => (
                      <tr key={result.file_path} className="border-b last:border-0">
                        <td className="px-3 py-2 font-mono text-xs" title={result.file_path}>
                          {basename(result.file_path)}
                        </td>
                        <td className="px-3 py-2 text-right">
                          {result.match_count}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </CardContent>
          </Card>

          <div className="flex gap-2">
            <AlertDialog>
              <AlertDialogTrigger asChild>
                <Button disabled={loading}>
                  {loading ? "Scrubbing..." : "Scrub All"}
                </Button>
              </AlertDialogTrigger>
              <AlertDialogContent>
                <AlertDialogHeader>
                  <AlertDialogTitle>Confirm Scrub</AlertDialogTitle>
                  <AlertDialogDescription>
                    This will replace {totalMatches} occurrence
                    {totalMatches !== 1 ? "s" : ""} of{" "}
                    <code className="rounded bg-muted px-1 font-mono text-xs">
                      {pattern}
                    </code>{" "}
                    with asterisks in {scanResults.length} log file
                    {scanResults.length !== 1 ? "s" : ""}. This cannot be
                    undone.
                  </AlertDialogDescription>
                </AlertDialogHeader>
                <AlertDialogFooter>
                  <AlertDialogCancel>Cancel</AlertDialogCancel>
                  <AlertDialogAction onClick={handleApply}>
                    Scrub
                  </AlertDialogAction>
                </AlertDialogFooter>
              </AlertDialogContent>
            </AlertDialog>
            <Button variant="outline" onClick={handleReset}>
              Cancel
            </Button>
          </div>
        </div>
      )}

      {phase === "complete" && (
        <div className="space-y-4">
          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm">Scrub Complete</CardTitle>
              <CardDescription>
                Replaced {totalReplacements} occurrence
                {totalReplacements !== 1 ? "s" : ""} across{" "}
                {applyResults.length} file{applyResults.length !== 1 ? "s" : ""}
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="rounded-md border">
                <table className="w-full text-sm">
                  <thead>
                    <tr className="border-b bg-muted/50">
                      <th className="px-3 py-2 text-left font-medium">File</th>
                      <th className="px-3 py-2 text-right font-medium">
                        Replacements
                      </th>
                    </tr>
                  </thead>
                  <tbody>
                    {applyResults.map((result) => (
                      <tr key={result.file_path} className="border-b last:border-0">
                        <td className="px-3 py-2 font-mono text-xs" title={result.file_path}>
                          {basename(result.file_path)}
                        </td>
                        <td className="px-3 py-2 text-right">
                          {result.replacements}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </CardContent>
          </Card>

          <Button onClick={handleReset}>Done</Button>
        </div>
      )}

      {error && (
        <Card className="border-destructive">
          <CardContent className="pt-4">
            <p className="text-sm text-destructive">{error}</p>
          </CardContent>
        </Card>
      )}
    </div>
  );
}

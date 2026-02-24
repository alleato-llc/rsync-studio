import { useState } from "react";
import type { CommandExplanation, ArgCategory } from "@/types/command";
import * as api from "@/lib/tauri";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { Badge } from "@/components/ui/badge";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";

function categoryVariant(
  category: ArgCategory
): "default" | "secondary" | "destructive" | "outline" {
  switch (category) {
    case "Flag":
      return "default";
    case "Pattern":
      return "secondary";
    case "Path":
      return "outline";
    case "Ssh":
      return "default";
    case "Performance":
      return "secondary";
    case "Unknown":
      return "destructive";
  }
}

export function ToolsPage() {
  const [command, setCommand] = useState("");
  const [explanation, setExplanation] = useState<CommandExplanation | null>(
    null
  );
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  async function handleExplain() {
    if (!command.trim()) return;
    setLoading(true);
    setError(null);
    setExplanation(null);
    try {
      const result = await api.explainCommand(command);
      setExplanation(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }

  async function handleImport() {
    if (!command.trim()) return;
    setLoading(true);
    setError(null);
    try {
      const job = await api.parseCommandToJob(command);
      job.name = "Imported Job";
      const created = await api.createJob(job);
      setError(null);
      setExplanation(null);
      setCommand("");
      alert(`Job "${created.name}" created successfully! Go to Jobs to edit it.`);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="space-y-4">
      <div>
        <h2 className="text-2xl font-bold">Command Tools</h2>
        <p className="text-muted-foreground mt-1">
          Paste an rsync command to see what each argument does, or import it as
          a job.
        </p>
      </div>

      <div className="space-y-2">
        <Textarea
          value={command}
          onChange={(e) => setCommand(e.target.value)}
          placeholder="rsync -avz --delete --exclude=*.log /source/ user@host:/backup/"
          className="font-mono text-sm min-h-[80px]"
        />
        <div className="flex gap-2">
          <Button onClick={handleExplain} disabled={loading || !command.trim()}>
            {loading ? "Analyzing..." : "Explain Command"}
          </Button>
          <Button
            variant="outline"
            onClick={handleImport}
            disabled={loading || !command.trim()}
          >
            Import as Job
          </Button>
        </div>
      </div>

      {error && (
        <Card className="border-destructive">
          <CardContent className="pt-4">
            <p className="text-sm text-destructive">{error}</p>
          </CardContent>
        </Card>
      )}

      {explanation && (
        <div className="space-y-4">
          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm">Summary</CardTitle>
            </CardHeader>
            <CardContent>
              <p className="text-sm">{explanation.summary}</p>
            </CardContent>
          </Card>

          <ScrollArea className="h-[calc(100vh-26rem)]">
            <div className="space-y-2 pr-4">
              {explanation.arguments.map((arg, i) => (
                <Card key={i}>
                  <CardHeader className="py-3 pb-1">
                    <div className="flex items-center gap-2">
                      <code className="text-sm font-mono font-medium">
                        {arg.argument}
                      </code>
                      <Badge
                        variant={categoryVariant(arg.category)}
                        className="text-xs"
                      >
                        {arg.category}
                      </Badge>
                    </div>
                  </CardHeader>
                  <CardContent className="pb-3">
                    <CardDescription className="text-sm">
                      {arg.description}
                    </CardDescription>
                  </CardContent>
                </Card>
              ))}
            </div>
          </ScrollArea>
        </div>
      )}
    </div>
  );
}

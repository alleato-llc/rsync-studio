import { useState } from "react";
import type { JobDefinition } from "@/types/job";
import { buildCommandString } from "@/lib/command-preview";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Copy, Check } from "lucide-react";

interface CommandPreviewProps {
  job: JobDefinition;
}

export function CommandPreview({ job }: CommandPreviewProps) {
  const [copied, setCopied] = useState(false);
  const command = buildCommandString(job);

  async function handleCopy() {
    await navigator.clipboard.writeText(command);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  }

  return (
    <Card>
      <CardHeader className="pb-2">
        <div className="flex items-center justify-between">
          <CardTitle className="text-sm">Command Preview</CardTitle>
          <Button variant="ghost" size="icon" className="h-7 w-7" onClick={handleCopy}>
            {copied ? (
              <Check className="h-3.5 w-3.5" />
            ) : (
              <Copy className="h-3.5 w-3.5" />
            )}
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        <pre className="whitespace-pre-wrap break-all rounded-md bg-muted p-3 text-xs font-mono">
          {command}
        </pre>
      </CardContent>
    </Card>
  );
}

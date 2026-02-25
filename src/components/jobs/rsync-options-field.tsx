import type { RsyncOptions } from "@/types/job";
import { Info } from "lucide-react";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Textarea } from "@/components/ui/textarea";
import { PatternListField } from "./pattern-list-field";

interface RsyncOptionsFieldProps {
  value: RsyncOptions;
  onChange: (value: RsyncOptions) => void;
  networkFs?: { location: "source" | "destination"; fsType: string } | null;
}

const BOOLEAN_FLAGS: { key: keyof RsyncOptions; label: string; description: string }[] = [
  { key: "archive", label: "Archive (-a)", description: "Preserve permissions, times, symlinks" },
  { key: "compress", label: "Compress (-z)", description: "Compress data during transfer" },
  { key: "verbose", label: "Verbose (-v)", description: "Show detailed output" },
  { key: "delete", label: "Delete (--delete)", description: "Delete extraneous files from dest" },
  { key: "partial", label: "Partial (--partial)", description: "Keep partially transferred files" },
  { key: "progress", label: "Progress (--progress)", description: "Show transfer progress" },
  { key: "human_readable", label: "Human Readable (-h)", description: "Output numbers in readable format" },
];

export function RsyncOptionsField({ value, onChange, networkFs }: RsyncOptionsFieldProps) {
  function toggleFlag(key: keyof RsyncOptions) {
    onChange({ ...value, [key]: !value[key] });
  }

  return (
    <div className="space-y-6">
      <div className="space-y-4">
        <Label>Flags</Label>
        <div className="grid grid-cols-2 gap-3">
          {BOOLEAN_FLAGS.map(({ key, label, description }) => (
            <div
              key={key}
              className="flex items-center justify-between rounded-md border p-3"
            >
              <div>
                <p className="text-sm font-medium">{label}</p>
                <p className="text-xs text-muted-foreground">{description}</p>
              </div>
              <Switch
                checked={value[key] as boolean}
                onCheckedChange={() => toggleFlag(key)}
              />
            </div>
          ))}
        </div>
      </div>

      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <div>
            <Label>NAS / Network Filesystem</Label>
            <p className="text-xs text-muted-foreground mt-0.5">
              Enable --size-only to compare files by size only, ignoring
              timestamps, permissions, and ownership. Essential for SMB, NFS,
              and other network mounts where these attributes are unreliable.
            </p>
          </div>
          <Switch
            checked={value.size_only}
            onCheckedChange={() => toggleFlag("size_only")}
          />
        </div>

        {networkFs && (
          <div className="flex gap-3 rounded-md border border-blue-200 bg-blue-50 p-3 dark:border-blue-900 dark:bg-blue-950">
            <Info className="mt-0.5 h-4 w-4 shrink-0 text-blue-600 dark:text-blue-400" />
            <div className="text-sm text-blue-800 dark:text-blue-200">
              <p className="font-medium">Network filesystem detected</p>
              <p className="mt-1">
                Your {networkFs.location} is on a <code className="rounded bg-blue-100 px-1 dark:bg-blue-900">{networkFs.fsType}</code> mount.
                These filesystems often report incorrect timestamps and permissions, causing rsync
                to re-transfer unchanged files every run. Size-only mode fixes this by comparing
                files by size alone.
              </p>
            </div>
          </div>
        )}
      </div>

      <PatternListField
        label="Exclude Patterns"
        patterns={value.exclude_patterns}
        onChange={(patterns) => onChange({ ...value, exclude_patterns: patterns })}
        placeholder="e.g. *.tmp"
      />

      <PatternListField
        label="Include Patterns"
        patterns={value.include_patterns}
        onChange={(patterns) => onChange({ ...value, include_patterns: patterns })}
        placeholder="e.g. *.log"
      />

      <div className="space-y-2">
        <Label>Bandwidth Limit (KB/s)</Label>
        <Input
          type="number"
          min={0}
          value={value.bandwidth_limit ?? ""}
          onChange={(e) =>
            onChange({
              ...value,
              bandwidth_limit: e.target.value ? parseInt(e.target.value) : null,
            })
          }
          placeholder="Unlimited"
        />
      </div>

      <div className="space-y-2">
        <Label>Custom Arguments</Label>
        <Textarea
          value={value.custom_args.join("\n")}
          onChange={(e) =>
            onChange({
              ...value,
              custom_args: e.target.value
                ? e.target.value.split("\n").filter((s) => s.trim())
                : [],
            })
          }
          placeholder="One argument per line"
          rows={3}
        />
      </div>
    </div>
  );
}

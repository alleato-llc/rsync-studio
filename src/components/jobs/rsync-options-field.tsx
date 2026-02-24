import type { RsyncOptions } from "@/types/job";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Textarea } from "@/components/ui/textarea";
import { PatternListField } from "./pattern-list-field";

interface RsyncOptionsFieldProps {
  value: RsyncOptions;
  onChange: (value: RsyncOptions) => void;
}

const BOOLEAN_FLAGS: { key: keyof RsyncOptions; label: string; description: string }[] = [
  { key: "archive", label: "Archive (-a)", description: "Preserve permissions, times, symlinks" },
  { key: "compress", label: "Compress (-z)", description: "Compress data during transfer" },
  { key: "verbose", label: "Verbose (-v)", description: "Show detailed output" },
  { key: "delete", label: "Delete (--delete)", description: "Delete extraneous files from dest" },
  { key: "dry_run", label: "Dry Run (--dry-run)", description: "Show what would be transferred" },
  { key: "partial", label: "Partial (--partial)", description: "Keep partially transferred files" },
  { key: "progress", label: "Progress (--progress)", description: "Show transfer progress" },
  { key: "human_readable", label: "Human Readable (-h)", description: "Output numbers in readable format" },
];

export function RsyncOptionsField({ value, onChange }: RsyncOptionsFieldProps) {
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

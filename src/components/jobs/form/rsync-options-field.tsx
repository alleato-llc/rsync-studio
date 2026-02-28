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
  showFileHandling?: boolean;
  showMetadata?: boolean;
  showOutput?: boolean;
}

type OptionGroup = "core_transfer" | "file_handling" | "metadata" | "output";

const BOOLEAN_FLAGS: { group: OptionGroup; key: string; label: string; description: string }[] = [
  { group: "core_transfer", key: "archive", label: "Archive (-a)", description: "Preserve permissions, times, symlinks" },
  { group: "core_transfer", key: "compress", label: "Compress (-z)", description: "Compress data during transfer" },
  { group: "output", key: "verbose", label: "Verbose (-v)", description: "Show detailed output" },
  { group: "file_handling", key: "delete", label: "Delete (--delete)", description: "Delete extraneous files from dest" },
  { group: "core_transfer", key: "partial", label: "Partial (--partial)", description: "Keep partially transferred files" },
  { group: "output", key: "progress", label: "Progress (--progress)", description: "Show transfer progress" },
  { group: "output", key: "human_readable", label: "Human Readable (-h)", description: "Output numbers in readable format" },
];

const FILE_HANDLING_FLAGS: { group: OptionGroup; key: string; label: string; description: string }[] = [
  { group: "file_handling", key: "checksum", label: "Checksum (-c)", description: "Use checksums to detect changes" },
  { group: "file_handling", key: "update", label: "Update (-u)", description: "Skip files newer on destination" },
  { group: "file_handling", key: "whole_file", label: "Whole File (-W)", description: "Disable delta-transfer algorithm" },
  { group: "file_handling", key: "ignore_existing", label: "Ignore Existing", description: "Skip files that already exist on dest" },
  { group: "file_handling", key: "one_file_system", label: "One File System (-x)", description: "Don't cross filesystem boundaries" },
];

const METADATA_FLAGS: { group: OptionGroup; key: string; label: string; description: string }[] = [
  { group: "metadata", key: "hard_links", label: "Hard Links (-H)", description: "Preserve hard links" },
  { group: "metadata", key: "acls", label: "ACLs (-A)", description: "Preserve Access Control Lists" },
  { group: "metadata", key: "xattrs", label: "Extended Attrs (-X)", description: "Preserve extended attributes" },
  { group: "metadata", key: "numeric_ids", label: "Numeric IDs", description: "Transfer numeric user/group IDs" },
];

const OUTPUT_FLAGS: { group: OptionGroup; key: string; label: string; description: string }[] = [
  { group: "output", key: "stats", label: "Stats (--stats)", description: "Print transfer statistics" },
  { group: "output", key: "itemize_changes", label: "Itemize Changes (-i)", description: "Show per-file change summary" },
];

export function RsyncOptionsField({ value, onChange, networkFs, showFileHandling, showMetadata, showOutput }: RsyncOptionsFieldProps) {
  function toggleFlag(group: OptionGroup, key: string) {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const groupObj = value[group] as any;
    onChange({ ...value, [group]: { ...groupObj, [key]: !groupObj[key] } });
  }

  function renderFlagGroup(
    label: string,
    flags: { group: OptionGroup; key: string; label: string; description: string }[],
  ) {
    return (
      <div className="space-y-4">
        <Label>{label}</Label>
        <div className="grid grid-cols-2 gap-3">
          {flags.map(({ group, key, label: flagLabel, description }) => (
            <div
              key={key}
              className="flex items-center justify-between rounded-md border p-3"
            >
              <div>
                <p className="text-sm font-medium">{flagLabel}</p>
                <p className="text-xs text-muted-foreground">{description}</p>
              </div>
              <Switch
                checked={(value[group] as any)[key] as boolean}
                onCheckedChange={() => toggleFlag(group, key)}
              />
            </div>
          ))}
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="space-y-4">
        <Label>Flags</Label>
        <div className="grid grid-cols-2 gap-3">
          {BOOLEAN_FLAGS.map(({ group, key, label, description }) => (
            <div
              key={key}
              className="flex items-center justify-between rounded-md border p-3"
            >
              <div>
                <p className="text-sm font-medium">{label}</p>
                <p className="text-xs text-muted-foreground">{description}</p>
              </div>
              <Switch
                checked={(value[group] as any)[key] as boolean}
                onCheckedChange={() => toggleFlag(group, key)}
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
            checked={value.file_handling.size_only}
            onCheckedChange={() => toggleFlag("file_handling", "size_only")}
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

      {showFileHandling && renderFlagGroup("File Handling", FILE_HANDLING_FLAGS)}
      {showMetadata && renderFlagGroup("Metadata Preservation", METADATA_FLAGS)}
      {showOutput && renderFlagGroup("Output", OUTPUT_FLAGS)}

      <PatternListField
        label="Exclude Patterns"
        patterns={value.advanced.exclude_patterns}
        onChange={(patterns) => onChange({ ...value, advanced: { ...value.advanced, exclude_patterns: patterns } })}
        placeholder="e.g. *.tmp"
      />

      <PatternListField
        label="Include Patterns"
        patterns={value.advanced.include_patterns}
        onChange={(patterns) => onChange({ ...value, advanced: { ...value.advanced, include_patterns: patterns } })}
        placeholder="e.g. *.log"
      />

      <div className="space-y-2">
        <Label>Bandwidth Limit (KB/s)</Label>
        <Input
          type="number"
          min={0}
          value={value.advanced.bandwidth_limit ?? ""}
          onChange={(e) =>
            onChange({
              ...value,
              advanced: {
                ...value.advanced,
                bandwidth_limit: e.target.value ? parseInt(e.target.value) : null,
              },
            })
          }
          placeholder="Unlimited"
        />
      </div>

      <div className="space-y-2">
        <Label>Custom Arguments</Label>
        <Textarea
          value={value.advanced.custom_args.join("\n")}
          onChange={(e) =>
            onChange({
              ...value,
              advanced: {
                ...value.advanced,
                custom_args: e.target.value
                  ? e.target.value.split("\n").filter((s) => s.trim())
                  : [],
              },
            })
          }
          placeholder="One argument per line"
          rows={3}
        />
      </div>
    </div>
  );
}

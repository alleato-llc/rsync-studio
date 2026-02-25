import { useState, useMemo, useRef, useCallback } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import type { ItemizedChange, TransferType, FileType, DifferenceKind } from "@/types/itemize";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { Separator } from "@/components/ui/separator";
import { Copy, Check, Filter, Loader2, X } from "lucide-react";

interface ItemizedChangesTableProps {
  changes: ItemizedChange[];
  isStreaming?: boolean;
  isTruncated?: boolean;
  logFilePath?: string | null;
}

const TRANSFER_OPTIONS: { value: TransferType; label: string }[] = [
  { value: "Sent", label: "Sending" },
  { value: "Received", label: "Receiving" },
  { value: "LocalChange", label: "Local change" },
  { value: "NoUpdate", label: "Up to date" },
  { value: "Message", label: "Deleting" },
];

const FILE_TYPE_OPTIONS: { value: FileType; label: string }[] = [
  { value: "File", label: "file" },
  { value: "Directory", label: "directory" },
  { value: "Symlink", label: "symlink" },
  { value: "Device", label: "device" },
  { value: "Special", label: "special" },
];

const DIFFERENCE_OPTIONS: { value: DifferenceKind; label: string }[] = [
  { value: "Checksum", label: "Checksum" },
  { value: "Size", label: "Size" },
  { value: "Timestamp", label: "Timestamp" },
  { value: "Permissions", label: "Permissions" },
  { value: "Owner", label: "Owner" },
  { value: "Group", label: "Group" },
  { value: "Acl", label: "ACL" },
  { value: "ExtendedAttributes", label: "Xattr" },
  { value: "NewlyCreated", label: "New" },
];

function transferLabel(t: TransferType): string {
  return TRANSFER_OPTIONS.find((o) => o.value === t)?.label ?? t;
}

function fileTypeLabel(t: FileType): string {
  return FILE_TYPE_OPTIONS.find((o) => o.value === t)?.label ?? t;
}

function differenceLabel(d: DifferenceKind): string {
  return DIFFERENCE_OPTIONS.find((o) => o.value === d)?.label ?? d;
}

function toggleSet<T>(set: Set<T>, value: T): Set<T> {
  const next = new Set(set);
  if (next.has(value)) {
    next.delete(value);
  } else {
    next.add(value);
  }
  return next;
}

const GRID_COLUMNS = "100px 80px minmax(120px, 1fr) minmax(200px, 2fr)";

export function ItemizedChangesTable({ changes, isStreaming, isTruncated, logFilePath }: ItemizedChangesTableProps) {
  const [transferFilter, setTransferFilter] = useState<Set<TransferType>>(new Set());
  const [fileTypeFilter, setFileTypeFilter] = useState<Set<FileType>>(new Set());
  const [differenceFilter, setDifferenceFilter] = useState<Set<DifferenceKind>>(new Set());
  const [copied, setCopied] = useState(false);

  const parentRef = useRef<HTMLDivElement>(null);

  const hasFilters = transferFilter.size > 0 || fileTypeFilter.size > 0 || differenceFilter.size > 0;
  const activeFilterCount = transferFilter.size + fileTypeFilter.size + differenceFilter.size;

  const filtered = useMemo(() => {
    if (!hasFilters) return changes;
    return changes.filter((c) => {
      if (transferFilter.size > 0 && !transferFilter.has(c.transfer_type)) return false;
      if (fileTypeFilter.size > 0 && !fileTypeFilter.has(c.file_type)) return false;
      if (differenceFilter.size > 0) {
        const hasDiff = c.differences.some((d) => differenceFilter.has(d));
        if (!hasDiff) return false;
      }
      return true;
    });
  }, [changes, transferFilter, fileTypeFilter, differenceFilter, hasFilters]);

  const virtualizer = useVirtualizer({
    count: filtered.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 32,
    overscan: 20,
  });

  const clearFilters = useCallback(() => {
    setTransferFilter(new Set());
    setFileTypeFilter(new Set());
    setDifferenceFilter(new Set());
  }, []);

  const handleCopyPath = useCallback(async () => {
    if (!logFilePath) return;
    await navigator.clipboard.writeText(logFilePath);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  }, [logFilePath]);

  const countText = hasFilters
    ? `${filtered.length.toLocaleString()} of ${changes.length.toLocaleString()}`
    : changes.length.toLocaleString();

  return (
    <div className="space-y-2">
      {/* Header row */}
      <div className="flex items-center gap-2">
        <h3 className="text-sm font-medium">
          Itemized Changes
          {changes.length > 0 && (
            <span className="text-muted-foreground font-normal"> ({countText})</span>
          )}
        </h3>
        {isStreaming && (
          <Loader2 className="h-3.5 w-3.5 animate-spin text-muted-foreground" />
        )}

        {/* Filter popover */}
        {changes.length > 0 && (
          <Popover>
            <PopoverTrigger asChild>
              <Button variant="ghost" size="icon" className="h-7 w-7 relative">
                <Filter className="h-3.5 w-3.5" />
                {activeFilterCount > 0 && (
                  <span className="absolute -top-0.5 -right-0.5 h-4 w-4 rounded-full bg-primary text-[10px] font-medium text-primary-foreground flex items-center justify-center">
                    {activeFilterCount}
                  </span>
                )}
              </Button>
            </PopoverTrigger>
            <PopoverContent align="start" className="w-80">
              <div className="space-y-3">
                <div className="flex items-center justify-between">
                  <h4 className="text-sm font-medium">Filters</h4>
                  {hasFilters && (
                    <Button variant="ghost" size="sm" className="h-6 text-xs" onClick={clearFilters}>
                      <X className="h-3 w-3 mr-1" />
                      Clear all
                    </Button>
                  )}
                </div>

                <Separator />

                <div className="space-y-1">
                  <span className="text-xs font-medium text-muted-foreground">Transfer</span>
                  <div className="flex flex-wrap gap-1">
                    {TRANSFER_OPTIONS.map((o) => (
                      <Badge
                        key={o.value}
                        variant={transferFilter.has(o.value) ? "default" : "outline"}
                        className="cursor-pointer text-xs"
                        onClick={() => setTransferFilter((s) => toggleSet(s, o.value))}
                      >
                        {o.label}
                      </Badge>
                    ))}
                  </div>
                </div>

                <div className="space-y-1">
                  <span className="text-xs font-medium text-muted-foreground">Type</span>
                  <div className="flex flex-wrap gap-1">
                    {FILE_TYPE_OPTIONS.map((o) => (
                      <Badge
                        key={o.value}
                        variant={fileTypeFilter.has(o.value) ? "default" : "outline"}
                        className="cursor-pointer text-xs"
                        onClick={() => setFileTypeFilter((s) => toggleSet(s, o.value))}
                      >
                        {o.label}
                      </Badge>
                    ))}
                  </div>
                </div>

                <div className="space-y-1">
                  <span className="text-xs font-medium text-muted-foreground">Changes</span>
                  <div className="flex flex-wrap gap-1">
                    {DIFFERENCE_OPTIONS.map((o) => (
                      <Badge
                        key={o.value}
                        variant={differenceFilter.has(o.value) ? "default" : "outline"}
                        className="cursor-pointer text-xs"
                        onClick={() => setDifferenceFilter((s) => toggleSet(s, o.value))}
                      >
                        {o.label}
                      </Badge>
                    ))}
                  </div>
                </div>
              </div>
            </PopoverContent>
          </Popover>
        )}
      </div>

      {/* Truncation warning */}
      {isTruncated && (
        <div className="rounded-md bg-amber-500/10 border border-amber-500/30 p-3 text-sm text-amber-700 dark:text-amber-300 space-y-1">
          <p>
            Table output is limited to the first {changes.length.toLocaleString()} entries. For complete results, review the full log file.
          </p>
          {logFilePath && (
            <div className="flex items-center gap-2">
              <code className="text-xs bg-muted px-1.5 py-0.5 rounded break-all">{logFilePath}</code>
              <Button variant="ghost" size="icon" className="h-6 w-6 shrink-0" onClick={handleCopyPath}>
                {copied ? <Check className="h-3 w-3" /> : <Copy className="h-3 w-3" />}
              </Button>
            </div>
          )}
        </div>
      )}

      {/* Loading state */}
      {changes.length === 0 && isStreaming && (
        <div className="rounded-md border p-6 flex items-center justify-center gap-2 text-sm text-muted-foreground">
          <Loader2 className="h-4 w-4 animate-spin" />
          Waiting for itemized changes...
        </div>
      )}

      {/* Empty filter state */}
      {changes.length > 0 && filtered.length === 0 && (
        <div className="rounded-md border p-6 text-center text-sm text-muted-foreground">
          No changes match the current filters.{" "}
          <button className="underline hover:text-foreground" onClick={clearFilters}>
            Clear filters
          </button>
        </div>
      )}

      {/* Virtualized table */}
      {filtered.length > 0 && (
        <div className="rounded-md border">
          {/* Sticky header */}
          <div
            className="grid text-sm font-medium border-b bg-background px-3 py-2"
            style={{ gridTemplateColumns: GRID_COLUMNS }}
          >
            <div>Transfer</div>
            <div>Type</div>
            <div>Changes</div>
            <div>Path</div>
          </div>

          {/* Scrollable body */}
          <div
            ref={parentRef}
            className="overflow-auto"
            style={{ height: 300 }}
          >
            <div
              style={{
                height: virtualizer.getTotalSize(),
                width: "100%",
                position: "relative",
              }}
            >
              {virtualizer.getVirtualItems().map((virtualRow) => {
                const change = filtered[virtualRow.index];
                return (
                  <div
                    key={virtualRow.index}
                    data-index={virtualRow.index}
                    ref={virtualizer.measureElement}
                    className="grid items-start border-b last:border-b-0 hover:bg-muted/50 px-3 py-1.5"
                    style={{
                      gridTemplateColumns: GRID_COLUMNS,
                      position: "absolute",
                      top: 0,
                      left: 0,
                      width: "100%",
                      transform: `translateY(${virtualRow.start}px)`,
                    }}
                  >
                    <div className="text-xs whitespace-nowrap">
                      {transferLabel(change.transfer_type)}
                    </div>
                    <div className="text-xs text-muted-foreground whitespace-nowrap">
                      {fileTypeLabel(change.file_type)}
                    </div>
                    <div>
                      {change.differences.length > 0 ? (
                        <div className="flex flex-wrap gap-1">
                          {change.differences.map((d) => (
                            <Badge key={d} variant="secondary" className="text-[10px] px-1.5 py-0">
                              {differenceLabel(d)}
                            </Badge>
                          ))}
                        </div>
                      ) : (
                        <span className="text-xs text-muted-foreground">-</span>
                      )}
                    </div>
                    <div className="font-mono text-xs break-all">
                      {change.path}
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

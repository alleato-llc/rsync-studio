import { useRef, useEffect, useCallback } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import type { LogLine } from "@/types/progress";
import { cn } from "@/lib/utils";

const MAX_LOG_LINES = 10_000;

interface VirtualLogViewerProps {
  logs: LogLine[];
  height: number;
  autoScroll: boolean;
  className?: string;
}

export function VirtualLogViewer({
  logs,
  height,
  autoScroll,
  className,
}: VirtualLogViewerProps) {
  const parentRef = useRef<HTMLDivElement>(null);
  const isNearBottomRef = useRef(true);
  const truncated = logs.length >= MAX_LOG_LINES;

  const virtualizer = useVirtualizer({
    count: logs.length + (truncated ? 1 : 0),
    getScrollElement: () => parentRef.current,
    estimateSize: () => 20,
    overscan: 20,
  });

  const handleScroll = useCallback(() => {
    const el = parentRef.current;
    if (!el) return;
    const distanceFromBottom = el.scrollHeight - el.scrollTop - el.clientHeight;
    isNearBottomRef.current = distanceFromBottom < 50;
  }, []);

  useEffect(() => {
    if (autoScroll && isNearBottomRef.current && logs.length > 0) {
      virtualizer.scrollToIndex(logs.length - 1 + (truncated ? 1 : 0), {
        align: "end",
      });
    }
  }, [logs.length, autoScroll, virtualizer, truncated]);

  return (
    <div
      ref={parentRef}
      onScroll={handleScroll}
      className={cn(
        "overflow-auto rounded-md border bg-muted/50 p-4",
        className
      )}
      style={{ height }}
    >
      <div
        style={{
          height: virtualizer.getTotalSize(),
          width: "100%",
          position: "relative",
        }}
      >
        {virtualizer.getVirtualItems().map((virtualRow) => {
          if (truncated && virtualRow.index === 0) {
            return (
              <div
                key="truncated"
                style={{
                  position: "absolute",
                  top: 0,
                  left: 0,
                  width: "100%",
                  height: virtualRow.size,
                  transform: `translateY(${virtualRow.start}px)`,
                }}
                className="text-xs font-mono text-amber-600 dark:text-amber-400"
              >
                --- Output truncated to {MAX_LOG_LINES.toLocaleString()} lines
                ---
              </div>
            );
          }

          const logIndex = truncated
            ? virtualRow.index - 1
            : virtualRow.index;
          const log = logs[logIndex];

          return (
            <div
              key={virtualRow.index}
              data-index={virtualRow.index}
              ref={virtualizer.measureElement}
              style={{
                position: "absolute",
                top: 0,
                left: 0,
                width: "100%",
                transform: `translateY(${virtualRow.start}px)`,
              }}
              className={cn(
                "text-xs font-mono whitespace-pre-wrap",
                log?.is_stderr && "text-destructive"
              )}
            >
              {log?.line}
            </div>
          );
        })}
      </div>
    </div>
  );
}

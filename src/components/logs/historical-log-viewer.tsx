import { useRef, useEffect, useState, useCallback } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import type { LogFileLine } from "@/types/log-file";
import { readLogFileLines } from "@/lib/tauri";
import { cn } from "@/lib/utils";

const CHUNK_SIZE = 500;

interface HistoricalLogViewerProps {
  filePath: string;
  height: number;
  className?: string;
}

export function HistoricalLogViewer({
  filePath,
  height,
  className,
}: HistoricalLogViewerProps) {
  const parentRef = useRef<HTMLDivElement>(null);
  const [totalLines, setTotalLines] = useState(0);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const loadedChunks = useRef<Map<number, LogFileLine[]>>(new Map());
  const pendingChunks = useRef<Set<number>>(new Set());

  const getChunkIndex = (lineIndex: number) =>
    Math.floor(lineIndex / CHUNK_SIZE);

  const getLine = useCallback((index: number): LogFileLine | null => {
    const chunkIdx = getChunkIndex(index);
    const chunk = loadedChunks.current.get(chunkIdx);
    if (!chunk) return null;
    const offsetInChunk = index - chunkIdx * CHUNK_SIZE;
    return chunk[offsetInChunk] ?? null;
  }, []);

  const fetchChunk = useCallback(
    async (chunkIdx: number) => {
      if (
        loadedChunks.current.has(chunkIdx) ||
        pendingChunks.current.has(chunkIdx)
      ) {
        return;
      }
      pendingChunks.current.add(chunkIdx);
      try {
        const offset = chunkIdx * CHUNK_SIZE;
        const result = await readLogFileLines(filePath, offset, CHUNK_SIZE);
        loadedChunks.current.set(chunkIdx, result.lines);
        setTotalLines(result.total_lines);
      } catch {
        // Chunk fetch failed — leave as unloaded
      } finally {
        pendingChunks.current.delete(chunkIdx);
      }
    },
    [filePath]
  );

  // Initial load
  useEffect(() => {
    loadedChunks.current = new Map();
    pendingChunks.current = new Set();
    setLoading(true);
    setError(null);

    readLogFileLines(filePath, 0, CHUNK_SIZE)
      .then((result) => {
        loadedChunks.current.set(0, result.lines);
        setTotalLines(result.total_lines);
        setLoading(false);
      })
      .catch((err) => {
        setError(err instanceof Error ? err.message : String(err));
        setLoading(false);
      });
  }, [filePath]);

  const virtualizer = useVirtualizer({
    count: totalLines,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 20,
    overscan: 20,
    enabled: !loading && !error,
  });

  // Fetch visible chunks on scroll
  const visibleItems = virtualizer.getVirtualItems();
  useEffect(() => {
    if (visibleItems.length === 0) return;
    const firstIdx = visibleItems[0].index;
    const lastIdx = visibleItems[visibleItems.length - 1].index;
    const firstChunk = getChunkIndex(firstIdx);
    const lastChunk = getChunkIndex(lastIdx);

    for (let c = firstChunk; c <= lastChunk; c++) {
      fetchChunk(c);
    }
  }, [visibleItems, fetchChunk]);

  if (loading) {
    return (
      <div
        className={cn(
          "flex items-center justify-center rounded-md border bg-muted/50",
          className
        )}
        style={{ height }}
      >
        <p className="text-sm text-muted-foreground">Loading log file...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div
        className={cn(
          "flex items-center justify-center rounded-md border bg-muted/50",
          className
        )}
        style={{ height }}
      >
        <p className="text-sm text-amber-600 dark:text-amber-400">{error}</p>
      </div>
    );
  }

  if (totalLines === 0) {
    return (
      <div
        className={cn(
          "flex items-center justify-center rounded-md border bg-muted/50",
          className
        )}
        style={{ height }}
      >
        <p className="text-sm text-muted-foreground">Log file is empty.</p>
      </div>
    );
  }

  return (
    <div className="space-y-1">
      <p className="text-xs text-muted-foreground">
        {totalLines.toLocaleString()} lines
      </p>
      <div
        ref={parentRef}
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
            const line = getLine(virtualRow.index);

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
                  line?.is_stderr && "text-destructive",
                  !line && "text-muted-foreground"
                )}
              >
                {line?.text ?? "Loading..."}
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}

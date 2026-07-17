import { useCallback, useEffect, useRef } from "react";
import { Virtuoso, type VirtuosoHandle } from "react-virtuoso";
import type { LogEntry, LogLevel } from "../types";

interface LogListProps {
  entries: LogEntry[];
  onScrollToEndRef?: (fn: () => void) => void;
}

const LEVEL_COLORS: Record<LogLevel, string> = {
  Verbose: "text-gray-400",
  Debug: "text-blue-600",
  Info: "text-green-700",
  Warn: "text-yellow-700",
  Error: "text-red-600",
  Fatal: "text-red-700 font-bold",
};

function formatLine(entry: LogEntry): string {
  return `${entry.timestamp} ${entry.pid} ${entry.tid} ${entry.level[0]} ${entry.tag}: ${entry.message}`;
}

export default function LogList({ entries, onScrollToEndRef }: LogListProps) {
  const virtuosoRef = useRef<VirtuosoHandle>(null);
  const isAtBottom = useRef(true);
  const prevLength = useRef(0);

  // Auto-scroll: when new entries arrive and user is at bottom, scroll down
  useEffect(() => {
    if (isAtBottom.current && entries.length > prevLength.current) {
      virtuosoRef.current?.scrollToIndex(entries.length - 1);
    }
    prevLength.current = entries.length;
  }, [entries.length]);

  // Expose scrollToEnd to parent
  const scrollToEnd = useCallback(() => {
    isAtBottom.current = true;
    virtuosoRef.current?.scrollToIndex(entries.length - 1);
  }, [entries.length]);

  useEffect(() => {
    onScrollToEndRef?.(scrollToEnd);
  }, [scrollToEnd, onScrollToEndRef]);

  const handleScroll = useCallback(
    (atBottom: boolean) => {
      isAtBottom.current = atBottom;
    },
    []
  );

  if (entries.length === 0) {
    return (
      <div className="flex-1 flex items-center justify-center text-gray-400 text-sm select-none">
        No log entries to display
      </div>
    );
  }

  return (
    <div className="flex-1 overflow-hidden">
      <Virtuoso
        ref={virtuosoRef}
        className="h-full"
        totalCount={entries.length}
        atBottomStateChange={handleScroll}
        followOutput={() => false}
        itemContent={(index) => {
          const entry = entries[index];
          const color = LEVEL_COLORS[entry.level] ?? "text-gray-400";
          return (
            <div
              className={`px-3 py-0.5 text-xs font-mono truncate hover:bg-gray-50 ${color}`}
            >
              {formatLine(entry)}
            </div>
          );
        }}
      />
    </div>
  );
}

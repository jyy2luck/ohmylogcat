import { useCallback, useEffect, useMemo, useRef, type ReactNode } from "react";
import { Virtuoso, type VirtuosoHandle } from "react-virtuoso";
import type { LogEntry, LogLevel } from "../types";
import type { FindMatch } from "../hooks/useFindInLog";
import { formatLogLine } from "../utils/logFormat";

interface LogListProps {
  entries: LogEntry[];
  softWrap?: boolean;
  findQuery?: string;
  findMatches?: FindMatch[];
  currentMatchIndex?: number;
  autoScrollToEnd?: boolean;
  onAutoScrollToEndChange?: (enabled: boolean) => void;
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

function renderHighlightedText(
  text: string,
  lineIndex: number,
  matches: FindMatch[],
  currentMatchIndex: number
) {
  const lineMatches = matches
    .map((match, globalIndex) => ({ match, globalIndex }))
    .filter(({ match }) => match.lineIndex === lineIndex);

  if (lineMatches.length === 0) return text;

  const parts: ReactNode[] = [];
  let lastEnd = 0;

  for (const { match, globalIndex } of lineMatches) {
    if (match.start > lastEnd) {
      parts.push(text.slice(lastEnd, match.start));
    }
    const isCurrent = globalIndex === currentMatchIndex;
    parts.push(
      <mark
        key={`${match.start}-${match.end}`}
        className={isCurrent ? "bg-yellow-300" : "bg-yellow-100"}
      >
        {text.slice(match.start, match.end)}
      </mark>
    );
    lastEnd = match.end;
  }

  if (lastEnd < text.length) {
    parts.push(text.slice(lastEnd));
  }

  return parts;
}

export default function LogList({
  entries,
  softWrap = false,
  findQuery = "",
  findMatches = [],
  currentMatchIndex = 0,
  autoScrollToEnd = true,
  onAutoScrollToEndChange,
  onScrollToEndRef,
}: LogListProps) {
  const virtuosoRef = useRef<VirtuosoHandle>(null);

  const findActive = findQuery.trim().length > 0 && findMatches.length > 0;

  useEffect(() => {
    if (findActive) {
      const match = findMatches[currentMatchIndex];
      if (match) {
        virtuosoRef.current?.scrollToIndex({
          index: match.lineIndex,
          align: "center",
        });
      }
      return;
    }

  }, [entries.length, findActive, findMatches, currentMatchIndex, findQuery]);

  const scrollToEnd = useCallback(() => {
    if (entries.length === 0) return;
    virtuosoRef.current?.scrollToIndex({
      index: entries.length - 1,
      align: "end",
    });
  }, [entries.length]);

  useEffect(() => {
    onScrollToEndRef?.(scrollToEnd);
  }, [scrollToEnd, onScrollToEndRef]);

  const handleScroll = useCallback(
    (atBottom: boolean) => {
      if (!atBottom && autoScrollToEnd) {
        onAutoScrollToEndChange?.(false);
      }
    },
    [autoScrollToEnd, onAutoScrollToEndChange]
  );

  const nowrapMinWidth = useMemo(() => {
    if (softWrap || entries.length === 0) return undefined;
    let maxChars = 0;
    for (const entry of entries) {
      maxChars = Math.max(maxChars, formatLogLine(entry).length);
    }
    return maxChars > 0 ? `calc(${maxChars}ch + 1.5rem)` : undefined;
  }, [entries, softWrap]);

  const lineClassName = softWrap
    ? "whitespace-pre-wrap break-all"
    : "whitespace-nowrap";

  if (entries.length === 0) {
    return (
      <div className="flex-1 flex items-center justify-center text-gray-400 text-sm select-none">
        No log entries to display
      </div>
    );
  }

  return (
    <div className="flex-1 min-h-0 overflow-hidden">
      <div
        className={
          softWrap
            ? "h-full"
            : "h-full overflow-x-auto overflow-y-hidden"
        }
      >
        <div
          className="h-full"
          style={nowrapMinWidth ? { minWidth: nowrapMinWidth } : undefined}
        >
          <Virtuoso
            ref={virtuosoRef}
            className="h-full"
            totalCount={entries.length}
            atBottomStateChange={handleScroll}
            followOutput={autoScrollToEnd && !findActive ? "auto" : false}
            increaseViewportBy={softWrap ? 400 : 200}
            itemContent={(index) => {
              const entry = entries[index];
              const color = LEVEL_COLORS[entry.level] ?? "text-gray-400";
              const text = formatLogLine(entry);
              const content =
                findQuery.trim().length > 0
                  ? renderHighlightedText(
                      text,
                      index,
                      findMatches,
                      currentMatchIndex
                    )
                  : text;

              return (
                <div
                  className={`px-3 py-0.5 text-xs font-mono hover:bg-gray-50 ${lineClassName} ${color}`}
                >
                  {content}
                </div>
              );
            }}
          />
        </div>
      </div>
    </div>
  );
}

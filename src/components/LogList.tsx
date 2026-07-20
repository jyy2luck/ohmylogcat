import {
  useCallback,
  useEffect,
  useRef,
  type ReactNode,
} from "react";
import { Virtuoso, type VirtuosoHandle } from "react-virtuoso";
import type { LogEntry, LogLevel } from "../types";
import type { FindMatch } from "../hooks/useFindInLog";
import { formatLogLine } from "../utils/logFormat";

interface LogListProps {
  entries: LogEntry[];
  scrollGeneration?: number;
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
  scrollGeneration = 0,
  softWrap = false,
  findQuery = "",
  findMatches = [],
  currentMatchIndex = 0,
  autoScrollToEnd = true,
  onAutoScrollToEndChange,
  onScrollToEndRef,
}: LogListProps) {
  const virtuosoRef = useRef<VirtuosoHandle>(null);
  const entriesRef = useRef(entries);
  const autoScrollToEndRef = useRef(autoScrollToEnd);
  const findMatchesRef = useRef(findMatches);
  const suppressAutoDisableRef = useRef(false);
  const suppressAutoDisableTimerRef = useRef<number | null>(null);
  const scrollerCleanupRef = useRef<(() => void) | null>(null);
  const lastScrollTopRef = useRef(0);

  entriesRef.current = entries;
  autoScrollToEndRef.current = autoScrollToEnd;
  findMatchesRef.current = findMatches;

  const findActive = findQuery.trim().length > 0 && findMatches.length > 0;

  const beginProgrammaticScroll = useCallback(() => {
    suppressAutoDisableRef.current = true;
    if (suppressAutoDisableTimerRef.current !== null) {
      window.clearTimeout(suppressAutoDisableTimerRef.current);
    }
    suppressAutoDisableTimerRef.current = window.setTimeout(() => {
      suppressAutoDisableRef.current = false;
      suppressAutoDisableTimerRef.current = null;
    }, 300);
  }, []);

  const disableTailFollowing = useCallback(() => {
    onAutoScrollToEndChange?.(false);
  }, [onAutoScrollToEndChange]);

  const handleScrollerRef = useCallback(
    (ref: HTMLElement | null | Window) => {
      scrollerCleanupRef.current?.();
      scrollerCleanupRef.current = null;

      if (!ref || !(ref instanceof HTMLElement)) return;

      lastScrollTopRef.current = ref.scrollTop;

      const onWheel = (event: WheelEvent) => {
        if (suppressAutoDisableRef.current || !autoScrollToEndRef.current) return;
        if (event.deltaY < 0) {
          disableTailFollowing();
        }
      };

      const onScroll = () => {
        const scrollTop = ref.scrollTop;
        if (suppressAutoDisableRef.current) {
          lastScrollTopRef.current = scrollTop;
          return;
        }
        if (
          autoScrollToEndRef.current &&
          scrollTop < lastScrollTopRef.current - 1
        ) {
          disableTailFollowing();
        }
        lastScrollTopRef.current = scrollTop;
      };

      const onKeyDown = (event: KeyboardEvent) => {
        if (suppressAutoDisableRef.current || !autoScrollToEndRef.current) {
          return;
        }
        if (
          event.key === "ArrowUp" ||
          event.key === "PageUp" ||
          event.key === "Home"
        ) {
          disableTailFollowing();
        }
      };

      ref.addEventListener("wheel", onWheel, { passive: true });
      ref.addEventListener("scroll", onScroll, { passive: true });
      ref.addEventListener("keydown", onKeyDown);

      scrollerCleanupRef.current = () => {
        ref.removeEventListener("wheel", onWheel);
        ref.removeEventListener("scroll", onScroll);
        ref.removeEventListener("keydown", onKeyDown);
      };
    },
    [disableTailFollowing]
  );

  const scrollToBottom = useCallback(() => {
    if (entriesRef.current.length === 0) return;
    beginProgrammaticScroll();
    virtuosoRef.current?.scrollToIndex({
      index: "LAST",
      align: "end",
    });
  }, [beginProgrammaticScroll]);

  useEffect(() => {
    return () => {
      scrollerCleanupRef.current?.();
      if (suppressAutoDisableTimerRef.current !== null) {
        window.clearTimeout(suppressAutoDisableTimerRef.current);
      }
    };
  }, []);

  useEffect(() => {
    if (findActive) {
      const match = findMatches[currentMatchIndex];
      if (match) {
        virtuosoRef.current?.scrollToIndex({
          index: match.lineIndex,
          align: "center",
        });
      }
    }
  }, [entries.length, findActive, findMatches, currentMatchIndex, findQuery]);

  useEffect(() => {
    if (!autoScrollToEnd || findActive || entries.length === 0) return;
    scrollToBottom();
  }, [
    scrollGeneration,
    autoScrollToEnd,
    findActive,
    entries.length,
    scrollToBottom,
  ]);

  useEffect(() => {
    onScrollToEndRef?.(scrollToBottom);
  }, [scrollToBottom, onScrollToEndRef]);

  const followOutput = useCallback(() => {
    const findActiveNow =
      findQuery.trim().length > 0 && findMatchesRef.current.length > 0;
    return autoScrollToEndRef.current && !findActiveNow ? "auto" : false;
  }, [findQuery]);

  const itemContent = useCallback(
    (index: number) => {
      const entry = entriesRef.current[index];
      if (!entry) return null;

      const color = LEVEL_COLORS[entry.level] ?? "text-gray-400";
      const text = formatLogLine(entry);
      const query = findQuery.trim();
      const content =
        query.length > 0
          ? renderHighlightedText(
              text,
              index,
              findMatches,
              currentMatchIndex
            )
          : text;

      if (softWrap) {
        return (
          <div
            className={`px-3 py-0.5 text-xs font-mono whitespace-pre-wrap break-all hover:bg-gray-50 ${color}`}
          >
            {content}
          </div>
        );
      }

      return (
        <div className="px-3 py-0.5 text-xs font-mono hover:bg-gray-50">
          <span className={`whitespace-nowrap inline-block ${color}`}>
            {content}
          </span>
        </div>
      );
    },
    [softWrap, findQuery, findMatches, currentMatchIndex]
  );

  if (entries.length === 0) {
    return (
      <div className="flex-1 flex items-center justify-center text-gray-400 text-sm select-none">
        No log entries to display
      </div>
    );
  }

  return (
    <div
      className={`flex-1 min-h-0 ${
        softWrap ? "overflow-hidden" : "overflow-x-auto overflow-y-hidden"
      }`}
    >
      <Virtuoso
        ref={virtuosoRef}
        className="h-full"
        totalCount={entries.length}
        scrollerRef={handleScrollerRef}
        followOutput={followOutput}
        increaseViewportBy={softWrap ? 400 : 200}
        itemContent={itemContent}
      />
    </div>
  );
}

import { useCallback, useEffect, useMemo, useState } from "react";
import type { LogEntry } from "../types";
import { formatLogLine } from "../utils/logFormat";

export interface FindMatch {
  lineIndex: number;
  start: number;
  end: number;
}

function scanMatches(entries: LogEntry[], query: string): FindMatch[] {
  const trimmed = query.trim();
  if (!trimmed) return [];

  const lowerQuery = trimmed.toLowerCase();
  const matches: FindMatch[] = [];

  entries.forEach((entry, lineIndex) => {
    const text = formatLogLine(entry);
    const lowerText = text.toLowerCase();
    let pos = 0;

    while (pos < lowerText.length) {
      const idx = lowerText.indexOf(lowerQuery, pos);
      if (idx === -1) break;
      matches.push({ lineIndex, start: idx, end: idx + trimmed.length });
      pos = idx + lowerQuery.length;
    }
  });

  return matches;
}

export function useFindInLog(entries: LogEntry[]) {
  const [isOpen, setIsOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [currentIndex, setCurrentIndex] = useState(0);

  const matches = useMemo(
    () => scanMatches(entries, query),
    [entries, query]
  );

  useEffect(() => {
    setCurrentIndex((index) => {
      if (matches.length === 0) return 0;
      if (index >= matches.length) return matches.length - 1;
      return index;
    });
  }, [matches]);

  const open = useCallback(() => setIsOpen(true), []);

  const close = useCallback(() => {
    setIsOpen(false);
    setQuery("");
    setCurrentIndex(0);
  }, []);

  const nextMatch = useCallback(() => {
    if (matches.length === 0) return;
    setCurrentIndex((index) => (index + 1) % matches.length);
  }, [matches.length]);

  const prevMatch = useCallback(() => {
    if (matches.length === 0) return;
    setCurrentIndex(
      (index) => (index - 1 + matches.length) % matches.length
    );
  }, [matches.length]);

  return {
    isOpen,
    query,
    setQuery,
    matches,
    currentIndex,
    open,
    close,
    nextMatch,
    prevMatch,
  };
}

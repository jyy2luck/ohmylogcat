import type { LogEntry } from "../types";

export function formatLogLine(entry: LogEntry): string {
  return `${entry.timestamp} ${entry.pid} ${entry.tid} ${entry.level[0]} ${entry.tag}: ${entry.message}`;
}

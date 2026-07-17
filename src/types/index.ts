/// Mirrors the Rust LogEntry struct from parser.
export interface LogEntry {
  timestamp: string;
  pid: number;
  tid: number;
  level: LogLevel;
  tag: string;
  message: string;
}

export type LogLevel = "Verbose" | "Debug" | "Info" | "Warn" | "Error" | "Fatal";

export interface Device {
  serial: string;
  state: string;
}

export interface FilterCriteria {
  tagSubstring?: string;
  messageSubstring?: string;
  minLevel?: LogLevel;
}

export interface BufferStats {
  count: number;
  capacity: number;
  linesPerSec: number;
  memoryEstimateMb: number;
}

export type BufferPreset = "Light" | "Normal" | "Heavy" | "Marathon" | "Custom";

export const BUFFER_PRESETS: Record<BufferPreset, number> = {
  Light: 50_000,
  Normal: 200_000,
  Heavy: 500_000,
  Marathon: 1_000_000,
  Custom: 200_000,
};

export const LOG_LEVEL_ORDER: LogLevel[] = [
  "Verbose",
  "Debug",
  "Info",
  "Warn",
  "Error",
  "Fatal",
];

export interface Settings {
  adbPath?: string;
  bufferCapacity: number;
}

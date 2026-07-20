import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  LogEntry,
  Device,
  BufferStats,
  FilterCriteria,
} from "../types";

/** Max entries kept in frontend memory for Virtuoso rendering. */
const DISPLAY_LIMIT = 10_000;
const STATS_UI_INTERVAL_MS = 200;

function appendDisplayEntries(
  current: LogEntry[],
  incoming: LogEntry[]
): LogEntry[] {
  if (incoming.length === 0) return current;
  const combined = current.length + incoming.length;
  if (combined <= DISPLAY_LIMIT) {
    return current.length === 0 ? incoming : current.concat(incoming);
  }
  if (incoming.length >= DISPLAY_LIMIT) {
    return incoming.slice(-DISPLAY_LIMIT);
  }
  const keepFromCurrent = DISPLAY_LIMIT - incoming.length;
  return current.slice(current.length - keepFromCurrent).concat(incoming);
}

export function useLogcat() {
  const [devices, setDevices] = useState<Device[]>([]);
  const [selectedDevice, setSelectedDevice] = useState("");
  const [entries, setEntries] = useState<LogEntry[]>([]);
  const [scrollGeneration, setScrollGeneration] = useState(0);
  const [isPaused, setIsPaused] = useState(false);
  const [isConnected, setIsConnected] = useState(false);
  const [filter, setFilter] = useState<FilterCriteria>({});
  const [stats, setStats] = useState<BufferStats>({
    count: 0,
    capacity: 200_000,
    linesPerSec: 0,
    memoryEstimateMb: 0,
  });
  const [error, setError] = useState<string | null>(null);

  const entriesRef = useRef<LogEntry[]>([]);
  const pendingBatchRef = useRef<LogEntry[]>([]);
  const flushScheduledRef = useRef(false);
  const latestStatsRef = useRef<BufferStats>(stats);
  const statsTimerRef = useRef<number | null>(null);
  const unlistenRef = useRef<UnlistenFn[]>([]);

  const flushPendingEntries = useCallback(() => {
    flushScheduledRef.current = false;
    if (pendingBatchRef.current.length === 0) return;

    const pending = pendingBatchRef.current;
    pendingBatchRef.current = [];
    const next = appendDisplayEntries(entriesRef.current, pending);
    if (next === entriesRef.current) return;

    entriesRef.current = next;
    setEntries(next);
  }, []);

  const scheduleFlush = useCallback(() => {
    if (flushScheduledRef.current) return;
    flushScheduledRef.current = true;
    requestAnimationFrame(flushPendingEntries);
  }, [flushPendingEntries]);

  const scheduleStatsUpdate = useCallback((next: BufferStats) => {
    latestStatsRef.current = next;
    if (statsTimerRef.current !== null) return;
    statsTimerRef.current = window.setTimeout(() => {
      statsTimerRef.current = null;
      setStats(latestStatsRef.current);
    }, STATS_UI_INTERVAL_MS);
  }, []);

  const bumpScrollGeneration = useCallback(() => {
    setScrollGeneration((value) => value + 1);
  }, []);

  // Refresh device list periodically
  useEffect(() => {
    const fetchDevices = async () => {
      try {
        const result = await invoke<Device[]>("list_devices");
        setDevices(result);
        if (error) setError(null);
      } catch (err) {
        setError(String(err));
      }
    };

    fetchDevices();
    const interval = setInterval(fetchDevices, 5000);
    return () => clearInterval(interval);
  }, []);

  // Listen for Tauri events
  useEffect(() => {
    const setup = async () => {
      const unlistenBatch = await listen<LogEntry[]>("log-batch", (event) => {
        if (event.payload.length === 0) return;
        pendingBatchRef.current.push(...event.payload);
        scheduleFlush();
      });

      const unlistenStats = await listen<BufferStats>(
        "buffer-stats",
        (event) => {
          scheduleStatsUpdate(event.payload);
        }
      );

      const unlistenError = await listen<string>("log-error", (event) => {
        setError(event.payload);
      });

      const unlistenSnapshot = await listen<LogEntry[]>(
        "log-snapshot",
        (event) => {
          pendingBatchRef.current = [];
          flushScheduledRef.current = false;
          entriesRef.current = event.payload.slice(-DISPLAY_LIMIT);
          setEntries(entriesRef.current);
          bumpScrollGeneration();
        }
      );

      const unlistenCleared = await listen("log-cleared", () => {
        pendingBatchRef.current = [];
        flushScheduledRef.current = false;
        entriesRef.current = [];
        setEntries([]);
        bumpScrollGeneration();
      });

      unlistenRef.current = [
        unlistenBatch,
        unlistenStats,
        unlistenError,
        unlistenSnapshot,
        unlistenCleared,
      ];
    };

    setup();

    return () => {
      unlistenRef.current.forEach((u) => u());
      if (statsTimerRef.current !== null) {
        window.clearTimeout(statsTimerRef.current);
      }
    };
  }, [bumpScrollGeneration, scheduleFlush, scheduleStatsUpdate]);

  // Update filter and notify Rust backend
  const updateFilter = useCallback(
    (partial: Partial<FilterCriteria>) => {
      const newFilter = { ...filter, ...partial };
      setFilter(newFilter);
      invoke("set_filter", {
        tag: newFilter.tagSubstring ?? null,
        message: newFilter.messageSubstring ?? null,
        level: newFilter.minLevel ?? null,
      }).catch((err) => setError(String(err)));
    },
    [filter]
  );

  // Select device and start streaming
  const handleDeviceChange = useCallback(
    async (serial: string) => {
      setSelectedDevice(serial);
      if (!serial) {
        setIsConnected(false);
        return;
      }
      try {
        await invoke("stop_stream");
        pendingBatchRef.current = [];
        flushScheduledRef.current = false;
        entriesRef.current = [];
        setEntries([]);
        bumpScrollGeneration();
        setIsPaused(false);
        await invoke("start_stream", { serial });
        setIsConnected(true);
        if (error) setError(null);
      } catch (err) {
        setError(String(err));
        setIsConnected(false);
      }
    },
    [bumpScrollGeneration, error]
  );

  const togglePause = useCallback(async () => {
    try {
      if (isPaused) {
        await invoke("resume_stream");
      } else {
        await invoke("pause_stream");
      }
      setIsPaused((p) => !p);
    } catch (err) {
      setError(String(err));
    }
  }, [isPaused]);

  const clearLogs = useCallback(async () => {
    try {
      await invoke("clear_buffer");
    } catch (err) {
      setError(String(err));
    }
  }, []);

  const scrollToEnd = useCallback(() => {
    scrollToEndRef.current?.();
  }, []);

  const scrollToEndRef = useRef<(() => void) | null>(null);
  const setScrollToEnd = useCallback((fn: () => void) => {
    scrollToEndRef.current = fn;
  }, []);

  return {
    devices,
    selectedDevice,
    entries,
    scrollGeneration,
    isPaused,
    isConnected,
    stats,
    error,
    filter,
    handleDeviceChange,
    togglePause,
    clearLogs,
    scrollToEnd,
    setScrollToEnd,
    updateFilter,
  };
}

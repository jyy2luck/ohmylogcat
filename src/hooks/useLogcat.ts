import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  LogEntry,
  Device,
  BufferStats,
  FilterCriteria,
} from "../types";

export function useLogcat() {
  const [devices, setDevices] = useState<Device[]>([]);
  const [selectedDevice, setSelectedDevice] = useState("");
  const [entries, setEntries] = useState<LogEntry[]>([]);
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
  const unlistenRef = useRef<UnlistenFn[]>([]);

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
        entriesRef.current = [...entriesRef.current, ...event.payload];
        // Keep only last 1000 entries in React state for rendering
        setEntries(entriesRef.current.slice(-10000));
      });

      const unlistenStats = await listen<BufferStats>(
        "buffer-stats",
        (event) => {
          setStats(event.payload);
        }
      );

      const unlistenError = await listen<string>("log-error", (event) => {
        setError(event.payload);
      });

      const unlistenSnapshot = await listen<LogEntry[]>(
        "log-snapshot",
        (event) => {
          entriesRef.current = event.payload;
          setEntries(event.payload.slice(-10000));
        }
      );

      const unlistenCleared = await listen("log-cleared", () => {
        entriesRef.current = [];
        setEntries([]);
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
    };
  }, []);

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
        entriesRef.current = [];
        setEntries([]);
        setIsPaused(false);
        await invoke("start_stream", { serial });
        setIsConnected(true);
        if (error) setError(null);
      } catch (err) {
        setError(String(err));
        setIsConnected(false);
      }
    },
    [error]
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
    // Handled by LogList component
  }, []);

  const scrollToEndRef = useRef<(() => void) | null>(null);
  const setScrollToEnd = useCallback((fn: () => void) => {
    scrollToEndRef.current = fn;
  }, []);

  return {
    devices,
    selectedDevice,
    entries,
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

import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import Toolbar from "./components/Toolbar";
import FilterBar from "./components/FilterBar";
import LogList from "./components/LogList";
import StatusBar from "./components/StatusBar";
import SettingsDialog from "./components/SettingsDialog";
import { useLogcat } from "./hooks/useLogcat";

function App() {
  const {
    devices,
    selectedDevice,
    entries,
    isPaused,
    isConnected,
    stats,
    error,
    handleDeviceChange,
    togglePause,
    clearLogs,
    scrollToEnd,
    updateFilter,
  } = useLogcat();

  const [tagFilter, setTagFilter] = useState("");
  const [messageFilter, setMessageFilter] = useState("");
  const [levelFilter, setLevelFilter] = useState("All");
  const [settingsOpen, setSettingsOpen] = useState(false);

  const handleTagChange = useCallback(
    (val: string) => {
      setTagFilter(val);
      updateFilter({ tagSubstring: val || undefined });
    },
    [updateFilter]
  );

  const handleMessageChange = useCallback(
    (val: string) => {
      setMessageFilter(val);
      updateFilter({ messageSubstring: val || undefined });
    },
    [updateFilter]
  );

  const handleLevelChange = useCallback(
    (val: string) => {
      setLevelFilter(val);
      updateFilter({
        minLevel: val === "All" ? undefined : (val as any),
      });
    },
    [updateFilter]
  );

  const handleExport = useCallback(async () => {
    try {
      await invoke("export_logs", { filteredOnly: true });
    } catch (e) {
      console.error("Export failed:", e);
    }
  }, []);

  return (
    <div className="h-screen w-screen flex flex-col overflow-hidden bg-white text-gray-900">
      {error && (
        <div className="px-3 py-1 text-sm bg-red-100 text-red-700 border-b border-red-200 shrink-0">
          {error}
        </div>
      )}

      <Toolbar
        devices={devices}
        selectedDevice={selectedDevice}
        isPaused={isPaused}
        onDeviceChange={handleDeviceChange}
        onPauseToggle={togglePause}
        onClear={clearLogs}
        onScrollToEnd={scrollToEnd}
        onExport={handleExport}
        onSettings={() => setSettingsOpen(true)}
      />

      <FilterBar
        tagFilter={tagFilter}
        messageFilter={messageFilter}
        levelFilter={levelFilter}
        onTagChange={handleTagChange}
        onMessageChange={handleMessageChange}
        onLevelChange={handleLevelChange}
      />

      <LogList entries={entries} />

      <StatusBar
        connected={isConnected}
        count={stats.count}
        capacity={stats.capacity}
        linesPerSec={stats.linesPerSec}
        memoryEstimateMb={stats.memoryEstimateMb}
      />

      <SettingsDialog
        open={settingsOpen}
        onClose={() => setSettingsOpen(false)}
      />
    </div>
  );
}

export default App;

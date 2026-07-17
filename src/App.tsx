import { useState, useCallback, useRef, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import Toolbar from "./components/Toolbar";
import FilterBar from "./components/FilterBar";
import FindBar from "./components/FindBar";
import LogList from "./components/LogList";
import StatusBar from "./components/StatusBar";
import SettingsDialog from "./components/SettingsDialog";
import { useLogcat } from "./hooks/useLogcat";
import { useSoftWrap } from "./hooks/useSoftWrap";
import { useFindInLog } from "./hooks/useFindInLog";
import { useKeyboardShortcuts } from "./hooks/useKeyboardShortcuts";

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
    setScrollToEnd,
    updateFilter,
  } = useLogcat();

  const [autoScrollToEnd, setAutoScrollToEnd] = useState(true);

  const { softWrap, toggleSoftWrap } = useSoftWrap();
  const find = useFindInLog(entries);
  const findInputRef = useRef<HTMLInputElement>(null);

  const [tagFilter, setTagFilter] = useState("");
  const [messageFilter, setMessageFilter] = useState("");
  const [levelFilter, setLevelFilter] = useState("All");
  const [settingsOpen, setSettingsOpen] = useState(false);

  const openFind = useCallback(() => {
    find.open();
  }, [find]);

  useEffect(() => {
    if (find.isOpen) {
      findInputRef.current?.focus();
      findInputRef.current?.select();
    }
  }, [find.isOpen]);

  useKeyboardShortcuts({
    isFindOpen: find.isOpen,
    onOpenFind: openFind,
    onNextMatch: find.nextMatch,
    onPrevMatch: find.prevMatch,
    onCloseFind: find.close,
  });

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

  const handleScrollToEndToggle = useCallback(() => {
    if (autoScrollToEnd) {
      setAutoScrollToEnd(false);
    } else {
      setAutoScrollToEnd(true);
      scrollToEnd();
    }
  }, [autoScrollToEnd, scrollToEnd]);

  const handleExport = useCallback(async () => {
    try {
      const filePath = await save({
        defaultPath: "logcat.log",
        filters: [{ name: "Log", extensions: ["log", "txt"] }],
      });
      if (!filePath) return;

      await invoke("export_logs", { filePath, filteredOnly: true });
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
        softWrap={softWrap}
        onDeviceChange={handleDeviceChange}
        onPauseToggle={togglePause}
        onClear={clearLogs}
        autoScrollToEnd={autoScrollToEnd}
        onScrollToEndToggle={handleScrollToEndToggle}
        onExport={handleExport}
        onSoftWrapToggle={toggleSoftWrap}
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

      {find.isOpen && (
        <FindBar
          ref={findInputRef}
          query={find.query}
          matchCount={find.matches.length}
          currentIndex={find.currentIndex}
          onQueryChange={find.setQuery}
          onNext={find.nextMatch}
          onPrev={find.prevMatch}
          onClose={find.close}
        />
      )}

      <LogList
        entries={entries}
        softWrap={softWrap}
        findQuery={find.isOpen ? find.query : ""}
        findMatches={find.isOpen ? find.matches : []}
        currentMatchIndex={find.currentIndex}
        autoScrollToEnd={autoScrollToEnd}
        onAutoScrollToEndChange={setAutoScrollToEnd}
        onScrollToEndRef={setScrollToEnd}
      />

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

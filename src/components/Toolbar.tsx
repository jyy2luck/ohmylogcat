import type { Device } from "../types";

interface ToolbarProps {
  devices: Device[];
  selectedDevice: string;
  isPaused: boolean;
  onDeviceChange: (serial: string) => void;
  onPauseToggle: () => void;
  onClear: () => void;
  onScrollToEnd: () => void;
  onExport: () => void;
  onSettings: () => void;
}

export default function Toolbar({
  devices,
  selectedDevice,
  isPaused,
  onDeviceChange,
  onPauseToggle,
  onClear,
  onScrollToEnd,
  onExport,
  onSettings,
}: ToolbarProps) {
  return (
    <header className="flex items-center gap-2 px-3 py-1.5 border-b border-gray-200 bg-gray-50 shrink-0">
      <select
        className="text-sm border border-gray-300 rounded px-2 py-1 min-w-[180px] bg-white"
        value={selectedDevice}
        onChange={(e) => onDeviceChange(e.target.value)}
      >
        <option value="">— No device —</option>
        {devices.map((d) => (
          <option key={d.serial} value={d.serial}>
            {d.serial} ({d.state})
          </option>
        ))}
      </select>

      <span className="text-gray-300 select-none">|</span>

      <button
        className="px-3 py-1 text-sm border border-gray-300 rounded hover:bg-gray-100 cursor-pointer"
        onClick={onPauseToggle}
      >
        {isPaused ? "Resume" : "Pause"}
      </button>

      <button
        className="px-3 py-1 text-sm border border-gray-300 rounded hover:bg-gray-100 cursor-pointer"
        onClick={onClear}
      >
        Clear
      </button>

      <button
        className="px-3 py-1 text-sm border border-gray-300 rounded hover:bg-gray-100 cursor-pointer"
        onClick={onScrollToEnd}
      >
        Scroll to End
      </button>

      <button
        className="px-3 py-1 text-sm border border-gray-300 rounded hover:bg-gray-100 cursor-pointer"
        onClick={onExport}
      >
        Export
      </button>

      <div className="flex-1" />

      <button
        className="px-2 py-1 text-sm border border-gray-300 rounded hover:bg-gray-100 cursor-pointer"
        onClick={onSettings}
        title="Settings"
      >
        ⚙
      </button>
    </header>
  );
}

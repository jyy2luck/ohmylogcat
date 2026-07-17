import type { Device } from "../types";

interface ToolbarProps {
  devices: Device[];
  selectedDevice: string;
  isPaused: boolean;
  softWrap: boolean;
  autoScrollToEnd: boolean;
  onDeviceChange: (serial: string) => void;
  onPauseToggle: () => void;
  onClear: () => void;
  onScrollToEndToggle: () => void;
  onExport: () => void;
  onSoftWrapToggle: () => void;
  onSettings: () => void;
}

export default function Toolbar({
  devices,
  selectedDevice,
  isPaused,
  softWrap,
  autoScrollToEnd,
  onDeviceChange,
  onPauseToggle,
  onClear,
  onScrollToEndToggle,
  onExport,
  onSoftWrapToggle,
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
        className={`px-3 py-1 text-sm border rounded cursor-pointer ${
          autoScrollToEnd
            ? "border-blue-400 bg-blue-50 hover:bg-blue-100"
            : "border-gray-300 hover:bg-gray-100"
        }`}
        onClick={onScrollToEndToggle}
        title={
          autoScrollToEnd
            ? "Auto-scroll to newest logs (on)"
            : "Auto-scroll to newest logs (off)"
        }
      >
        Scroll to End
      </button>

      <button
        className="px-3 py-1 text-sm border border-gray-300 rounded hover:bg-gray-100 cursor-pointer"
        onClick={onExport}
      >
        Export
      </button>

      <span className="text-gray-300 select-none">|</span>

      <button
        className={`px-3 py-1 text-sm border rounded cursor-pointer ${
          softWrap
            ? "border-blue-400 bg-blue-50 hover:bg-blue-100"
            : "border-gray-300 hover:bg-gray-100"
        }`}
        onClick={onSoftWrapToggle}
        title="Use Soft Wraps"
      >
        Soft-Wrap
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

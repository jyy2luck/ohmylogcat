interface StatusBarProps {
  connected: boolean;
  count: number;
  capacity: number;
  linesPerSec: number;
  memoryEstimateMb: number;
}

export default function StatusBar({
  connected,
  count,
  capacity,
  linesPerSec,
  memoryEstimateMb,
}: StatusBarProps) {
  return (
    <footer className="flex items-center gap-4 px-3 py-1 border-t border-gray-200 bg-gray-50 text-xs text-gray-500 shrink-0 font-mono">
      <span className="flex items-center gap-1">
        <span
          className={`w-2 h-2 rounded-full inline-block ${
            connected ? "bg-green-500" : "bg-gray-300"
          }`}
        />
        {connected ? "Connected" : "Disconnected"}
      </span>
      <span>
        {count.toLocaleString()} / {capacity.toLocaleString()}
      </span>
      <span>{linesPerSec.toFixed(0)} lines/s</span>
      <span>~{memoryEstimateMb.toFixed(0)} MB</span>
    </footer>
  );
}

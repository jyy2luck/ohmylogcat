interface FilterBarProps {
  tagFilter: string;
  messageFilter: string;
  levelFilter: string;
  onTagChange: (val: string) => void;
  onMessageChange: (val: string) => void;
  onLevelChange: (val: string) => void;
}

export default function FilterBar({
  tagFilter,
  messageFilter,
  levelFilter,
  onTagChange,
  onMessageChange,
  onLevelChange,
}: FilterBarProps) {
  return (
    <div className="flex items-center gap-2 px-3 py-1.5 border-b border-gray-200 bg-gray-50 shrink-0">
      <input
        className="flex-1 text-sm border border-gray-300 rounded px-2 py-1 bg-white placeholder:text-gray-400"
        placeholder="Tag filter..."
        value={tagFilter}
        onChange={(e) => onTagChange(e.target.value)}
      />
      <input
        className="flex-1 text-sm border border-gray-300 rounded px-2 py-1 bg-white placeholder:text-gray-400"
        placeholder="Message filter..."
        value={messageFilter}
        onChange={(e) => onMessageChange(e.target.value)}
      />
      <select
        className="text-sm border border-gray-300 rounded px-2 py-1 bg-white"
        value={levelFilter}
        onChange={(e) => onLevelChange(e.target.value)}
      >
        <option value="All">All</option>
        <option value="Verbose">Verbose</option>
        <option value="Debug">Debug</option>
        <option value="Info">Info</option>
        <option value="Warn">Warn</option>
        <option value="Error">Error</option>
      </select>
    </div>
  );
}

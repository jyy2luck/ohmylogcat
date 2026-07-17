import { forwardRef } from "react";

interface FindBarProps {
  query: string;
  matchCount: number;
  currentIndex: number;
  onQueryChange: (query: string) => void;
  onNext: () => void;
  onPrev: () => void;
  onClose: () => void;
}

const FindBar = forwardRef<HTMLInputElement, FindBarProps>(function FindBar(
  { query, matchCount, currentIndex, onQueryChange, onNext, onPrev, onClose },
  ref
) {
  const counterText =
    matchCount === 0
      ? query.trim()
        ? "0/0"
        : ""
      : `${currentIndex + 1}/${matchCount}`;

  return (
    <div className="flex items-center gap-2 px-3 py-1.5 border-b border-gray-200 bg-gray-50 shrink-0">
      <input
        ref={ref}
        className="flex-1 text-sm border border-gray-300 rounded px-2 py-1 bg-white placeholder:text-gray-400"
        placeholder="Find in log..."
        value={query}
        onChange={(e) => onQueryChange(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Enter") {
            e.preventDefault();
            if (e.shiftKey) {
              onPrev();
            } else {
              onNext();
            }
          } else if (e.key === "Escape") {
            e.preventDefault();
            onClose();
          }
        }}
      />

      <span className="text-xs text-gray-500 tabular-nums min-w-[3rem] text-center">
        {counterText}
      </span>

      <button
        className="px-2 py-1 text-sm border border-gray-300 rounded hover:bg-gray-100 cursor-pointer"
        onClick={onPrev}
        title="Previous match (Shift+Enter)"
        disabled={matchCount === 0}
      >
        ▲
      </button>

      <button
        className="px-2 py-1 text-sm border border-gray-300 rounded hover:bg-gray-100 cursor-pointer"
        onClick={onNext}
        title="Next match (Enter)"
        disabled={matchCount === 0}
      >
        ▼
      </button>

      <button
        className="px-2 py-1 text-sm border border-gray-300 rounded hover:bg-gray-100 cursor-pointer"
        onClick={onClose}
        title="Close (Esc)"
      >
        ✕
      </button>
    </div>
  );
});

export default FindBar;

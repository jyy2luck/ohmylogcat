import { useEffect } from "react";

interface UseKeyboardShortcutsOptions {
  isFindOpen: boolean;
  onOpenFind: () => void;
  onNextMatch: () => void;
  onPrevMatch: () => void;
  onCloseFind: () => void;
}

export function useKeyboardShortcuts({
  isFindOpen,
  onOpenFind,
  onNextMatch,
  onPrevMatch,
  onCloseFind,
}: UseKeyboardShortcutsOptions) {
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      const isMod = event.metaKey || event.ctrlKey;

      if (isMod && event.key.toLowerCase() === "f") {
        event.preventDefault();
        onOpenFind();
        return;
      }

      if (!isFindOpen) return;

      if (event.key === "Escape") {
        event.preventDefault();
        onCloseFind();
        return;
      }

      if (event.key === "Enter") {
        event.preventDefault();
        if (event.shiftKey) {
          onPrevMatch();
        } else {
          onNextMatch();
        }
      }
    };

    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [isFindOpen, onOpenFind, onNextMatch, onPrevMatch, onCloseFind]);
}

import { useCallback, useState } from "react";

const STORAGE_KEY = "ohmylogcat.scrollToEnd";

function readStored(): boolean {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored === null) return true;
    return stored === "true";
  } catch {
    return true;
  }
}

export function useScrollToEnd() {
  const [scrollToEndEnabled, setScrollToEndEnabledState] = useState(readStored);

  const setScrollToEndEnabled = useCallback((enabled: boolean) => {
    setScrollToEndEnabledState(enabled);
    try {
      localStorage.setItem(STORAGE_KEY, String(enabled));
    } catch {
      // ignore storage errors
    }
  }, []);

  return { scrollToEndEnabled, setScrollToEndEnabled };
}

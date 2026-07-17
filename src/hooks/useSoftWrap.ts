import { useCallback, useState } from "react";

const STORAGE_KEY = "ohmylogcat.softWrap";

function readStored(): boolean {
  try {
    return localStorage.getItem(STORAGE_KEY) === "true";
  } catch {
    return false;
  }
}

export function useSoftWrap() {
  const [softWrap, setSoftWrap] = useState(readStored);

  const toggleSoftWrap = useCallback(() => {
    setSoftWrap((prev) => {
      const next = !prev;
      try {
        localStorage.setItem(STORAGE_KEY, String(next));
      } catch {
        // ignore storage errors
      }
      return next;
    });
  }, []);

  return { softWrap, toggleSoftWrap };
}

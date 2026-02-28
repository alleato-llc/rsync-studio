import { useState, useEffect } from "react";
import * as api from "@/lib/tauri";

export function useShowFileHandlingOptions() {
  const [enabled, setEnabled] = useState(false);
  useEffect(() => {
    api.getShowFileHandlingOptions().then(setEnabled).catch(console.error);
  }, []);
  return enabled;
}

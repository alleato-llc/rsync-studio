import { useState, useEffect } from "react";
import * as api from "@/lib/tauri";

export function useShowOutputOptions() {
  const [enabled, setEnabled] = useState(false);
  useEffect(() => {
    api.getShowOutputOptions().then(setEnabled).catch(console.error);
  }, []);
  return enabled;
}

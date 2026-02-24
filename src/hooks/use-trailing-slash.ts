import { useState, useEffect } from "react";
import * as api from "@/lib/tauri";

export function useTrailingSlash() {
  const [enabled, setEnabled] = useState(true);
  useEffect(() => {
    api.getAutoTrailingSlash().then(setEnabled).catch(console.error);
  }, []);
  return enabled;
}

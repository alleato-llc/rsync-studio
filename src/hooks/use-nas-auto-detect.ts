import { useState, useEffect } from "react";
import * as api from "@/lib/tauri";

export function useNasAutoDetect() {
  const [enabled, setEnabled] = useState(true);
  useEffect(() => {
    api.getNasAutoDetect().then(setEnabled).catch(console.error);
  }, []);
  return enabled;
}

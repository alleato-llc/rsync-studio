import { useState, useEffect } from "react";
import * as api from "@/lib/tauri";

export function useShowMetadataOptions() {
  const [enabled, setEnabled] = useState(false);
  useEffect(() => {
    api.getShowMetadataOptions().then(setEnabled).catch(console.error);
  }, []);
  return enabled;
}

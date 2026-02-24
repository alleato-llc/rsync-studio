import { useState, useEffect, useCallback } from "react";
import { DEFAULT_THEME, DEFAULT_APPEARANCE, type AppearanceMode } from "@/lib/themes";

const THEME_KEY = "rsync-studio-theme";
const APPEARANCE_KEY = "rsync-studio-appearance";

function getSystemDark(): boolean {
  return window.matchMedia("(prefers-color-scheme: dark)").matches;
}

function applyDarkClass(mode: AppearanceMode) {
  const isDark = mode === "dark" || (mode === "system" && getSystemDark());
  document.documentElement.classList.toggle("dark", isDark);
}

export function useTheme() {
  const [theme, setThemeState] = useState(() => {
    return localStorage.getItem(THEME_KEY) || DEFAULT_THEME;
  });

  const [appearance, setAppearanceState] = useState<AppearanceMode>(() => {
    return (localStorage.getItem(APPEARANCE_KEY) as AppearanceMode) || DEFAULT_APPEARANCE;
  });

  const setAppearance = useCallback((mode: AppearanceMode) => {
    setAppearanceState(mode);
    localStorage.setItem(APPEARANCE_KEY, mode);
    applyDarkClass(mode);
  }, []);

  // Apply color theme
  useEffect(() => {
    document.documentElement.setAttribute("data-theme", theme);
    localStorage.setItem(THEME_KEY, theme);
  }, [theme]);

  // Apply appearance mode
  useEffect(() => {
    applyDarkClass(appearance);
  }, [appearance]);

  // Listen for system preference changes when in "system" mode
  useEffect(() => {
    if (appearance !== "system") return;

    const mql = window.matchMedia("(prefers-color-scheme: dark)");
    const handler = () => applyDarkClass("system");
    mql.addEventListener("change", handler);
    return () => mql.removeEventListener("change", handler);
  }, [appearance]);

  // Apply on mount
  useEffect(() => {
    const savedTheme = localStorage.getItem(THEME_KEY) || DEFAULT_THEME;
    const savedAppearance = (localStorage.getItem(APPEARANCE_KEY) as AppearanceMode) || DEFAULT_APPEARANCE;
    document.documentElement.setAttribute("data-theme", savedTheme);
    applyDarkClass(savedAppearance);
  }, []);

  return {
    theme,
    setTheme: setThemeState,
    appearance,
    setAppearance,
  };
}

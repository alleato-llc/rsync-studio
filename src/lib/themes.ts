export interface Theme {
  name: string;
  label: string;
  color: string; // CSS color for the preview swatch
}

export const themes: Theme[] = [
  { name: "default", label: "Gray", color: "hsl(0 0% 45%)" },
  { name: "blue", label: "Blue", color: "hsl(221 83% 53%)" },
  { name: "green", label: "Green", color: "hsl(142 71% 45%)" },
  { name: "orange", label: "Orange", color: "hsl(25 95% 53%)" },
  { name: "red", label: "Red", color: "hsl(0 84% 60%)" },
  { name: "rose", label: "Rose", color: "hsl(347 77% 50%)" },
  { name: "violet", label: "Violet", color: "hsl(263 70% 50%)" },
  { name: "yellow", label: "Yellow", color: "hsl(48 96% 53%)" },
];

export const DEFAULT_THEME = "violet";

export type AppearanceMode = "light" | "dark" | "system";
export const DEFAULT_APPEARANCE: AppearanceMode = "system";

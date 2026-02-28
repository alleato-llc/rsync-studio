export interface ArgumentExplanation {
  argument: string;
  description: string;
  category: ArgCategory;
}

export type ArgCategory =
  | "Flag"
  | "Pattern"
  | "Path"
  | "Ssh"
  | "Performance"
  | "FileHandling"
  | "Metadata"
  | "Output"
  | "Deletion"
  | "Unknown";

export interface CommandExplanation {
  arguments: ArgumentExplanation[];
  summary: string;
}

export type TransferType =
  | "Sent"
  | "Received"
  | "LocalChange"
  | "NoUpdate"
  | "Message";

export type FileType =
  | "File"
  | "Directory"
  | "Symlink"
  | "Device"
  | "Special";

export type DifferenceKind =
  | "Checksum"
  | "Size"
  | "Timestamp"
  | "Permissions"
  | "Owner"
  | "Group"
  | "Acl"
  | "ExtendedAttributes"
  | "NewlyCreated";

export interface ItemizedChange {
  transfer_type: TransferType;
  file_type: FileType;
  differences: DifferenceKind[];
  path: string;
}

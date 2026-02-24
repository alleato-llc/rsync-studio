import { FolderOpen } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
import type { StorageLocation } from "@/types/job";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

interface StorageLocationFieldProps {
  label: string;
  value: StorageLocation;
  onChange: (value: StorageLocation) => void;
  autoTrailingSlash?: boolean;
}

export function StorageLocationField({
  label,
  value,
  onChange,
  autoTrailingSlash,
}: StorageLocationFieldProps) {
  const showSlashIndicator =
    autoTrailingSlash && value.path && !value.path.endsWith("/");
  function handleTypeChange(type: string) {
    switch (type) {
      case "Local":
        onChange({ type: "Local", path: "" });
        break;
      case "RemoteSsh":
        onChange({
          type: "RemoteSsh",
          user: "",
          host: "",
          port: 22,
          path: "",
          identity_file: null,
        });
        break;
      case "RemoteRsync":
        onChange({
          type: "RemoteRsync",
          host: "",
          module: "",
          path: "",
        });
        break;
    }
  }

  return (
    <div className="space-y-3">
      <Label>{label}</Label>
      <Select value={value.type} onValueChange={handleTypeChange}>
        <SelectTrigger>
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="Local">Local</SelectItem>
          <SelectItem value="RemoteSsh">Remote (SSH)</SelectItem>
          <SelectItem value="RemoteRsync">Remote (rsync daemon)</SelectItem>
        </SelectContent>
      </Select>

      {value.type === "Local" && (
        <div className="space-y-1.5">
          <div className="flex gap-2">
            <div className="relative flex-1">
              <Input
                className={showSlashIndicator ? "pr-6" : ""}
                value={value.path}
                onChange={(e) => onChange({ ...value, path: e.target.value })}
                placeholder="/path/to/directory"
              />
              {showSlashIndicator && (
                <span className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground pointer-events-none text-sm">
                  /
                </span>
              )}
            </div>
            <Button
              type="button"
              variant="outline"
              size="icon"
              title="Browse for folder"
              onClick={async () => {
                const selected = await open({ directory: true, multiple: false });
                if (selected) {
                  onChange({ ...value, path: selected as string });
                }
              }}
            >
              <FolderOpen className="h-4 w-4" />
            </Button>
          </div>
          {autoTrailingSlash !== undefined && (
            <p className="text-xs text-muted-foreground">
              {autoTrailingSlash
                ? "A trailing slash will be auto-appended — syncs directory contents."
                : "Without trailing slash, rsync copies the directory itself into the destination."}
            </p>
          )}
        </div>
      )}

      {value.type === "RemoteSsh" && (
        <div className="space-y-1.5">
          <div className="grid grid-cols-2 gap-2">
            <div className="space-y-1">
              <Label className="text-xs text-muted-foreground">User</Label>
              <Input
                value={value.user}
                onChange={(e) => onChange({ ...value, user: e.target.value })}
                placeholder="user"
              />
            </div>
            <div className="space-y-1">
              <Label className="text-xs text-muted-foreground">Host</Label>
              <Input
                value={value.host}
                onChange={(e) => onChange({ ...value, host: e.target.value })}
                placeholder="example.com"
              />
            </div>
            <div className="space-y-1">
              <Label className="text-xs text-muted-foreground">Port</Label>
              <Input
                type="number"
                value={value.port}
                onChange={(e) =>
                  onChange({ ...value, port: parseInt(e.target.value) || 22 })
                }
              />
            </div>
            <div className="space-y-1">
              <Label className="text-xs text-muted-foreground">Path</Label>
              <div className="relative">
                <Input
                  className={showSlashIndicator ? "pr-6" : ""}
                  value={value.path}
                  onChange={(e) => onChange({ ...value, path: e.target.value })}
                  placeholder="/remote/path"
                />
                {showSlashIndicator && (
                  <span className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground pointer-events-none text-sm">
                    /
                  </span>
                )}
              </div>
            </div>
          </div>
          {autoTrailingSlash !== undefined && (
            <p className="text-xs text-muted-foreground">
              {autoTrailingSlash
                ? "A trailing slash will be auto-appended — syncs directory contents."
                : "Without trailing slash, rsync copies the directory itself into the destination."}
            </p>
          )}
        </div>
      )}

      {value.type === "RemoteRsync" && (
        <div className="space-y-1.5">
          <div className="grid grid-cols-2 gap-2">
            <div className="space-y-1">
              <Label className="text-xs text-muted-foreground">Host</Label>
              <Input
                value={value.host}
                onChange={(e) => onChange({ ...value, host: e.target.value })}
                placeholder="example.com"
              />
            </div>
            <div className="space-y-1">
              <Label className="text-xs text-muted-foreground">Module</Label>
              <Input
                value={value.module}
                onChange={(e) => onChange({ ...value, module: e.target.value })}
                placeholder="backup"
              />
            </div>
            <div className="col-span-2 space-y-1">
              <Label className="text-xs text-muted-foreground">Path</Label>
              <div className="relative">
                <Input
                  className={showSlashIndicator ? "pr-6" : ""}
                  value={value.path}
                  onChange={(e) => onChange({ ...value, path: e.target.value })}
                  placeholder="/path"
                />
                {showSlashIndicator && (
                  <span className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground pointer-events-none text-sm">
                    /
                  </span>
                )}
              </div>
            </div>
          </div>
          {autoTrailingSlash !== undefined && (
            <p className="text-xs text-muted-foreground">
              {autoTrailingSlash
                ? "A trailing slash will be auto-appended — syncs directory contents."
                : "Without trailing slash, rsync copies the directory itself into the destination."}
            </p>
          )}
        </div>
      )}
    </div>
  );
}

import type { SshConfig } from "@/types/job";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";

interface SshConfigFieldProps {
  value: SshConfig;
  onChange: (value: SshConfig) => void;
}

export function SshConfigField({ value, onChange }: SshConfigFieldProps) {
  return (
    <div className="space-y-4">
      <Label className="text-base font-medium">SSH Configuration</Label>

      <div className="grid grid-cols-2 gap-3">
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
          <Label className="text-xs text-muted-foreground">Identity File</Label>
          <Input
            value={value.identity_file ?? ""}
            onChange={(e) =>
              onChange({
                ...value,
                identity_file: e.target.value || null,
              })
            }
            placeholder="~/.ssh/id_rsa"
          />
        </div>
      </div>

      <div className="flex items-center justify-between rounded-md border p-3">
        <div>
          <p className="text-sm font-medium">Strict Host Key Checking</p>
          <p className="text-xs text-muted-foreground">
            Verify remote host key on connection
          </p>
        </div>
        <Switch
          checked={value.strict_host_key_checking}
          onCheckedChange={(checked) =>
            onChange({ ...value, strict_host_key_checking: checked })
          }
        />
      </div>

      <div className="space-y-1">
        <Label className="text-xs text-muted-foreground">Custom SSH Command</Label>
        <Input
          value={value.custom_ssh_command ?? ""}
          onChange={(e) =>
            onChange({
              ...value,
              custom_ssh_command: e.target.value || null,
            })
          }
          placeholder="e.g. ssh -o ConnectTimeout=10"
        />
      </div>
    </div>
  );
}

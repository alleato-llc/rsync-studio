import type {
  JobDefinition,
  StorageLocation,
  RsyncOptions,
  SshConfig,
} from "@/types/job";

function storageLocationToRsyncPath(loc: StorageLocation): string {
  switch (loc.type) {
    case "Local":
      return loc.path;
    case "RemoteSsh":
      return `${loc.user}@${loc.host}:${loc.path}`;
    case "RemoteRsync":
      return `rsync://${loc.host}/${loc.module}/${loc.path}`;
  }
}

function buildSshArgs(ssh: SshConfig): string[] {
  if (ssh.custom_ssh_command) {
    return [`-e ${ssh.custom_ssh_command}`];
  }

  const sshParts = ["ssh"];

  if (ssh.port !== 22) {
    sshParts.push(`-p ${ssh.port}`);
  }

  if (ssh.identity_file) {
    sshParts.push(`-i ${ssh.identity_file}`);
  }

  if (!ssh.strict_host_key_checking) {
    sshParts.push("-o StrictHostKeyChecking=no");
  }

  if (sshParts.length > 1) {
    return ["-e", sshParts.join(" ")];
  }

  return [];
}

export function buildRsyncArgs(
  source: StorageLocation,
  destination: StorageLocation,
  options: RsyncOptions,
  sshConfig: SshConfig | null
): string[] {
  const args: string[] = [];

  if (options.archive) args.push("-a");
  if (options.compress) args.push("-z");
  if (options.verbose) args.push("-v");
  if (options.delete) args.push("--delete");
  if (options.dry_run) args.push("--dry-run");
  if (options.partial) args.push("--partial");
  if (options.progress) args.push("--progress");
  if (options.human_readable) args.push("-h");

  for (const pattern of options.exclude_patterns) {
    args.push(`--exclude=${pattern}`);
  }

  for (const pattern of options.include_patterns) {
    args.push(`--include=${pattern}`);
  }

  if (options.bandwidth_limit !== null) {
    args.push(`--bwlimit=${options.bandwidth_limit}`);
  }

  if (sshConfig) {
    args.push(...buildSshArgs(sshConfig));
  }

  for (const arg of options.custom_args) {
    args.push(arg);
  }

  args.push(storageLocationToRsyncPath(source));
  args.push(storageLocationToRsyncPath(destination));

  return args;
}

export function buildCommandString(job: JobDefinition): string {
  const args = buildRsyncArgs(
    job.source,
    job.destination,
    job.options,
    job.ssh_config
  );
  return `rsync ${args.join(" ")}`;
}

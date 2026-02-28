import type {
  JobDefinition,
  StorageLocation,
  RsyncOptions,
  SshConfig,
} from "@/types/job";

function ensureTrailingSlash(path: string): string {
  return path.endsWith("/") ? path : `${path}/`;
}

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
  sshConfig: SshConfig | null,
  autoTrailingSlash: boolean = false,
): string[] {
  const args: string[] = [];

  // Core transfer
  if (options.core_transfer.archive) args.push("-a");
  if (options.core_transfer.compress) args.push("-z");
  if (options.core_transfer.partial) args.push("--partial");
  if (options.core_transfer.dry_run) args.push("--dry-run");
  // File handling
  if (options.file_handling.delete) args.push("--delete");
  if (options.file_handling.size_only) args.push("--size-only");
  if (options.file_handling.checksum) args.push("--checksum");
  if (options.file_handling.update) args.push("--update");
  if (options.file_handling.whole_file) args.push("--whole-file");
  if (options.file_handling.ignore_existing) args.push("--ignore-existing");
  if (options.file_handling.one_file_system) args.push("--one-file-system");
  // Metadata
  if (options.metadata.hard_links) args.push("--hard-links");
  if (options.metadata.acls) args.push("--acls");
  if (options.metadata.xattrs) args.push("--xattrs");
  if (options.metadata.numeric_ids) args.push("--numeric-ids");
  // Output
  if (options.output.verbose) args.push("-v");
  if (options.output.progress) args.push("--progress");
  if (options.output.human_readable) args.push("-h");
  if (options.output.stats) args.push("--stats");
  if (options.output.itemize_changes) args.push("--itemize-changes");
  // Patterns & advanced
  for (const pattern of options.advanced.exclude_patterns) {
    args.push(`--exclude=${pattern}`);
  }

  for (const pattern of options.advanced.include_patterns) {
    args.push(`--include=${pattern}`);
  }

  if (options.advanced.bandwidth_limit !== null) {
    args.push(`--bwlimit=${options.advanced.bandwidth_limit}`);
  }

  if (sshConfig) {
    args.push(...buildSshArgs(sshConfig));
  }

  for (const arg of options.advanced.custom_args) {
    args.push(arg);
  }

  const sourcePath = storageLocationToRsyncPath(source);
  const destPath = storageLocationToRsyncPath(destination);

  if (autoTrailingSlash) {
    args.push(ensureTrailingSlash(sourcePath));
    args.push(ensureTrailingSlash(destPath));
  } else {
    args.push(sourcePath);
    args.push(destPath);
  }

  return args;
}

export function buildCommandString(
  job: JobDefinition,
  autoTrailingSlash: boolean = false,
): string {
  const args = buildRsyncArgs(
    job.transfer.source,
    job.transfer.destination,
    job.options,
    job.ssh_config,
    autoTrailingSlash,
  );
  return `rsync ${args.join(" ")}`;
}

use crate::models::job::{RsyncOptions, SshConfig, StorageLocation};

fn ensure_trailing_slash(path: &str) -> String {
    if path.ends_with('/') {
        path.to_string()
    } else {
        format!("{}/", path)
    }
}

pub fn build_rsync_args(
    source: &StorageLocation,
    destination: &StorageLocation,
    options: &RsyncOptions,
    ssh_config: Option<&SshConfig>,
    link_dest: Option<&str>,
    auto_trailing_slash: bool,
) -> Vec<String> {
    let mut args = Vec::new();

    // Core transfer
    if options.core_transfer.archive {
        args.push("-a".to_string());
    }
    if options.core_transfer.compress {
        args.push("-z".to_string());
    }
    if options.core_transfer.partial {
        args.push("--partial".to_string());
    }
    if options.core_transfer.dry_run {
        args.push("--dry-run".to_string());
    }
    // File handling
    if options.file_handling.delete {
        args.push("--delete".to_string());
    }
    if options.file_handling.size_only {
        args.push("--size-only".to_string());
    }
    if options.file_handling.checksum {
        args.push("--checksum".to_string());
    }
    if options.file_handling.update {
        args.push("--update".to_string());
    }
    if options.file_handling.whole_file {
        args.push("--whole-file".to_string());
    }
    if options.file_handling.ignore_existing {
        args.push("--ignore-existing".to_string());
    }
    if options.file_handling.one_file_system {
        args.push("--one-file-system".to_string());
    }
    // Metadata
    if options.metadata.hard_links {
        args.push("--hard-links".to_string());
    }
    if options.metadata.acls {
        args.push("--acls".to_string());
    }
    if options.metadata.xattrs {
        args.push("--xattrs".to_string());
    }
    if options.metadata.numeric_ids {
        args.push("--numeric-ids".to_string());
    }
    // Output
    if options.output.verbose {
        args.push("-v".to_string());
    }
    if options.output.progress {
        args.push("--progress".to_string());
    }
    if options.output.human_readable {
        args.push("-h".to_string());
    }
    if options.output.stats {
        args.push("--stats".to_string());
    }
    if options.output.itemize_changes {
        args.push("--itemize-changes".to_string());
    }
    // Patterns & advanced
    for pattern in &options.advanced.exclude_patterns {
        args.push(format!("--exclude={}", pattern));
    }

    for pattern in &options.advanced.include_patterns {
        args.push(format!("--include={}", pattern));
    }

    if let Some(limit) = options.advanced.bandwidth_limit {
        args.push(format!("--bwlimit={}", limit));
    }

    if let Some(link) = link_dest {
        args.push(format!("--link-dest={}", link));
    }

    // Build SSH command if needed
    if let Some(ssh) = ssh_config {
        if let Some(ref custom_cmd) = ssh.custom_ssh_command {
            args.push(format!("-e {}", custom_cmd));
        } else {
            let mut ssh_parts = vec!["ssh".to_string()];

            if ssh.port != 22 {
                ssh_parts.push(format!("-p {}", ssh.port));
            }

            if let Some(ref key) = ssh.identity_file {
                ssh_parts.push(format!("-i {}", key));
            }

            if !ssh.strict_host_key_checking {
                ssh_parts.push("-o StrictHostKeyChecking=no".to_string());
            }

            if ssh_parts.len() > 1 {
                args.push("-e".to_string());
                args.push(ssh_parts.join(" "));
            }
        }
    }

    for arg in &options.advanced.custom_args {
        args.push(arg.clone());
    }

    let source_path = source.to_rsync_path();
    let dest_path = destination.to_rsync_path();

    if auto_trailing_slash {
        args.push(ensure_trailing_slash(&source_path));
        args.push(ensure_trailing_slash(&dest_path));
    } else {
        args.push(source_path);
        args.push(dest_path);
    }

    args
}

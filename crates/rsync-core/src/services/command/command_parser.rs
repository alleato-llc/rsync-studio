use crate::models::command::ParsedCommand;
use crate::models::job::{
    AdvancedOptions, CoreTransferOptions, FileHandlingOptions, JobDefinition, MetadataOptions,
    OutputOptions, RsyncOptions, SshConfig, StorageLocation,
};

/// Parse an rsync command string into its component parts.
///
/// Handles the `rsync` prefix, short flags (-a, -avz), long flags (--delete),
/// flags with values (--exclude=PATTERN, --bwlimit=1000), SSH -e flag,
/// and extracts source/destination as the last two non-flag arguments.
pub fn parse_rsync_command(command: &str) -> Result<ParsedCommand, String> {
    let parts = shell_words::split(command)
        .map_err(|e| format!("Failed to parse command: {}", e))?;

    let mut iter = parts.iter().peekable();

    // Skip "rsync" if it's the first token
    if let Some(first) = iter.peek() {
        if first.as_str() == "rsync" {
            iter.next();
        }
    }

    let mut flags = Vec::new();
    let mut exclude_patterns = Vec::new();
    let mut include_patterns = Vec::new();
    let mut bandwidth_limit: Option<u64> = None;
    let mut ssh_command: Option<String> = None;
    let mut link_dest: Option<String> = None;
    let mut custom_args = Vec::new();
    let mut positional: Vec<String> = Vec::new();

    while let Some(arg) = iter.next() {
        if arg == "--" {
            // Everything after -- is positional
            for remaining in iter.by_ref() {
                positional.push(remaining.clone());
            }
            break;
        }

        if let Some(rest) = arg.strip_prefix("--") {
            // Long option
            if let Some((key, value)) = rest.split_once('=') {
                handle_long_with_value(
                    key,
                    value,
                    &mut flags,
                    &mut exclude_patterns,
                    &mut include_patterns,
                    &mut bandwidth_limit,
                    &mut link_dest,
                    &mut custom_args,
                );
            } else {
                handle_long_flag(rest, &mut flags, &mut custom_args);
            }
        } else if arg.starts_with('-') && arg.len() > 1 {
            // Short option(s)
            let chars: Vec<char> = arg[1..].chars().collect();
            let mut i = 0;
            while i < chars.len() {
                let c = chars[i];
                if c == 'e' {
                    // -e takes the next argument as the SSH command
                    // If more chars follow in this token, they're part of -e value? No, typical usage is `-e "ssh ..."`.
                    // But if it's combined like `-ave "ssh..."`, the `e` consumes the next arg.
                    let next_val = if i + 1 < chars.len() {
                        // Remaining chars in this token are the value (unusual but possible)
                        Some(chars[i + 1..].iter().collect::<String>())
                    } else {
                        iter.next().cloned()
                    };
                    ssh_command = next_val;
                    break; // consumed rest of this token
                }
                handle_short_flag(c, &mut flags, &mut custom_args);
                i += 1;
            }
        } else {
            // Positional argument (source or destination)
            positional.push(arg.clone());
        }
    }

    // Last two positional args are source and destination
    let (source, destination) = match positional.len() {
        0 => (None, None),
        1 => (Some(positional[0].clone()), None),
        _ => {
            let dest = positional.pop();
            let src = positional.pop();
            // Any remaining positional args before src are also sources (rsync supports multiple),
            // but we put extras into custom_args for simplicity
            for extra in positional {
                custom_args.push(extra);
            }
            (src, dest)
        }
    };

    Ok(ParsedCommand {
        source,
        destination,
        flags,
        exclude_patterns,
        include_patterns,
        bandwidth_limit,
        ssh_command,
        link_dest,
        custom_args,
    })
}

fn handle_short_flag(c: char, flags: &mut Vec<String>, custom_args: &mut Vec<String>) {
    match c {
        'a' => flags.push("archive".to_string()),
        'z' => flags.push("compress".to_string()),
        'v' => flags.push("verbose".to_string()),
        'h' => flags.push("human_readable".to_string()),
        'r' => flags.push("recursive".to_string()),
        'l' => flags.push("links".to_string()),
        'p' => flags.push("perms".to_string()),
        't' => flags.push("times".to_string()),
        'g' => flags.push("group".to_string()),
        'o' => flags.push("owner".to_string()),
        'D' => flags.push("devices_specials".to_string()),
        'n' => flags.push("dry_run".to_string()),
        'u' => flags.push("update".to_string()),
        'c' => flags.push("checksum".to_string()),
        'P' => {
            flags.push("partial".to_string());
            flags.push("progress".to_string());
        }
        'q' => flags.push("quiet".to_string()),
        'H' => flags.push("hard_links".to_string()),
        'A' => flags.push("acls".to_string()),
        'X' => flags.push("xattrs".to_string()),
        'W' => flags.push("whole_file".to_string()),
        'x' => flags.push("one_file_system".to_string()),
        'i' => flags.push("itemize_changes".to_string()),
        'S' => flags.push("sparse".to_string()),
        'R' => flags.push("relative".to_string()),
        'K' => flags.push("keep_dirlinks".to_string()),
        'L' => flags.push("copy_links".to_string()),
        'k' => flags.push("copy_dirlinks".to_string()),
        'b' => flags.push("backup".to_string()),
        'y' => flags.push("fuzzy".to_string()),
        'm' => flags.push("prune_empty_dirs".to_string()),
        '4' => flags.push("ipv4".to_string()),
        '6' => flags.push("ipv6".to_string()),
        _ => custom_args.push(format!("-{}", c)),
    }
}

fn handle_long_flag(flag: &str, flags: &mut Vec<String>, custom_args: &mut Vec<String>) {
    match flag {
        // Core / archive
        "archive" => flags.push("archive".to_string()),
        "recursive" => flags.push("recursive".to_string()),
        "links" => flags.push("links".to_string()),
        "perms" => flags.push("perms".to_string()),
        "times" => flags.push("times".to_string()),
        "group" => flags.push("group".to_string()),
        "owner" => flags.push("owner".to_string()),
        "devices" => flags.push("devices".to_string()),
        "specials" => flags.push("specials".to_string()),

        // Transfer behavior
        "compress" => flags.push("compress".to_string()),
        "verbose" => flags.push("verbose".to_string()),
        "human-readable" => flags.push("human_readable".to_string()),
        "delete" => flags.push("delete".to_string()),
        "dry-run" => flags.push("dry_run".to_string()),
        "partial" => flags.push("partial".to_string()),
        "progress" => flags.push("progress".to_string()),
        "update" => flags.push("update".to_string()),
        "checksum" => flags.push("checksum".to_string()),
        "quiet" => flags.push("quiet".to_string()),
        "whole-file" => flags.push("whole_file".to_string()),
        "one-file-system" | "xdev" => flags.push("one_file_system".to_string()),
        "ignore-existing" => flags.push("ignore_existing".to_string()),
        "size-only" => flags.push("size_only".to_string()),
        "inplace" => flags.push("inplace".to_string()),
        "append" => flags.push("append".to_string()),
        "append-verify" => flags.push("append_verify".to_string()),
        "sparse" => flags.push("sparse".to_string()),
        "existing" => flags.push("existing".to_string()),
        "delay-updates" => flags.push("delay_updates".to_string()),
        "relative" => flags.push("relative".to_string()),
        "no-relative" | "no-R" => flags.push("no_relative".to_string()),

        // Symlink handling
        "copy-links" => flags.push("copy_links".to_string()),
        "copy-dirlinks" => flags.push("copy_dirlinks".to_string()),
        "keep-dirlinks" => flags.push("keep_dirlinks".to_string()),
        "safe-links" => flags.push("safe_links".to_string()),

        // Metadata
        "hard-links" => flags.push("hard_links".to_string()),
        "acls" => flags.push("acls".to_string()),
        "xattrs" => flags.push("xattrs".to_string()),
        "numeric-ids" => flags.push("numeric_ids".to_string()),
        "no-perms" | "no-p" => flags.push("no_perms".to_string()),
        "no-times" | "no-t" => flags.push("no_times".to_string()),
        "no-owner" | "no-o" => flags.push("no_owner".to_string()),
        "no-group" | "no-g" => flags.push("no_group".to_string()),
        "super" => flags.push("super_".to_string()),
        "fake-super" => flags.push("fake_super".to_string()),
        "no-implied-dirs" => flags.push("no_implied_dirs".to_string()),

        // Output
        "itemize-changes" => flags.push("itemize_changes".to_string()),
        "stats" => flags.push("stats".to_string()),
        "msgs2stderr" => flags.push("msgs2stderr".to_string()),

        // Deletion
        "delete-before" => flags.push("delete_before".to_string()),
        "delete-during" | "del" => flags.push("delete_during".to_string()),
        "delete-delay" => flags.push("delete_delay".to_string()),
        "delete-after" => flags.push("delete_after".to_string()),
        "delete-excluded" => flags.push("delete_excluded".to_string()),
        "force" => flags.push("force".to_string()),
        "ignore-errors" => flags.push("ignore_errors".to_string()),

        // Backup
        "backup" => flags.push("backup".to_string()),

        // File selection
        "prune-empty-dirs" => flags.push("prune_empty_dirs".to_string()),
        "fuzzy" => flags.push("fuzzy".to_string()),

        // Networking
        "blocking-io" => flags.push("blocking_io".to_string()),
        "ipv4" => flags.push("ipv4".to_string()),
        "ipv6" => flags.push("ipv6".to_string()),

        _ => custom_args.push(format!("--{}", flag)),
    }
}

fn handle_long_with_value(
    key: &str,
    value: &str,
    flags: &mut Vec<String>,
    exclude_patterns: &mut Vec<String>,
    include_patterns: &mut Vec<String>,
    bandwidth_limit: &mut Option<u64>,
    link_dest: &mut Option<String>,
    custom_args: &mut Vec<String>,
) {
    match key {
        "exclude" => exclude_patterns.push(value.to_string()),
        "include" => include_patterns.push(value.to_string()),
        "bwlimit" => {
            *bandwidth_limit = value.parse().ok();
            if bandwidth_limit.is_none() {
                custom_args.push(format!("--bwlimit={}", value));
            }
        }
        "link-dest" => *link_dest = Some(value.to_string()),
        "backup-dir" => {
            flags.push("backup_dir".to_string());
            custom_args.push(format!("--backup-dir={}", value));
        }
        // Recognized value-carrying flags — store as flag name for explanation
        "log-file" => {
            flags.push("log_file".to_string());
            custom_args.push(format!("--log-file={}", value));
        }
        "out-format" => {
            flags.push("out_format".to_string());
            custom_args.push(format!("--out-format={}", value));
        }
        "info" => {
            flags.push("info".to_string());
            custom_args.push(format!("--info={}", value));
        }
        "debug" => {
            flags.push("debug".to_string());
            custom_args.push(format!("--debug={}", value));
        }
        "max-size" => {
            flags.push("max_size".to_string());
            custom_args.push(format!("--max-size={}", value));
        }
        "min-size" => {
            flags.push("min_size".to_string());
            custom_args.push(format!("--min-size={}", value));
        }
        "max-delete" => {
            flags.push("max_delete".to_string());
            custom_args.push(format!("--max-delete={}", value));
        }
        "timeout" => {
            flags.push("timeout".to_string());
            custom_args.push(format!("--timeout={}", value));
        }
        "contimeout" => {
            flags.push("contimeout".to_string());
            custom_args.push(format!("--contimeout={}", value));
        }
        "address" => {
            flags.push("address".to_string());
            custom_args.push(format!("--address={}", value));
        }
        "port" => {
            flags.push("port".to_string());
            custom_args.push(format!("--port={}", value));
        }
        "rsync-path" => {
            flags.push("rsync_path".to_string());
            custom_args.push(format!("--rsync-path={}", value));
        }
        "suffix" => {
            flags.push("suffix".to_string());
            custom_args.push(format!("--suffix={}", value));
        }
        "temp-dir" => {
            flags.push("temp_dir".to_string());
            custom_args.push(format!("--temp-dir={}", value));
        }
        "compare-dest" => {
            flags.push("compare_dest".to_string());
            custom_args.push(format!("--compare-dest={}", value));
        }
        "copy-dest" => {
            flags.push("copy_dest".to_string());
            custom_args.push(format!("--copy-dest={}", value));
        }
        "filter" => {
            flags.push("filter".to_string());
            custom_args.push(format!("--filter={}", value));
        }
        "chmod" => {
            flags.push("chmod".to_string());
            custom_args.push(format!("--chmod={}", value));
        }
        "chown" => {
            flags.push("chown".to_string());
            custom_args.push(format!("--chown={}", value));
        }
        "compress-level" => {
            flags.push("compress_level".to_string());
            custom_args.push(format!("--compress-level={}", value));
        }
        "skip-compress" => {
            flags.push("skip_compress".to_string());
            custom_args.push(format!("--skip-compress={}", value));
        }
        "files-from" => {
            flags.push("files_from".to_string());
            custom_args.push(format!("--files-from={}", value));
        }
        "exclude-from" => {
            flags.push("exclude_from".to_string());
            custom_args.push(format!("--exclude-from={}", value));
        }
        "include-from" => {
            flags.push("include_from".to_string());
            custom_args.push(format!("--include-from={}", value));
        }
        "iconv" => {
            flags.push("iconv".to_string());
            custom_args.push(format!("--iconv={}", value));
        }
        "rsh" | "log-file-format" => {
            custom_args.push(format!("--{}={}", key, value));
        }
        _ => custom_args.push(format!("--{}={}", key, value)),
    }
}

/// Parse a path string into a StorageLocation.
pub fn parse_storage_location(path: &str) -> StorageLocation {
    // rsync://host/module/path
    if let Some(rest) = path.strip_prefix("rsync://") {
        let parts: Vec<&str> = rest.splitn(3, '/').collect();
        return StorageLocation::RemoteRsync {
            host: parts.first().unwrap_or(&"").to_string(),
            module: parts.get(1).unwrap_or(&"").to_string(),
            path: parts.get(2).unwrap_or(&"").to_string(),
        };
    }

    // user@host:path (SSH)
    if let Some(colon_pos) = path.find(':') {
        let before_colon = &path[..colon_pos];
        // Only treat as SSH if there's an @ before the colon (or it looks like host:path)
        if let Some(at_pos) = before_colon.find('@') {
            let user = &before_colon[..at_pos];
            let host = &before_colon[at_pos + 1..];
            let remote_path = &path[colon_pos + 1..];
            return StorageLocation::RemoteSsh {
                user: user.to_string(),
                host: host.to_string(),
                port: 22,
                path: remote_path.to_string(),
                identity_file: None,
            };
        }
        // host:path without @ — treat as SSH with empty user
        let host = before_colon;
        let remote_path = &path[colon_pos + 1..];
        // But skip windows-style paths like C:\
        if host.len() == 1 && host.chars().next().map_or(false, |c| c.is_ascii_alphabetic()) {
            return StorageLocation::Local {
                path: path.to_string(),
            };
        }
        return StorageLocation::RemoteSsh {
            user: String::new(),
            host: host.to_string(),
            port: 22,
            path: remote_path.to_string(),
            identity_file: None,
        };
    }

    // Local path
    StorageLocation::Local {
        path: path.to_string(),
    }
}

/// Attempt to convert a ParsedCommand into a partial JobDefinition.
///
/// This creates a job with defaults for fields that can't be inferred from the command.
pub fn to_job_definition(parsed: &ParsedCommand) -> Result<JobDefinition, String> {
    let source = parsed
        .source
        .as_ref()
        .map(|s| parse_storage_location(s))
        .unwrap_or(StorageLocation::Local {
            path: String::new(),
        });

    let destination = parsed
        .destination
        .as_ref()
        .map(|d| parse_storage_location(d))
        .unwrap_or(StorageLocation::Local {
            path: String::new(),
        });

    let has = |name: &str| parsed.flags.contains(&name.to_string());

    let options = RsyncOptions {
        core_transfer: CoreTransferOptions {
            archive: has("archive"),
            compress: has("compress"),
            partial: has("partial"),
            dry_run: has("dry_run"),
        },
        file_handling: FileHandlingOptions {
            delete: has("delete"),
            size_only: has("size_only"),
            checksum: has("checksum"),
            update: has("update"),
            whole_file: has("whole_file"),
            ignore_existing: has("ignore_existing"),
            one_file_system: has("one_file_system"),
        },
        metadata: MetadataOptions {
            hard_links: has("hard_links"),
            acls: has("acls"),
            xattrs: has("xattrs"),
            numeric_ids: has("numeric_ids"),
        },
        output: OutputOptions {
            verbose: has("verbose"),
            progress: has("progress"),
            human_readable: has("human_readable"),
            stats: has("stats"),
            itemize_changes: has("itemize_changes"),
        },
        advanced: AdvancedOptions {
            exclude_patterns: parsed.exclude_patterns.clone(),
            include_patterns: parsed.include_patterns.clone(),
            bandwidth_limit: parsed.bandwidth_limit,
            custom_args: parsed.custom_args.clone(),
        },
    };

    // Parse SSH config from -e flag
    let ssh_config = parsed.ssh_command.as_ref().and_then(|cmd| parse_ssh_command(cmd));

    // Determine if SSH is needed from the locations
    let needs_ssh = matches!(&source, StorageLocation::RemoteSsh { .. })
        || matches!(&destination, StorageLocation::RemoteSsh { .. });
    let ssh_config = if needs_ssh {
        Some(ssh_config.unwrap_or_default())
    } else {
        ssh_config
    };

    let now = chrono::Utc::now();

    Ok(JobDefinition {
        id: uuid::Uuid::new_v4(),
        name: String::new(), // To be filled by user
        description: None,
        transfer: crate::models::job::TransferConfig {
            source,
            destination,
            backup_mode: crate::models::job::BackupMode::Mirror,
        },
        options,
        ssh_config,
        schedule: None,
        enabled: true,
        created_at: now,
        updated_at: now,
    })
}

/// Parse an SSH command string (from -e flag) into an SshConfig.
pub(crate) fn parse_ssh_command(cmd: &str) -> Option<SshConfig> {
    let parts = shell_words::split(cmd).ok()?;
    let mut config = SshConfig::default();
    let mut has_custom = false;

    let mut iter = parts.iter().peekable();
    // Skip "ssh" prefix if present
    if let Some(first) = iter.peek() {
        if first.as_str() == "ssh" {
            iter.next();
        } else {
            // Custom SSH command
            config.custom_ssh_command = Some(cmd.to_string());
            return Some(config);
        }
    }

    while let Some(part) = iter.next() {
        if part == "-p" {
            if let Some(port_str) = iter.next() {
                if let Ok(port) = port_str.parse::<u16>() {
                    config.port = port;
                }
            }
        } else if part == "-i" {
            config.identity_file = iter.next().cloned();
        } else if part == "-o" {
            if let Some(opt) = iter.next() {
                if opt == "StrictHostKeyChecking=no" {
                    config.strict_host_key_checking = false;
                } else {
                    has_custom = true;
                }
            }
        } else {
            has_custom = true;
        }
    }

    if has_custom && config == SshConfig::default() {
        config.custom_ssh_command = Some(cmd.to_string());
    }

    Some(config)
}

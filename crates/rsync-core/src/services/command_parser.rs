use serde::{Deserialize, Serialize};

use crate::models::job::{
    JobDefinition, RsyncOptions, SshConfig, StorageLocation,
};

/// Result of parsing an rsync command string.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParsedCommand {
    /// The source path/location as a raw string
    pub source: Option<String>,
    /// The destination path/location as a raw string
    pub destination: Option<String>,
    /// Recognized boolean flags (e.g., "archive", "compress")
    pub flags: Vec<String>,
    /// Exclude patterns found
    pub exclude_patterns: Vec<String>,
    /// Include patterns found
    pub include_patterns: Vec<String>,
    /// Bandwidth limit if specified
    pub bandwidth_limit: Option<u64>,
    /// SSH command string if -e was used
    pub ssh_command: Option<String>,
    /// Link-dest path if specified
    pub link_dest: Option<String>,
    /// Arguments not recognized by the parser
    pub custom_args: Vec<String>,
}

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
        _ => custom_args.push(format!("-{}", c)),
    }
}

fn handle_long_flag(flag: &str, flags: &mut Vec<String>, custom_args: &mut Vec<String>) {
    match flag {
        "archive" => flags.push("archive".to_string()),
        "compress" => flags.push("compress".to_string()),
        "verbose" => flags.push("verbose".to_string()),
        "human-readable" => flags.push("human_readable".to_string()),
        "delete" => flags.push("delete".to_string()),
        "dry-run" => flags.push("dry_run".to_string()),
        "partial" => flags.push("partial".to_string()),
        "progress" => flags.push("progress".to_string()),
        "recursive" => flags.push("recursive".to_string()),
        "links" => flags.push("links".to_string()),
        "perms" => flags.push("perms".to_string()),
        "times" => flags.push("times".to_string()),
        "group" => flags.push("group".to_string()),
        "owner" => flags.push("owner".to_string()),
        "devices" => flags.push("devices".to_string()),
        "specials" => flags.push("specials".to_string()),
        "update" => flags.push("update".to_string()),
        "checksum" => flags.push("checksum".to_string()),
        "quiet" => flags.push("quiet".to_string()),
        "whole-file" => flags.push("whole_file".to_string()),
        "one-file-system" | "xdev" => flags.push("one_file_system".to_string()),
        "ignore-existing" => flags.push("ignore_existing".to_string()),
        "size-only" => flags.push("size_only".to_string()),
        "itemize-changes" => flags.push("itemize_changes".to_string()),
        "stats" => flags.push("stats".to_string()),
        "no-implied-dirs" => flags.push("no_implied_dirs".to_string()),
        "numeric-ids" => flags.push("numeric_ids".to_string()),
        "hard-links" | "H" => flags.push("hard_links".to_string()),
        "acls" | "A" => flags.push("acls".to_string()),
        "xattrs" | "X" => flags.push("xattrs".to_string()),
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
        "info" | "debug" | "log-file" | "log-file-format" | "out-format"
        | "max-size" | "min-size" | "max-delete" | "timeout" | "contimeout"
        | "rsh" | "rsync-path" | "suffix" | "temp-dir" | "compare-dest"
        | "copy-dest" | "filter" | "chmod" | "chown" => {
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

    let options = RsyncOptions {
        archive: parsed.flags.contains(&"archive".to_string()),
        compress: parsed.flags.contains(&"compress".to_string()),
        verbose: parsed.flags.contains(&"verbose".to_string()),
        delete: parsed.flags.contains(&"delete".to_string()),
        dry_run: parsed.flags.contains(&"dry_run".to_string()),
        partial: parsed.flags.contains(&"partial".to_string()),
        progress: parsed.flags.contains(&"progress".to_string()),
        human_readable: parsed.flags.contains(&"human_readable".to_string()),
        exclude_patterns: parsed.exclude_patterns.clone(),
        include_patterns: parsed.include_patterns.clone(),
        bandwidth_limit: parsed.bandwidth_limit,
        custom_args: parsed.custom_args.clone(),
        size_only: parsed.flags.contains(&"size_only".to_string()),
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
        source,
        destination,
        backup_mode: crate::models::job::BackupMode::Mirror,
        options,
        ssh_config,
        schedule: None,
        enabled: true,
        created_at: now,
        updated_at: now,
    })
}

/// Parse an SSH command string (from -e flag) into an SshConfig.
fn parse_ssh_command(cmd: &str) -> Option<SshConfig> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_command() {
        let parsed = parse_rsync_command("rsync -a /src/ /dst/").unwrap();
        assert_eq!(parsed.source.as_deref(), Some("/src/"));
        assert_eq!(parsed.destination.as_deref(), Some("/dst/"));
        assert!(parsed.flags.contains(&"archive".to_string()));
    }

    #[test]
    fn parse_combined_short_flags() {
        let parsed = parse_rsync_command("rsync -avz /src/ /dst/").unwrap();
        assert!(parsed.flags.contains(&"archive".to_string()));
        assert!(parsed.flags.contains(&"verbose".to_string()));
        assert!(parsed.flags.contains(&"compress".to_string()));
    }

    #[test]
    fn parse_long_flags() {
        let parsed =
            parse_rsync_command("rsync --archive --delete --dry-run /src/ /dst/").unwrap();
        assert!(parsed.flags.contains(&"archive".to_string()));
        assert!(parsed.flags.contains(&"delete".to_string()));
        assert!(parsed.flags.contains(&"dry_run".to_string()));
    }

    #[test]
    fn parse_exclude_patterns() {
        let parsed = parse_rsync_command(
            "rsync -a --exclude=*.log --exclude=tmp/ /src/ /dst/",
        )
        .unwrap();
        assert_eq!(parsed.exclude_patterns, vec!["*.log", "tmp/"]);
    }

    #[test]
    fn parse_include_patterns() {
        let parsed =
            parse_rsync_command("rsync -a --include=*.rs --include=*.toml /src/ /dst/")
                .unwrap();
        assert_eq!(parsed.include_patterns, vec!["*.rs", "*.toml"]);
    }

    #[test]
    fn parse_bandwidth_limit() {
        let parsed =
            parse_rsync_command("rsync -a --bwlimit=1000 /src/ /dst/").unwrap();
        assert_eq!(parsed.bandwidth_limit, Some(1000));
    }

    #[test]
    fn parse_link_dest() {
        let parsed =
            parse_rsync_command("rsync -a --link-dest=/prev /src/ /dst/").unwrap();
        assert_eq!(parsed.link_dest.as_deref(), Some("/prev"));
    }

    #[test]
    fn parse_ssh_flag() {
        let parsed = parse_rsync_command(
            r#"rsync -a -e "ssh -p 2222 -i /home/user/.ssh/key" /src/ user@host:/dst/"#,
        )
        .unwrap();
        assert_eq!(
            parsed.ssh_command.as_deref(),
            Some("ssh -p 2222 -i /home/user/.ssh/key")
        );
        assert_eq!(parsed.destination.as_deref(), Some("user@host:/dst/"));
    }

    #[test]
    fn parse_remote_ssh_paths() {
        let loc = parse_storage_location("admin@server.example.com:/data/backup/");
        assert_eq!(
            loc,
            StorageLocation::RemoteSsh {
                user: "admin".to_string(),
                host: "server.example.com".to_string(),
                port: 22,
                path: "/data/backup/".to_string(),
                identity_file: None,
            }
        );
    }

    #[test]
    fn parse_remote_rsync_paths() {
        let loc = parse_storage_location("rsync://rsync.example.com/backups/daily/");
        assert_eq!(
            loc,
            StorageLocation::RemoteRsync {
                host: "rsync.example.com".to_string(),
                module: "backups".to_string(),
                path: "daily/".to_string(),
            }
        );
    }

    #[test]
    fn parse_local_path() {
        let loc = parse_storage_location("/home/user/docs/");
        assert_eq!(
            loc,
            StorageLocation::Local {
                path: "/home/user/docs/".to_string()
            }
        );
    }

    #[test]
    fn parse_without_rsync_prefix() {
        let parsed = parse_rsync_command("-avz /src/ /dst/").unwrap();
        assert!(parsed.flags.contains(&"archive".to_string()));
        assert_eq!(parsed.source.as_deref(), Some("/src/"));
    }

    #[test]
    fn parse_capital_p_flag() {
        let parsed = parse_rsync_command("rsync -aP /src/ /dst/").unwrap();
        assert!(parsed.flags.contains(&"archive".to_string()));
        assert!(parsed.flags.contains(&"partial".to_string()));
        assert!(parsed.flags.contains(&"progress".to_string()));
    }

    #[test]
    fn parse_unknown_flags_go_to_custom() {
        let parsed =
            parse_rsync_command("rsync -a --weird-flag --foo=bar /src/ /dst/").unwrap();
        assert!(parsed.custom_args.contains(&"--weird-flag".to_string()));
        assert!(parsed.custom_args.contains(&"--foo=bar".to_string()));
    }

    #[test]
    fn to_job_definition_basic() {
        let parsed = parse_rsync_command("rsync -avz --delete /src/ /dst/").unwrap();
        let job = to_job_definition(&parsed).unwrap();
        assert!(job.options.archive);
        assert!(job.options.verbose);
        assert!(job.options.compress);
        assert!(job.options.delete);
        assert_eq!(
            job.source,
            StorageLocation::Local {
                path: "/src/".to_string()
            }
        );
        assert_eq!(
            job.destination,
            StorageLocation::Local {
                path: "/dst/".to_string()
            }
        );
    }

    #[test]
    fn to_job_definition_with_ssh() {
        let parsed = parse_rsync_command(
            r#"rsync -a -e "ssh -p 2222" /src/ admin@server:/backup/"#,
        )
        .unwrap();
        let job = to_job_definition(&parsed).unwrap();
        let ssh = job.ssh_config.unwrap();
        assert_eq!(ssh.port, 2222);
        assert!(matches!(
            job.destination,
            StorageLocation::RemoteSsh { .. }
        ));
    }

    #[test]
    fn parse_ssh_command_parts() {
        let config = parse_ssh_command("ssh -p 2222 -i /key -o StrictHostKeyChecking=no");
        let config = config.unwrap();
        assert_eq!(config.port, 2222);
        assert_eq!(config.identity_file.as_deref(), Some("/key"));
        assert!(!config.strict_host_key_checking);
    }

    #[test]
    fn roundtrip_simple_command() {
        use crate::services::command_builder::build_rsync_args;

        let source = StorageLocation::Local {
            path: "/src/".to_string(),
        };
        let dest = StorageLocation::Local {
            path: "/dst/".to_string(),
        };
        let opts = RsyncOptions {
            archive: true,
            compress: true,
            verbose: true,
            delete: true,
            exclude_patterns: vec!["*.log".to_string()],
            bandwidth_limit: Some(500),
            ..RsyncOptions::default()
        };

        let args = build_rsync_args(&source, &dest, &opts, None, None, false);
        let cmd = format!("rsync {}", args.join(" "));

        let parsed = parse_rsync_command(&cmd).unwrap();
        assert!(parsed.flags.contains(&"archive".to_string()));
        assert!(parsed.flags.contains(&"compress".to_string()));
        assert!(parsed.flags.contains(&"verbose".to_string()));
        assert!(parsed.flags.contains(&"delete".to_string()));
        assert_eq!(parsed.exclude_patterns, vec!["*.log"]);
        assert_eq!(parsed.bandwidth_limit, Some(500));
        assert_eq!(parsed.source.as_deref(), Some("/src/"));
        assert_eq!(parsed.destination.as_deref(), Some("/dst/"));
    }
}

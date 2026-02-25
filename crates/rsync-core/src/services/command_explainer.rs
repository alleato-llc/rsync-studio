use serde::{Deserialize, Serialize};

use crate::services::command_parser::ParsedCommand;

/// A single argument with its explanation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArgumentExplanation {
    /// The raw argument as it appeared in the command
    pub argument: String,
    /// Human-readable explanation
    pub description: String,
    /// Category of the argument
    pub category: ArgCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ArgCategory {
    Flag,
    Pattern,
    Path,
    Ssh,
    Performance,
    Unknown,
}

/// Full explanation of a parsed rsync command.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommandExplanation {
    /// Per-argument explanations
    pub arguments: Vec<ArgumentExplanation>,
    /// Overall summary of what the command does
    pub summary: String,
}

/// Get a human-readable description for a recognized flag name.
pub fn explain_flag(flag: &str) -> &'static str {
    match flag {
        "archive" => "Archive mode (-a): preserves permissions, timestamps, symlinks, owner, group, and recurses into directories. Equivalent to -rlptgoD.",
        "compress" => "Compress (-z): compresses data during transfer to reduce bandwidth usage.",
        "verbose" => "Verbose (-v): increases the amount of information displayed during transfer.",
        "human_readable" => "Human-readable (-h): outputs numbers in a more human-readable format (e.g., 1.5M instead of 1572864).",
        "delete" => "Delete (--delete): removes files from the destination that don't exist in the source, making it a true mirror.",
        "dry_run" => "Dry run (-n/--dry-run): simulates the transfer without making any changes. Useful for previewing what would happen.",
        "partial" => "Partial (--partial): keeps partially transferred files so interrupted transfers can resume.",
        "progress" => "Progress (--progress): shows transfer progress for each file during the sync.",
        "recursive" => "Recursive (-r): recurses into directories. Already included in archive mode (-a).",
        "links" => "Links (-l): copies symlinks as symlinks. Already included in archive mode (-a).",
        "perms" => "Perms (-p): preserves file permissions. Already included in archive mode (-a).",
        "times" => "Times (-t): preserves file modification times. Already included in archive mode (-a).",
        "group" => "Group (-g): preserves the group ownership of files. Already included in archive mode (-a).",
        "owner" => "Owner (-o): preserves the owner of files. Already included in archive mode (-a).",
        "devices_specials" => "Devices & Specials (-D): preserves device files and special files. Already included in archive mode (-a).",
        "update" => "Update (-u): skips files that are newer on the destination than the source.",
        "checksum" => "Checksum (-c): uses checksums instead of file size and modification time to decide whether to transfer a file.",
        "quiet" => "Quiet (-q): suppresses non-error messages during transfer.",
        "whole_file" => "Whole file (-W/--whole-file): disables rsync's delta-transfer algorithm and transfers whole files. Faster on fast networks.",
        "one_file_system" => "One file system (-x/--one-file-system): doesn't cross filesystem boundaries when recursing.",
        "ignore_existing" => "Ignore existing (--ignore-existing): skips files that already exist on the destination.",
        "size_only" => "Size only (--size-only): compares files by size only, ignoring modification times. Essential for NAS/SMB mounts where timestamps are unreliable.",
        "itemize_changes" => "Itemize changes (-i/--itemize-changes): outputs a change-summary for all updates.",
        "stats" => "Stats (--stats): prints a set of statistics about the file transfer at the end.",
        "no_implied_dirs" => "No implied dirs (--no-implied-dirs): doesn't send implied directory info, affecting how relative paths are handled.",
        "numeric_ids" => "Numeric IDs (--numeric-ids): transfers numeric group and user IDs rather than mapping them by name.",
        "hard_links" => "Hard links (-H/--hard-links): preserves hard links between files.",
        "acls" => "ACLs (-A/--acls): preserves Access Control Lists.",
        "xattrs" => "Extended attributes (-X/--xattrs): preserves extended attributes.",
        "backup_dir" => "Backup dir (--backup-dir=DIR): moves replaced/deleted files to the specified backup directory.",
        "devices" => "Devices (--devices): preserves device files. Part of -D.",
        "specials" => "Specials (--specials): preserves special files. Part of -D.",
        _ => "Unknown flag.",
    }
}

/// Generate a full explanation for a parsed rsync command.
pub fn explain_command(parsed: &ParsedCommand) -> CommandExplanation {
    let mut arguments = Vec::new();

    // Explain flags
    for flag in &parsed.flags {
        arguments.push(ArgumentExplanation {
            argument: flag.clone(),
            description: explain_flag(flag).to_string(),
            category: ArgCategory::Flag,
        });
    }

    // Explain exclude patterns
    for pattern in &parsed.exclude_patterns {
        arguments.push(ArgumentExplanation {
            argument: format!("--exclude={}", pattern),
            description: format!(
                "Exclude files matching the pattern '{}' from the transfer.",
                pattern
            ),
            category: ArgCategory::Pattern,
        });
    }

    // Explain include patterns
    for pattern in &parsed.include_patterns {
        arguments.push(ArgumentExplanation {
            argument: format!("--include={}", pattern),
            description: format!(
                "Include files matching the pattern '{}' in the transfer (overrides excludes).",
                pattern
            ),
            category: ArgCategory::Pattern,
        });
    }

    // Explain bandwidth limit
    if let Some(limit) = parsed.bandwidth_limit {
        arguments.push(ArgumentExplanation {
            argument: format!("--bwlimit={}", limit),
            description: format!(
                "Limits the transfer bandwidth to {} KB/s to avoid saturating the network.",
                limit
            ),
            category: ArgCategory::Performance,
        });
    }

    // Explain link-dest
    if let Some(ref link_dest) = parsed.link_dest {
        arguments.push(ArgumentExplanation {
            argument: format!("--link-dest={}", link_dest),
            description: format!(
                "Uses '{}' as a reference directory. Unchanged files are hard-linked from this directory instead of being copied, saving disk space.",
                link_dest
            ),
            category: ArgCategory::Performance,
        });
    }

    // Explain SSH command
    if let Some(ref ssh_cmd) = parsed.ssh_command {
        arguments.push(ArgumentExplanation {
            argument: format!("-e \"{}\"", ssh_cmd),
            description: format!(
                "Uses a custom remote shell command: '{}'. This configures how rsync connects to remote hosts.",
                ssh_cmd
            ),
            category: ArgCategory::Ssh,
        });
    }

    // Explain source
    if let Some(ref source) = parsed.source {
        arguments.push(ArgumentExplanation {
            argument: source.clone(),
            description: format!("Source: files will be read from '{}'.", source),
            category: ArgCategory::Path,
        });
    }

    // Explain destination
    if let Some(ref dest) = parsed.destination {
        arguments.push(ArgumentExplanation {
            argument: dest.clone(),
            description: format!("Destination: files will be written to '{}'.", dest),
            category: ArgCategory::Path,
        });
    }

    // Explain custom/unknown args
    for arg in &parsed.custom_args {
        arguments.push(ArgumentExplanation {
            argument: arg.clone(),
            description: format!(
                "Unrecognized argument '{}'. This will be passed directly to rsync.",
                arg
            ),
            category: ArgCategory::Unknown,
        });
    }

    // Build summary
    let summary = build_summary(parsed);

    CommandExplanation {
        arguments,
        summary,
    }
}

fn build_summary(parsed: &ParsedCommand) -> String {
    let mut parts = Vec::new();

    let has_archive = parsed.flags.contains(&"archive".to_string());
    let has_delete = parsed.flags.contains(&"delete".to_string());
    let has_dry_run = parsed.flags.contains(&"dry_run".to_string());
    let has_compress = parsed.flags.contains(&"compress".to_string());

    if has_dry_run {
        parts.push("This is a DRY RUN — no actual changes will be made.".to_string());
    }

    if has_archive && has_delete {
        parts.push("Mirrors the source to the destination, preserving all file attributes and deleting files not present in the source.".to_string());
    } else if has_archive {
        parts.push("Syncs files from source to destination, preserving permissions, timestamps, and other attributes.".to_string());
    } else {
        parts.push("Transfers files from source to destination.".to_string());
    }

    if has_compress {
        parts.push("Data is compressed during transfer.".to_string());
    }

    if !parsed.exclude_patterns.is_empty() {
        parts.push(format!(
            "{} pattern(s) are excluded.",
            parsed.exclude_patterns.len()
        ));
    }

    if parsed.bandwidth_limit.is_some() {
        parts.push("Bandwidth is limited.".to_string());
    }

    if parsed.link_dest.is_some() {
        parts.push("Using hard-link deduplication from a reference snapshot.".to_string());
    }

    if parsed.ssh_command.is_some() {
        parts.push("Connecting via custom SSH configuration.".to_string());
    }

    parts.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::command_parser::parse_rsync_command;

    #[test]
    fn explain_basic_mirror() {
        let parsed = parse_rsync_command("rsync -a --delete /src/ /dst/").unwrap();
        let explanation = explain_command(&parsed);

        assert!(explanation.summary.contains("Mirrors"));
        assert!(explanation
            .arguments
            .iter()
            .any(|a| a.argument == "archive"));
        assert!(explanation.arguments.iter().any(|a| a.argument == "delete"));
    }

    #[test]
    fn explain_dry_run_mentioned() {
        let parsed = parse_rsync_command("rsync -an /src/ /dst/").unwrap();
        let explanation = explain_command(&parsed);
        assert!(explanation.summary.contains("DRY RUN"));
    }

    #[test]
    fn explain_excludes() {
        let parsed =
            parse_rsync_command("rsync -a --exclude=*.log --exclude=.git /src/ /dst/")
                .unwrap();
        let explanation = explain_command(&parsed);
        assert!(explanation.summary.contains("2 pattern(s) are excluded"));
        assert!(explanation
            .arguments
            .iter()
            .any(|a| a.argument == "--exclude=*.log" && a.category == ArgCategory::Pattern));
    }

    #[test]
    fn explain_unknown_args() {
        let parsed =
            parse_rsync_command("rsync -a --weird-flag /src/ /dst/").unwrap();
        let explanation = explain_command(&parsed);
        assert!(explanation
            .arguments
            .iter()
            .any(|a| a.argument == "--weird-flag" && a.category == ArgCategory::Unknown));
    }

    #[test]
    fn explain_ssh_command() {
        let parsed = parse_rsync_command(
            r#"rsync -a -e "ssh -p 2222" /src/ user@host:/dst/"#,
        )
        .unwrap();
        let explanation = explain_command(&parsed);
        assert!(explanation.summary.contains("SSH"));
        assert!(explanation
            .arguments
            .iter()
            .any(|a| a.category == ArgCategory::Ssh));
    }

    #[test]
    fn explain_bandwidth_limit() {
        let parsed =
            parse_rsync_command("rsync -a --bwlimit=500 /src/ /dst/").unwrap();
        let explanation = explain_command(&parsed);
        assert!(explanation.summary.contains("Bandwidth is limited"));
        assert!(explanation
            .arguments
            .iter()
            .any(|a| a.argument == "--bwlimit=500" && a.category == ArgCategory::Performance));
    }

    #[test]
    fn explain_link_dest() {
        let parsed = parse_rsync_command(
            "rsync -a --link-dest=/prev/snap /src/ /dst/",
        )
        .unwrap();
        let explanation = explain_command(&parsed);
        assert!(explanation.summary.contains("hard-link deduplication"));
    }

    #[test]
    fn all_flag_descriptions_are_non_empty() {
        let known_flags = [
            "archive",
            "compress",
            "verbose",
            "human_readable",
            "delete",
            "dry_run",
            "partial",
            "progress",
            "recursive",
            "links",
            "perms",
            "times",
            "group",
            "owner",
            "devices_specials",
            "update",
            "checksum",
            "quiet",
            "whole_file",
            "one_file_system",
            "ignore_existing",
            "size_only",
            "itemize_changes",
            "stats",
            "hard_links",
            "acls",
            "xattrs",
        ];

        for flag in &known_flags {
            let desc = explain_flag(flag);
            assert!(
                desc != "Unknown flag.",
                "Flag '{}' has no description",
                flag
            );
        }
    }
}
